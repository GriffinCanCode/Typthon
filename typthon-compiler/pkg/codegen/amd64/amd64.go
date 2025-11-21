// Package amd64 implements x86-64 code generation.
//
// Design: Direct assembly generation, no LLVM dependencies.
// System V calling convention for Unix/macOS.
// Fast compilation over perfect code - optimize for compile speed.
package amd64

import (
	"fmt"
	"io"
	"strings"

	"github.com/GriffinCanCode/typthon-compiler/pkg/codegen/regalloc"
	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
	"github.com/GriffinCanCode/typthon-compiler/pkg/ssa"
)

// Generator generates x86-64 assembly
type Generator struct {
	w         io.Writer
	alloc     *regalloc.Allocator
	paramMap  map[*ir.Param]int
	stackSize int
	phiMoves  map[*ssa.Block][]phiMove
}

type phiMove struct {
	src  ir.Value
	dest ir.Value
}

func NewGenerator(w io.Writer) *Generator {
	return &Generator{
		w:        w,
		paramMap: make(map[*ir.Param]int),
		phiMoves: make(map[*ssa.Block][]phiMove),
	}
}

// Generate emits assembly for an SSA program
func (g *Generator) Generate(prog *ssa.Program) error {
	logger.Debug("Generating amd64 assembly", "functions", len(prog.Functions))

	// Emit assembly header
	fmt.Fprintf(g.w, "\t.text\n")

	for _, fn := range prog.Functions {
		logger.Debug("Generating function assembly", "arch", "amd64", "name", fn.Name)
		if err := g.generateFunction(fn); err != nil {
			logger.Error("Failed to generate function", "arch", "amd64", "name", fn.Name, "error", err)
			return err
		}
	}

	logger.Info("amd64 code generation complete", "functions", len(prog.Functions))
	return nil
}

// GenerateWithValidation generates and validates assembly
func (g *Generator) GenerateWithValidation(prog *ssa.Program) (string, error) {
	// Generate to a buffer first
	var buf strings.Builder
	g.w = &buf

	if err := g.Generate(prog); err != nil {
		return "", fmt.Errorf("generation failed: %w", err)
	}

	assembly := buf.String()

	// Validate the generated assembly
	if err := ValidateProgram(assembly); err != nil {
		logger.Error("Assembly validation failed", "error", err)
		return assembly, fmt.Errorf("validation failed: %w", err)
	}

	logger.Info("Assembly generated and validated successfully")
	return assembly, nil
}

// generateFunction emits assembly for a single function
func (g *Generator) generateFunction(fn *ssa.Function) error {
	g.paramMap = make(map[*ir.Param]int)
	g.phiMoves = make(map[*ssa.Block][]phiMove)

	instCount := 0
	for _, block := range fn.Blocks {
		instCount += len(block.Insts)
	}
	logger.LogCodeGen("amd64", fn.Name, instCount)

	// Map parameters to their indices
	if err := g.mapParameters(fn); err != nil {
		return err
	}

	// Perform register allocation
	cfg := &regalloc.Config{
		Available:   []string{"%rbx", "%r12", "%r13", "%r14", "%r15"},
		Reserved:    []string{"%rax", "%rdi", "%rsi", "%rdx", "%rcx", "%r8", "%r9"},
		CalleeSaved: CalleeSaved,
		CallerSaved: CallerSaved,
	}
	g.alloc = regalloc.NewAllocator(fn, cfg)
	if err := g.alloc.Allocate(); err != nil {
		return fmt.Errorf("register allocation failed: %w", err)
	}

	// Compute stack frame size (spills + stack args)
	g.stackSize = g.alloc.GetStackSize()
	if g.stackSize > 0 {
		// Align to 16 bytes (required by System V ABI)
		g.stackSize = (g.stackSize + 15) & ^15
	}

	// Resolve phi nodes by inserting moves in predecessor blocks
	g.resolvePhi(fn)

	// Prologue
	fmt.Fprintf(g.w, "\t.globl _%s\n", fn.Name)
	fmt.Fprintf(g.w, "_%s:\n", fn.Name)
	fmt.Fprintf(g.w, "\tpushq %%rbp\n")
	fmt.Fprintf(g.w, "\tmovq %%rsp, %%rbp\n")

	// Allocate stack space if needed
	if g.stackSize > 0 {
		fmt.Fprintf(g.w, "\tsubq $%d, %%rsp\n", g.stackSize)
	}

	// Save callee-saved registers that we use
	usedCalleeSaved := g.getUsedCalleeSaved()
	for _, reg := range usedCalleeSaved {
		fmt.Fprintf(g.w, "\tpushq %s\n", reg)
	}

	// Move parameters from arg regs to allocated locations
	g.saveParameters(fn)

	// Generate blocks
	for _, block := range fn.Blocks {
		if err := g.generateBlock(block); err != nil {
			return err
		}
	}

	return nil
}

