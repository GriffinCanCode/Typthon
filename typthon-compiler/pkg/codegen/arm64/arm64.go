// Package arm64 implements ARM64/AArch64 code generation.
//
// Design: Direct assembly generation for Apple Silicon and ARM servers.
// ARM64 calling convention (AAPCS64).
package arm64

import (
	"fmt"
	"io"
	"strings"

	"github.com/GriffinCanCode/typthon-compiler/pkg/codegen/regalloc"
	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
	"github.com/GriffinCanCode/typthon-compiler/pkg/ssa"
)

// Generator generates ARM64 assembly
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
	logger.Debug("Generating arm64 assembly", "functions", len(prog.Functions))

	// Emit assembly header
	fmt.Fprintf(g.w, "\t.text\n")
	fmt.Fprintf(g.w, "\t.align 2\n")

	for _, fn := range prog.Functions {
		logger.Debug("Generating function assembly", "arch", "arm64", "name", fn.Name)
		if err := g.generateFunction(fn); err != nil {
			logger.Error("Failed to generate function", "arch", "arm64", "name", fn.Name, "error", err)
			return err
		}
	}

	logger.Info("arm64 code generation complete", "functions", len(prog.Functions))
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
	logger.LogCodeGen("arm64", fn.Name, instCount)

	// Map parameters to their indices
	if err := g.mapParameters(fn); err != nil {
		return err
	}

	// Perform register allocation
	cfg := &regalloc.Config{
		Available:   []string{"x19", "x20", "x21", "x22", "x23", "x24", "x25", "x26", "x27", "x28"},
		Reserved:    []string{"x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7", "x29", "x30"},
		CalleeSaved: []string{"x19", "x20", "x21", "x22", "x23", "x24", "x25", "x26", "x27", "x28"},
		CallerSaved: []string{"x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7", "x9", "x10", "x11", "x12", "x13", "x14", "x15", "x16", "x17"},
	}
	g.alloc = regalloc.NewAllocator(fn, cfg)
	if err := g.alloc.Allocate(); err != nil {
		return fmt.Errorf("register allocation failed: %w", err)
	}

	// Compute stack frame size (spills + stack args)
	g.stackSize = g.alloc.GetStackSize()
	if g.stackSize > 0 {
		// Align to 16 bytes (required by AAPCS64)
		g.stackSize = (g.stackSize + 15) & ^15
	}

	// Resolve phi nodes by inserting moves in predecessor blocks
	g.resolvePhi(fn)

	// Prologue
	fmt.Fprintf(g.w, "\t.global _%s\n", fn.Name)
	fmt.Fprintf(g.w, "_%s:\n", fn.Name)

	// ARM64 prologue: save frame pointer and link register
	frameSize := g.stackSize + 16 // 16 for fp + lr
	if frameSize > 0 {
		fmt.Fprintf(g.w, "\tstp x29, x30, [sp, #-%d]!\n", frameSize)
		fmt.Fprintf(g.w, "\tmov x29, sp\n")
	} else {
		fmt.Fprintf(g.w, "\tstp x29, x30, [sp, #-16]!\n")
		fmt.Fprintf(g.w, "\tmov x29, sp\n")
	}

	// Save callee-saved registers that we use
	usedCalleeSaved := g.getUsedCalleeSaved()
	offset := 16
	for i := 0; i < len(usedCalleeSaved); i += 2 {
		if i+1 < len(usedCalleeSaved) {
			fmt.Fprintf(g.w, "\tstp %s, %s, [sp, #%d]\n", usedCalleeSaved[i], usedCalleeSaved[i+1], offset)
			offset += 16
		} else {
			fmt.Fprintf(g.w, "\tstr %s, [sp, #%d]\n", usedCalleeSaved[i], offset)
			offset += 8
		}
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
					fmt.Fprintf(g.w, "\tmov %s, %s\n", reg, ArgRegs[i])
				}
			} else if slot, ok := g.alloc.GetSpillSlot(param); ok {
				// Spilled parameter
				fmt.Fprintf(g.w, "\tstr %s, [x29, #-%d]\n", ArgRegs[i], slot)
			}
		} else {
			// Parameter on stack (from caller)
			// Stack layout: ... [arg8] [arg9] ... [ret addr stored by bl]
			// Our frame: [saved fp][saved lr][...our locals...]
			stackOffset := g.stackSize + 16 + (i-len(ArgRegs))*8
			if reg, ok := g.alloc.GetRegister(param); ok {
				fmt.Fprintf(g.w, "\tldr %s, [x29, #%d]\n", reg, stackOffset)
			} else if slot, ok := g.alloc.GetSpillSlot(param); ok {
				// Load from caller stack and store to our spill area
				fmt.Fprintf(g.w, "\tldr x9, [x29, #%d]\n", stackOffset)
				fmt.Fprintf(g.w, "\tstr x9, [x29, #-%d]\n", slot)
			}
		}
	}
}

