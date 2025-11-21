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
	// Emit assembly header
	fmt.Fprintf(g.w, "\t.text\n")

	for _, fn := range prog.Functions {
		if err := g.generateFunction(fn); err != nil {
			return err
		}
	}

	return nil
}

// generateFunction emits assembly for a single function
func (g *Generator) generateFunction(fn *ssa.Function) error {
	g.regs.Reset()
	g.valRegs = make(map[ir.Value]string)
	g.stackOff = 0

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

	// Emit phi nodes (none in Phase 1)
	for _, phi := range block.Phis {
		_ = phi // TODO: implement when we have control flow
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

	// Move left operand to destination
	fmt.Fprintf(g.w, "\tmovq %s, %s\n", leftReg, destReg)

	// Perform operation
	switch binop.Op {
	case ir.OpAdd:
		fmt.Fprintf(g.w, "\taddq %s, %s\n", rightReg, destReg)
	case ir.OpSub:
		fmt.Fprintf(g.w, "\tsubq %s, %s\n", rightReg, destReg)
	case ir.OpMul:
		fmt.Fprintf(g.w, "\timulq %s, %s\n", rightReg, destReg)
	case ir.OpDiv:
		// Division is more complex (uses rax/rdx)
		fmt.Fprintf(g.w, "\tmovq %s, %%rax\n", leftReg)
		fmt.Fprintf(g.w, "\tcqto\n") // Sign-extend rax into rdx
		fmt.Fprintf(g.w, "\tidivq %s\n", rightReg)
		fmt.Fprintf(g.w, "\tmovq %%rax, %s\n", destReg)
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
		fmt.Fprintf(g.w, "\ttestq %s, %s\n", condReg, condReg)
		fmt.Fprintf(g.w, "\tjne .L%s\n", t.TrueBlock)
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