// mapParameters builds the parameter index map
func (g *Generator) mapParameters(fn *ssa.Function) error {
	for i, param := range fn.Params {
		g.paramMap[param] = i
	}
	return nil
}

// resolvePhi resolves phi nodes by inserting moves in predecessor blocks
func (g *Generator) resolvePhi(fn *ssa.Function) {
	for _, block := range fn.Blocks {
		if len(block.Phis) == 0 {
			continue
		}

		// For each phi, insert moves in predecessor blocks
		for _, phi := range block.Phis {
			for _, phiVal := range phi.Values {
				pred := phiVal.Block
				if g.phiMoves[pred] == nil {
					g.phiMoves[pred] = make([]phiMove, 0)
				}
				g.phiMoves[pred] = append(g.phiMoves[pred], phiMove{
					src:  phiVal.Value,
					dest: phi.Dest,
				})
			}
		}
	}
}

// saveParameters moves parameters from arg registers to allocated locations
func (g *Generator) saveParameters(fn *ssa.Function) {
	for i, param := range fn.Params {
		if i < len(ArgRegs) {
			// Parameter in register
			if reg, ok := g.alloc.GetRegister(param); ok {
				if reg != ArgRegs[i] {
					fmt.Fprintf(g.w, "\tmovq %s, %s\n", ArgRegs[i], reg)
				}
			} else if slot, ok := g.alloc.GetSpillSlot(param); ok {
				// Spilled parameter
				fmt.Fprintf(g.w, "\tmovq %s, -%d(%%rbp)\n", ArgRegs[i], slot)
			}
		} else {
			// Parameter on stack (from caller)
			// Stack layout: ... [arg7] [arg6] [ret addr] [saved rbp] <- rbp
			stackOffset := 16 + (i-len(ArgRegs))*8
			if reg, ok := g.alloc.GetRegister(param); ok {
				fmt.Fprintf(g.w, "\tmovq %d(%%rbp), %s\n", stackOffset, reg)
			} else if slot, ok := g.alloc.GetSpillSlot(param); ok {
				// Load from caller stack and store to our spill area
				fmt.Fprintf(g.w, "\tmovq %d(%%rbp), %%rax\n", stackOffset)
				fmt.Fprintf(g.w, "\tmovq %%rax, -%d(%%rbp)\n", slot)
			}
		}
	}
}

// getUsedCalleeSaved returns callee-saved registers that were allocated
func (g *Generator) getUsedCalleeSaved() []string {
	used := make(map[string]bool)
	// Check all intervals for callee-saved regs
	for _, block := range g.alloc.GetFunction().Blocks {
		for _, inst := range block.Insts {
			if def := getDef(inst); def != nil {
				if reg, ok := g.alloc.GetRegister(def); ok {
					for _, cs := range CalleeSaved {
						if reg == cs {
							used[cs] = true
						}
					}
				}
			}
		}
	}
	result := make([]string, 0, len(used))
	for reg := range used {
		result = append(result, reg)
	}
	return result
}