// getUsedCalleeSaved returns callee-saved registers that were allocated
func (g *Generator) getUsedCalleeSaved() []string {
	used := make(map[string]bool)
	calleeSaved := map[string]bool{
		"x19": true, "x20": true, "x21": true, "x22": true,
		"x23": true, "x24": true, "x25": true, "x26": true,
		"x27": true, "x28": true,
	}

	// Check all intervals for callee-saved regs
	for _, block := range g.alloc.GetFunction().Blocks {
		for _, inst := range block.Insts {
			if def := getDef(inst); def != nil {
				if reg, ok := g.alloc.GetRegister(def); ok {
					if calleeSaved[reg] {
						used[reg] = true
					}
				}
			}
		}
	}

	result := make([]string, 0, len(used))
	// Return in order
	for _, reg := range []string{"x19", "x20", "x21", "x22", "x23", "x24", "x25", "x26", "x27", "x28"} {
		if used[reg] {
			result = append(result, reg)
		}
	}
	return result
}

// generateBlock emits assembly for a basic block
func (g *Generator) generateBlock(block *ssa.Block) error {
	// Emit label (skip for entry block)
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
			srcLoc := g.getValueLocation(move.src)
			destLoc := g.getValueLocation(move.dest)
			// ARM64 doesn't support memory-to-memory moves
			if srcLoc[0] == '[' && destLoc[0] == '[' {
				// Load to temp register first
				fmt.Fprintf(g.w, "\tldr x9, %s\n", srcLoc)
				fmt.Fprintf(g.w, "\tstr x9, %s\n", destLoc)
			} else if srcLoc != destLoc {
				if srcLoc[0] == '[' {
					fmt.Fprintf(g.w, "\tldr %s, %s\n", destLoc, srcLoc)
				} else if destLoc[0] == '[' {
					fmt.Fprintf(g.w, "\tstr %s, %s\n", srcLoc, destLoc)
				} else {
					fmt.Fprintf(g.w, "\tmov %s, %s\n", destLoc, srcLoc)
				}
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

	// Load operands from memory if needed
	leftReg := g.ensureInRegister(leftLoc, "x9")
	rightReg := g.ensureInRegister(rightLoc, "x10")
	destReg := destLoc
	if destLoc[0] == '[' || destLoc[0] == '#' {
		destReg = "x11"
	}

	switch binop.Op {
	case ir.OpAdd:
		fmt.Fprintf(g.w, "\tadd %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpSub:
		fmt.Fprintf(g.w, "\tsub %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpMul:
		fmt.Fprintf(g.w, "\tmul %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpDiv:
		fmt.Fprintf(g.w, "\tsdiv %s, %s, %s\n", destReg, leftReg, rightReg)

	// Comparisons
	case ir.OpEq:
		fmt.Fprintf(g.w, "\tcmp %s, %s\n", leftReg, rightReg)
		fmt.Fprintf(g.w, "\tcset %s, eq\n", destReg)
	case ir.OpNe:
		fmt.Fprintf(g.w, "\tcmp %s, %s\n", leftReg, rightReg)
		fmt.Fprintf(g.w, "\tcset %s, ne\n", destReg)
	case ir.OpLt:
		fmt.Fprintf(g.w, "\tcmp %s, %s\n", leftReg, rightReg)
		fmt.Fprintf(g.w, "\tcset %s, lt\n", destReg)
	case ir.OpLe:
		fmt.Fprintf(g.w, "\tcmp %s, %s\n", leftReg, rightReg)
		fmt.Fprintf(g.w, "\tcset %s, le\n", destReg)
	case ir.OpGt:
		fmt.Fprintf(g.w, "\tcmp %s, %s\n", leftReg, rightReg)
		fmt.Fprintf(g.w, "\tcset %s, gt\n", destReg)
	case ir.OpGe:
		fmt.Fprintf(g.w, "\tcmp %s, %s\n", leftReg, rightReg)
		fmt.Fprintf(g.w, "\tcset %s, ge\n", destReg)

	// Boolean operations
	case ir.OpAnd:
		fmt.Fprintf(g.w, "\tand %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpOr:
		fmt.Fprintf(g.w, "\torr %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpXor:
		fmt.Fprintf(g.w, "\teor %s, %s, %s\n", destReg, leftReg, rightReg)

	default:
		return fmt.Errorf("unsupported operation: %v", binop.Op)
	}

	// Store result if destination is memory
	if destLoc[0] == '[' {
		fmt.Fprintf(g.w, "\tstr %s, %s\n", destReg, destLoc)
	}

	return nil
}

// ensureInRegister loads a value into a register if it's not already
func (g *Generator) ensureInRegister(loc string, tempReg string) string {
	if loc[0] == '[' {
		// Memory location - load it
		fmt.Fprintf(g.w, "\tldr %s, %s\n", tempReg, loc)
		return tempReg
	} else if loc[0] == '#' {
		// Immediate - move it
		fmt.Fprintf(g.w, "\tmov %s, %s\n", tempReg, loc)
		return tempReg
	}
	return loc
}

// generateCall emits assembly for function calls
func (g *Generator) generateCall(call *ir.Call) error {
	// AAPCS64: up to 8 args in registers, rest on stack
	numStackArgs := 0
	if len(call.Args) > len(ArgRegs) {
		numStackArgs = len(call.Args) - len(ArgRegs)
		// Align to 16 bytes
		stackBytes := (numStackArgs*8 + 15) & ^15
		if stackBytes > 0 {
			fmt.Fprintf(g.w, "\tsub sp, sp, #%d\n", stackBytes)
		}
	}

	// Store stack arguments
	for i := len(ArgRegs); i < len(call.Args); i++ {
		argLoc := g.getValueLocation(call.Args[i])
		offset := (i - len(ArgRegs)) * 8
		argReg := g.ensureInRegister(argLoc, "x9")
		fmt.Fprintf(g.w, "\tstr %s, [sp, #%d]\n", argReg, offset)
	}

	// Move register arguments
	for i := 0; i < len(call.Args) && i < len(ArgRegs); i++ {
		argLoc := g.getValueLocation(call.Args[i])
		if argLoc != ArgRegs[i] {
			argReg := g.ensureInRegister(argLoc, ArgRegs[i])
			if argReg != ArgRegs[i] {
				fmt.Fprintf(g.w, "\tmov %s, %s\n", ArgRegs[i], argReg)
			}
		}
	}

	// Call function
	fmt.Fprintf(g.w, "\tbl _%s\n", call.Function)

	// Clean up stack arguments
	if numStackArgs > 0 {
		stackBytes := (numStackArgs*8 + 15) & ^15
		if stackBytes > 0 {
			fmt.Fprintf(g.w, "\tadd sp, sp, #%d\n", stackBytes)
		}
	}

	// Move result to destination
	destLoc := g.getValueLocation(call.Dest)
	if destLoc != "x0" {
		if destLoc[0] == '[' {
			fmt.Fprintf(g.w, "\tstr x0, %s\n", destLoc)
		} else {
			fmt.Fprintf(g.w, "\tmov %s, x0\n", destLoc)
		}
	}

	return nil
}

// generateLoad emits assembly for load instructions
func (g *Generator) generateLoad(load *ir.Load) error {
	srcLoc := g.getValueLocation(load.Src)
	destLoc := g.getValueLocation(load.Dest)
	if srcLoc != destLoc {
		if srcLoc[0] == '[' && destLoc[0] == '[' {
			// Memory to memory - use temp
			fmt.Fprintf(g.w, "\tldr x9, %s\n", srcLoc)
			fmt.Fprintf(g.w, "\tstr x9, %s\n", destLoc)
		} else if srcLoc[0] == '[' {
			fmt.Fprintf(g.w, "\tldr %s, %s\n", destLoc, srcLoc)
		} else if destLoc[0] == '[' {
			srcReg := g.ensureInRegister(srcLoc, "x9")
			fmt.Fprintf(g.w, "\tstr %s, %s\n", srcReg, destLoc)
		} else {
			fmt.Fprintf(g.w, "\tmov %s, %s\n", destLoc, srcLoc)
		}
	}
	return nil
}

// generateStore emits assembly for store instructions
func (g *Generator) generateStore(store *ir.Store) error {
	srcLoc := g.getValueLocation(store.Src)
	destLoc := g.getValueLocation(store.Dest)

	if srcLoc[0] == '[' && destLoc[0] == '[' {
		// Memory to memory - use temp
		fmt.Fprintf(g.w, "\tldr x9, %s\n", srcLoc)
		fmt.Fprintf(g.w, "\tstr x9, %s\n", destLoc)
	} else if srcLoc[0] == '[' {
		fmt.Fprintf(g.w, "\tldr x9, %s\n", srcLoc)
		fmt.Fprintf(g.w, "\tstr x9, %s\n", destLoc)
	} else if destLoc[0] == '[' {
		srcReg := g.ensureInRegister(srcLoc, "x9")
		fmt.Fprintf(g.w, "\tstr %s, %s\n", srcReg, destLoc)
	} else {
		srcReg := g.ensureInRegister(srcLoc, "x9")
		fmt.Fprintf(g.w, "\tmov %s, %s\n", destLoc, srcReg)
	}

	return nil
}

// generateTerm emits assembly for terminator instructions
func (g *Generator) generateTerm(term ir.Terminator) error {
	switch t := term.(type) {
	case *ir.Return:
		// Move return value to x0
		if t.Value != nil {
			valLoc := g.getValueLocation(t.Value)
			if valLoc != "x0" {
				valReg := g.ensureInRegister(valLoc, "x0")
				if valReg != "x0" {
					fmt.Fprintf(g.w, "\tmov x0, %s\n", valReg)
				}
			}
		}

		// Restore callee-saved registers
		usedCalleeSaved := g.getUsedCalleeSaved()
		offset := 16
		for i := 0; i < len(usedCalleeSaved); i += 2 {
			if i+1 < len(usedCalleeSaved) {
				fmt.Fprintf(g.w, "\tldp %s, %s, [sp, #%d]\n", usedCalleeSaved[i], usedCalleeSaved[i+1], offset)
				offset += 16
			} else {
				fmt.Fprintf(g.w, "\tldr %s, [sp, #%d]\n", usedCalleeSaved[i], offset)
				offset += 8
			}
		}

		// Epilogue: restore frame pointer and link register, return
		frameSize := g.stackSize + 16
		if frameSize > 0 {
			fmt.Fprintf(g.w, "\tldp x29, x30, [sp], #%d\n", frameSize)
		} else {
			fmt.Fprintf(g.w, "\tldp x29, x30, [sp], #16\n")
		}
		fmt.Fprintf(g.w, "\tret\n")

	case *ir.Branch:
		fmt.Fprintf(g.w, "\tb .L%s\n", t.Target)

	case *ir.CondBranch:
		condLoc := g.getValueLocation(t.Cond)
		condReg := g.ensureInRegister(condLoc, "x9")
		fmt.Fprintf(g.w, "\ttst %s, #1\n", condReg)
		fmt.Fprintf(g.w, "\tb.ne .L%s\n", t.TrueBlock)
		fmt.Fprintf(g.w, "\tb .L%s\n", t.FalseBlock)

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
		return fmt.Sprintf("#%d", v.Val)
	case *ir.Temp, *ir.Param:
		// Check if in register
		if reg, ok := g.alloc.GetRegister(val); ok {
			return reg
		}
		// Check if spilled
		if slot, ok := g.alloc.GetSpillSlot(val); ok {
			return fmt.Sprintf("[x29, #-%d]", slot)
		}
		// Fallback - shouldn't happen
		panic(fmt.Sprintf("no location for value: %T", val))
	default:
		panic(fmt.Sprintf("unsupported value type: %T", val))
	}
}

// ARM64 calling convention (AAPCS64)
var (
	// Argument registers
	ArgRegs = []string{"x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7"}
	// Return register
	RetReg = "x0"
	// Frame pointer
	FramePointer = "x29"
	// Link register
	LinkReg = "x30"
	// Stack pointer
	StackPointer = "sp"
)

// Helper to get definition from instruction
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
