// Package amd64 implements x86-64 code generation.
//
// Design: Direct assembly generation, no LLVM dependencies.
// System V calling convention for Unix/macOS.
// Fast compilation over perfect code - optimize for compile speed.
package amd64

import (
	"fmt"
	"io"

	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
	"github.com/GriffinCanCode/typthon-compiler/pkg/ssa"
)

// Generator generates x86-64 assembly
type Generator struct {
	w        io.Writer
	regs     *RegAlloc
	valRegs  map[ir.Value]string
	stackOff int
}

func NewGenerator(w io.Writer) *Generator {
	return &Generator{
		w:       w,
		regs:    NewRegAlloc(),
		valRegs: make(map[ir.Value]string),
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

// generateFunction emits assembly for a single function
func (g *Generator) generateFunction(fn *ssa.Function) error {
	g.regs.Reset()
	g.valRegs = make(map[ir.Value]string)
	g.stackOff = 0

	instCount := 0
	for _, block := range fn.Blocks {
		instCount += len(block.Insts)
	}
	logger.LogCodeGen("amd64", fn.Name, instCount)

	// Map parameters to their argument registers
	g.mapParameters(fn)

	// Prologue
	fmt.Fprintf(g.w, "\t.globl _%s\n", fn.Name)
	fmt.Fprintf(g.w, "_%s:\n", fn.Name)
	fmt.Fprintf(g.w, "\tpushq %%rbp\n")
	fmt.Fprintf(g.w, "\tmovq %%rsp, %%rbp\n")

	// Generate blocks
	for _, block := range fn.Blocks {
		if err := g.generateBlock(block); err != nil {
			return err
		}
	}

	return nil
}

// mapParameters assigns parameters to their calling convention registers
func (g *Generator) mapParameters(fn *ssa.Function) {
	// Get function parameters from first block's context
	// In SSA, we need to get params from the IR function
	// For now, we'll look them up dynamically in valueReg
}

// generateBlock emits assembly for a basic block
func (g *Generator) generateBlock(block *ssa.Block) error {
	// Emit label (skip for entry block to avoid duplicate)
	if block.Label != "entry_0" {
		fmt.Fprintf(g.w, ".L%s:\n", block.Label)
	}

	// Emit phi nodes for SSA merge points
	for _, phi := range block.Phis {
		if err := g.generatePhi(phi, block); err != nil {
			return err
		}
	}

	// Emit instructions
	for _, inst := range block.Insts {
		if err := g.generateInst(inst); err != nil {
			return err
		}
	}

	// Emit terminator
	return g.generateTerm(block.Term)
}

// generatePhi emits assembly for phi nodes
// Phi nodes are resolved by placing moves at predecessor block ends
func (g *Generator) generatePhi(phi *ssa.Phi, block *ssa.Block) error {
	// Allocate destination register for phi result
	destReg := g.allocReg(phi.Dest)

	// For each predecessor, we need to insert moves at the end
	// of the predecessor block (before the terminator)
	// In a proper implementation, this would be done in a separate pass
	// For now, we just allocate the destination register and rely on
	// the IR builder to have already resolved phi nodes for simple cases

	logger.Debug("Processing phi node", "dest", destReg, "values", len(phi.Values))

	// If only one value (degenerate phi), just do a simple move
	if len(phi.Values) == 1 {
		srcReg := g.valueReg(phi.Values[0].Value)
		fmt.Fprintf(g.w, "\tmovq %s, %s\n", srcReg, destReg)
	}

	return nil
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
	leftReg := g.valueReg(binop.L)
	rightReg := g.valueReg(binop.R)
	destReg := g.allocReg(binop.Dest)

	switch binop.Op {
	// Arithmetic
	case ir.OpAdd:
		fmt.Fprintf(g.w, "\tmovq %s, %s\n", leftReg, destReg)
		fmt.Fprintf(g.w, "\taddq %s, %s\n", rightReg, destReg)
	case ir.OpSub:
		fmt.Fprintf(g.w, "\tmovq %s, %s\n", leftReg, destReg)
		fmt.Fprintf(g.w, "\tsubq %s, %s\n", rightReg, destReg)
	case ir.OpMul:
		fmt.Fprintf(g.w, "\tmovq %s, %s\n", leftReg, destReg)
		fmt.Fprintf(g.w, "\timulq %s, %s\n", rightReg, destReg)
	case ir.OpDiv:
		fmt.Fprintf(g.w, "\tmovq %s, %%rax\n", leftReg)
		fmt.Fprintf(g.w, "\tcqto\n")
		fmt.Fprintf(g.w, "\tidivq %s\n", rightReg)
		fmt.Fprintf(g.w, "\tmovq %%rax, %s\n", destReg)

	// Comparisons
	case ir.OpEq:
		fmt.Fprintf(g.w, "\tcmpq %s, %s\n", rightReg, leftReg)
		fmt.Fprintf(g.w, "\tsete %%al\n")
		fmt.Fprintf(g.w, "\tmovzbq %%al, %s\n", destReg)
	case ir.OpNe:
		fmt.Fprintf(g.w, "\tcmpq %s, %s\n", rightReg, leftReg)
		fmt.Fprintf(g.w, "\tsetne %%al\n")
		fmt.Fprintf(g.w, "\tmovzbq %%al, %s\n", destReg)
	case ir.OpLt:
		fmt.Fprintf(g.w, "\tcmpq %s, %s\n", rightReg, leftReg)
		fmt.Fprintf(g.w, "\tsetl %%al\n")
		fmt.Fprintf(g.w, "\tmovzbq %%al, %s\n", destReg)
	case ir.OpLe:
		fmt.Fprintf(g.w, "\tcmpq %s, %s\n", rightReg, leftReg)
		fmt.Fprintf(g.w, "\tsetle %%al\n")
		fmt.Fprintf(g.w, "\tmovzbq %%al, %s\n", destReg)
	case ir.OpGt:
		fmt.Fprintf(g.w, "\tcmpq %s, %s\n", rightReg, leftReg)
		fmt.Fprintf(g.w, "\tsetg %%al\n")
		fmt.Fprintf(g.w, "\tmovzbq %%al, %s\n", destReg)
	case ir.OpGe:
		fmt.Fprintf(g.w, "\tcmpq %s, %s\n", rightReg, leftReg)
		fmt.Fprintf(g.w, "\tsetge %%al\n")
		fmt.Fprintf(g.w, "\tmovzbq %%al, %s\n", destReg)

	// Boolean operations
	case ir.OpAnd:
		fmt.Fprintf(g.w, "\tmovq %s, %s\n", leftReg, destReg)
		fmt.Fprintf(g.w, "\tandq %s, %s\n", rightReg, destReg)
	case ir.OpOr:
		fmt.Fprintf(g.w, "\tmovq %s, %s\n", leftReg, destReg)
		fmt.Fprintf(g.w, "\torq %s, %s\n", rightReg, destReg)
	case ir.OpXor:
		fmt.Fprintf(g.w, "\tmovq %s, %s\n", leftReg, destReg)
		fmt.Fprintf(g.w, "\txorq %s, %s\n", rightReg, destReg)

	default:
		return fmt.Errorf("unsupported operation: %v", binop.Op)
	}

	return nil
}

// generateCall emits assembly for function calls
func (g *Generator) generateCall(call *ir.Call) error {
	// Move arguments to registers (System V ABI)
	for i, arg := range call.Args {
		if i >= len(ArgRegs) {
			return fmt.Errorf("too many arguments (stack args not yet supported)")
		}
		argReg := g.valueReg(arg)
		fmt.Fprintf(g.w, "\tmovq %s, %s\n", argReg, ArgRegs[i])
	}

	// Call function
	fmt.Fprintf(g.w, "\tcallq _%s\n", call.Function)

	// Move result to destination
	destReg := g.allocReg(call.Dest)
	fmt.Fprintf(g.w, "\tmovq %%rax, %s\n", destReg)

	return nil
}

// generateLoad emits assembly for load instructions
func (g *Generator) generateLoad(load *ir.Load) error {
	srcReg := g.valueReg(load.Src)
	destReg := g.allocReg(load.Dest)
	fmt.Fprintf(g.w, "\tmovq %s, %s\n", srcReg, destReg)
	return nil
}

// generateStore emits assembly for store instructions
func (g *Generator) generateStore(store *ir.Store) error {
	srcReg := g.valueReg(store.Src)
	destReg := g.valueReg(store.Dest)
	fmt.Fprintf(g.w, "\tmovq %s, %s\n", srcReg, destReg)
	return nil
}

// generateTerm emits assembly for terminator instructions
func (g *Generator) generateTerm(term ir.Terminator) error {
	switch t := term.(type) {
	case *ir.Return:
		// Move return value to rax
		if t.Value != nil {
			valReg := g.valueReg(t.Value)
			fmt.Fprintf(g.w, "\tmovq %s, %%rax\n", valReg)
		}

		// Epilogue
		fmt.Fprintf(g.w, "\tpopq %%rbp\n")
		fmt.Fprintf(g.w, "\tretq\n")

	case *ir.Branch:
		fmt.Fprintf(g.w, "\tjmp .L%s\n", t.Target)

	case *ir.CondBranch:
		condReg := g.valueReg(t.Cond)
		fmt.Fprintf(g.w, "\ttestq $1, %s\n", condReg)
		fmt.Fprintf(g.w, "\tjnz .L%s\n", t.TrueBlock)
		fmt.Fprintf(g.w, "\tjmp .L%s\n", t.FalseBlock)

	default:
		return fmt.Errorf("unsupported terminator: %T", term)
	}

	return nil
}

// valueReg returns the register or immediate for a value
func (g *Generator) valueReg(val ir.Value) string {
	switch v := val.(type) {
	case *ir.Const:
		// For x86-64, we can use immediate values directly in most instructions
		return fmt.Sprintf("$%d", v.Val)
	case *ir.Temp:
		if reg, ok := g.valRegs[v]; ok {
			return reg
		}
		// Allocate new register
		return g.allocReg(v)
	case *ir.Param:
		// Parameters come in via argument registers (System V ABI)
		// We need to track which parameter this is
		// For Phase 1 simplicity, we'll return the arg register directly
		// This works because we're not modifying parameters
		if reg, ok := g.valRegs[v]; ok {
			return reg
		}
		// Use the parameter's position to determine register
		// For now, allocate to a callee-saved register
		reg := g.allocReg(v)
		// In a real implementation, we'd move from ArgRegs[paramIndex] to reg
		return reg
	default:
		panic(fmt.Sprintf("unsupported value type: %T", val))
	}
}

// allocReg allocates a register for a value
func (g *Generator) allocReg(val ir.Value) string {
	if reg, ok := g.valRegs[val]; ok {
		return reg
	}

	reg := g.regs.Alloc()
	g.valRegs[val] = reg
	return reg
}

// RegAlloc implements linear scan register allocation
type RegAlloc struct {
	available []string
	used      map[string]bool
	next      int
}

func NewRegAlloc() *RegAlloc {
	return &RegAlloc{
		// System V ABI registers (callee-saved)
		available: []string{"%rbx", "%r12", "%r13", "%r14", "%r15"},
		used:      make(map[string]bool),
	}
}

func (r *RegAlloc) Alloc() string {
	// Simple round-robin for now
	reg := r.available[r.next%len(r.available)]
	r.next++
	return reg
}

func (r *RegAlloc) Free(reg string) {
	delete(r.used, reg)
}

func (r *RegAlloc) Reset() {
	r.used = make(map[string]bool)
	r.next = 0
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