// generateBlock emits assembly for a basic block
func (g *Generator) generateBlock(block *ssa.Block) error {
	// Emit label (skip for entry block to avoid duplicate)
	if block.Label != "entry_0" {
		fmt.Fprintf(g.w, ".L%s:\n", block.Label)
	}

	// Emit instructions
	for _, inst := range block.Insts {
		if err := g.generateInst(inst); err != nil {
			return err
		}
	}

	// Emit phi resolution moves before terminator
	if moves, ok := g.phiMoves[block]; ok {
		for _, move := range moves {
			srcReg := g.getValueLocation(move.src)
			destReg := g.getValueLocation(move.dest)
			if srcReg != destReg {
				fmt.Fprintf(g.w, "\tmovq %s, %s\n", srcReg, destReg)
			}
		}
	}

	// Emit terminator
	return g.generateTerm(block.Term)
}

// generateInst emits assembly for an instruction
func (g *Generator) generateInst(inst ir.Inst) error {
	switch i := inst.(type) {
	case *ir.BinOp:
		return g.generateBinOp(i)
	case *ir.Call:
		return g.generateCall(i)
	case *ir.Load:
		return g.generateLoad(i)
	case *ir.Store:
		return g.generateStore(i)
	default:
		return fmt.Errorf("unsupported instruction: %T", inst)
	}
}

// generateBinOp emits assembly for binary operations
func (g *Generator) generateBinOp(binop *ir.BinOp) error {
	leftLoc := g.getValueLocation(binop.L)
	rightLoc := g.getValueLocation(binop.R)
	destLoc := g.getValueLocation(binop.Dest)

	switch binop.Op {
	// Arithmetic
	case ir.OpAdd:
		fmt.Fprintf(g.w, "\tmovq %s, %s\n", leftLoc, destLoc)
		fmt.Fprintf(g.w, "\taddq %s, %s\n", rightLoc, destLoc)
	case ir.OpSub:
		fmt.Fprintf(g.w, "\tmovq %s, %s\n", leftLoc, destLoc)
		fmt.Fprintf(g.w, "\tsubq %s, %s\n", rightLoc, destLoc)
	case ir.OpMul:
		fmt.Fprintf(g.w, "\tmovq %s, %s\n", leftLoc, destLoc)
		fmt.Fprintf(g.w, "\timulq %s, %s\n", rightLoc, destLoc)
	case ir.OpDiv:
		// Load left operand into rax
		if leftLoc != "%rax" {
			fmt.Fprintf(g.w, "\tmovq %s, %%rax\n", leftLoc)
		}
		fmt.Fprintf(g.w, "\tcqto\n")
		fmt.Fprintf(g.w, "\tidivq %s\n", rightLoc)
		if destLoc != "%rax" {
			fmt.Fprintf(g.w, "\tmovq %%rax, %s\n", destLoc)
		}

	// Comparisons
	case ir.OpEq:
		fmt.Fprintf(g.w, "\tcmpq %s, %s\n", rightLoc, leftLoc)
		fmt.Fprintf(g.w, "\tsete %%al\n")
		fmt.Fprintf(g.w, "\tmovzbq %%al, %s\n", destLoc)
	case ir.OpNe:
		fmt.Fprintf(g.w, "\tcmpq %s, %s\n", rightLoc, leftLoc)
		fmt.Fprintf(g.w, "\tsetne %%al\n")
		fmt.Fprintf(g.w, "\tmovzbq %%al, %s\n", destLoc)
	case ir.OpLt:
		fmt.Fprintf(g.w, "\tcmpq %s, %s\n", rightLoc, leftLoc)
		fmt.Fprintf(g.w, "\tsetl %%al\n")
		fmt.Fprintf(g.w, "\tmovzbq %%al, %s\n", destLoc)
	case ir.OpLe:
		fmt.Fprintf(g.w, "\tcmpq %s, %s\n", rightLoc, leftLoc)
		fmt.Fprintf(g.w, "\tsetle %%al\n")
		fmt.Fprintf(g.w, "\tmovzbq %%al, %s\n", destLoc)
	case ir.OpGt:
		fmt.Fprintf(g.w, "\tcmpq %s, %s\n", rightLoc, leftLoc)
		fmt.Fprintf(g.w, "\tsetg %%al\n")
		fmt.Fprintf(g.w, "\tmovzbq %%al, %s\n", destLoc)
	case ir.OpGe:
		fmt.Fprintf(g.w, "\tcmpq %s, %s\n", rightLoc, leftLoc)
		fmt.Fprintf(g.w, "\tsetge %%al\n")
		fmt.Fprintf(g.w, "\tmovzbq %%al, %s\n", destLoc)

	// Boolean operations
	case ir.OpAnd:
		fmt.Fprintf(g.w, "\tmovq %s, %s\n", leftLoc, destLoc)
		fmt.Fprintf(g.w, "\tandq %s, %s\n", rightLoc, destLoc)
	case ir.OpOr:
		fmt.Fprintf(g.w, "\tmovq %s, %s\n", leftLoc, destLoc)
		fmt.Fprintf(g.w, "\torq %s, %s\n", rightLoc, destLoc)
	case ir.OpXor:
		fmt.Fprintf(g.w, "\tmovq %s, %s\n", leftLoc, destLoc)
		fmt.Fprintf(g.w, "\txorq %s, %s\n", rightLoc, destLoc)

	default:
		return fmt.Errorf("unsupported operation: %v", binop.Op)
	}

	return nil
}

// generateCall emits assembly for function calls
func (g *Generator) generateCall(call *ir.Call) error {
	// System V ABI: up to 6 args in registers, rest on stack
	numStackArgs := 0
	if len(call.Args) > len(ArgRegs) {
		numStackArgs = len(call.Args) - len(ArgRegs)
		// Align stack to 16 bytes if needed
		if numStackArgs%2 != 0 {
			fmt.Fprintf(g.w, "\tsubq $8, %%rsp\n")
		}
	}

	// Push stack arguments in reverse order
	for i := len(call.Args) - 1; i >= len(ArgRegs); i-- {
		argLoc := g.getValueLocation(call.Args[i])
		if argLoc[0] == '$' {
			// Immediate value
			fmt.Fprintf(g.w, "\tpushq %s\n", argLoc)
		} else {
			fmt.Fprintf(g.w, "\tpushq %s\n", argLoc)
		}
	}

	// Move register arguments
	for i := 0; i < len(call.Args) && i < len(ArgRegs); i++ {
		argLoc := g.getValueLocation(call.Args[i])
		if argLoc != ArgRegs[i] {
			fmt.Fprintf(g.w, "\tmovq %s, %s\n", argLoc, ArgRegs[i])
		}
	}

	// Call function
	fmt.Fprintf(g.w, "\tcallq _%s\n", call.Function)

	// Clean up stack arguments
	if numStackArgs > 0 {
		stackBytes := numStackArgs * 8
		if numStackArgs%2 != 0 {
			stackBytes += 8 // Include alignment padding
		}
		fmt.Fprintf(g.w, "\taddq $%d, %%rsp\n", stackBytes)
	}

	// Move result to destination
	destLoc := g.getValueLocation(call.Dest)
	if destLoc != "%rax" {
		fmt.Fprintf(g.w, "\tmovq %%rax, %s\n", destLoc)
	}

	return nil
}

// generateLoad emits assembly for load instructions
func (g *Generator) generateLoad(load *ir.Load) error {
	srcLoc := g.getValueLocation(load.Src)
	destLoc := g.getValueLocation(load.Dest)
	if srcLoc != destLoc {
		fmt.Fprintf(g.w, "\tmovq %s, %s\n", srcLoc, destLoc)
	}
	return nil
}

// generateStore emits assembly for store instructions
func (g *Generator) generateStore(store *ir.Store) error {
	srcLoc := g.getValueLocation(store.Src)
	destLoc := g.getValueLocation(store.Dest)
	fmt.Fprintf(g.w, "\tmovq %s, %s\n", srcLoc, destLoc)
	return nil
}

// generateTerm emits assembly for terminator instructions
func (g *Generator) generateTerm(term ir.Terminator) error {
	switch t := term.(type) {
	case *ir.Return:
		// Move return value to rax
		if t.Value != nil {
			valLoc := g.getValueLocation(t.Value)
			if valLoc != "%rax" {
				fmt.Fprintf(g.w, "\tmovq %s, %%rax\n", valLoc)
			}
		}

		// Restore callee-saved registers
		usedCalleeSaved := g.getUsedCalleeSaved()
		for i := len(usedCalleeSaved) - 1; i >= 0; i-- {
			fmt.Fprintf(g.w, "\tpopq %s\n", usedCalleeSaved[i])
		}

		// Epilogue
		fmt.Fprintf(g.w, "\tleave\n")
		fmt.Fprintf(g.w, "\tretq\n")

	case *ir.Branch:
		fmt.Fprintf(g.w, "\tjmp .L%s\n", t.Target)

	case *ir.CondBranch:
		condLoc := g.getValueLocation(t.Cond)
		fmt.Fprintf(g.w, "\ttestq $1, %s\n", condLoc)
		fmt.Fprintf(g.w, "\tjnz .L%s\n", t.TrueBlock)
		fmt.Fprintf(g.w, "\tjmp .L%s\n", t.FalseBlock)

	default:
		return fmt.Errorf("unsupported terminator: %T", term)
	}

	return nil
}

// getValueLocation returns the register or memory location for a value
func (g *Generator) getValueLocation(val ir.Value) string {
	switch v := val.(type) {
	case *ir.Const:
		// Immediate value
		return fmt.Sprintf("$%d", v.Val)
	case *ir.Temp, *ir.Param:
		// Check if in register
		if reg, ok := g.alloc.GetRegister(val); ok {
			return reg
		}
		// Check if spilled
		if slot, ok := g.alloc.GetSpillSlot(val); ok {
			return fmt.Sprintf("-%d(%%rbp)", slot)
		}
		// Fallback - shouldn't happen
		panic(fmt.Sprintf("no location for value: %T", val))
	default:
		panic(fmt.Sprintf("unsupported value type: %T", val))
	}
}

// System V calling convention
var (
	// Argument registers (order matters)
	ArgRegs = []string{"%rdi", "%rsi", "%rdx", "%rcx", "%r8", "%r9"}
	// Return register
	RetReg = "%rax"
	// Caller-saved
	CallerSaved = []string{"%rax", "%rcx", "%rdx", "%rsi", "%rdi", "%r8", "%r9", "%r10", "%r11"}
	// Callee-saved
	CalleeSaved = []string{"%rbx", "%r12", "%r13", "%r14", "%r15"}
)

// Helper to get definition from instruction (for callee-saved check)
func getDef(inst ir.Inst) ir.Value {
	switch i := inst.(type) {
	case *ir.BinOp:
		return i.Dest
	case *ir.Call:
		return i.Dest
	case *ir.Load:
		return i.Dest
	case *ir.Alloc:
		return i.Dest
	case *ir.AllocObject:
		return i.Dest
	case *ir.GetAttr:
		return i.Dest
	case *ir.GetItem:
		return i.Dest
	case *ir.MethodCall:
		return i.Dest
	case *ir.ClosureCall:
		return i.Dest
	case *ir.MakeClosure:
		return i.Dest
	}
	return nil
}
