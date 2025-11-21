// Package arm64 implements ARM64/AArch64 code generation.
//
// Design: Direct assembly generation for Apple Silicon and ARM servers.
// ARM64 calling convention (AAPCS64).
package arm64

import (
	"fmt"
	"io"

	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/ssa"
)

// Generator generates ARM64 assembly
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
	fmt.Fprintf(g.w, "\t.align 2\n")

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

	// Prologue
	fmt.Fprintf(g.w, "\t.global _%s\n", fn.Name)
	fmt.Fprintf(g.w, "_%s:\n", fn.Name)

	// ARM64 prologue: save frame pointer and link register
	fmt.Fprintf(g.w, "\tstp x29, x30, [sp, #-16]!\n")
	fmt.Fprintf(g.w, "\tmov x29, sp\n")

	// Generate blocks
	for _, block := range fn.Blocks {
		if err := g.generateBlock(block); err != nil {
			return err
		}
	}

	return nil
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

	switch binop.Op {
	case ir.OpAdd:
		fmt.Fprintf(g.w, "\tadd %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpSub:
		fmt.Fprintf(g.w, "\tsub %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpMul:
		fmt.Fprintf(g.w, "\tmul %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpDiv:
		fmt.Fprintf(g.w, "\tsdiv %s, %s, %s\n", destReg, leftReg, rightReg)
	default:
		return fmt.Errorf("unsupported operation: %v", binop.Op)
	}

	return nil
}

// generateCall emits assembly for function calls
func (g *Generator) generateCall(call *ir.Call) error {
	// Move arguments to registers (AAPCS64: x0-x7 for args)
	for i, arg := range call.Args {
		if i >= len(ArgRegs) {
			return fmt.Errorf("too many arguments")
		}
		argReg := g.valueReg(arg)
		if argReg != ArgRegs[i] {
			fmt.Fprintf(g.w, "\tmov %s, %s\n", ArgRegs[i], argReg)
		}
	}

	// Call function
	fmt.Fprintf(g.w, "\tbl _%s\n", call.Function)

	// Move result to destination
	destReg := g.allocReg(call.Dest)
	if destReg != "x0" {
		fmt.Fprintf(g.w, "\tmov %s, x0\n", destReg)
	}

	return nil
}

// generateLoad emits assembly for load instructions
func (g *Generator) generateLoad(load *ir.Load) error {
	srcReg := g.valueReg(load.Src)
	destReg := g.allocReg(load.Dest)
	fmt.Fprintf(g.w, "\tmov %s, %s\n", destReg, srcReg)
	return nil
}

// generateStore emits assembly for store instructions
func (g *Generator) generateStore(store *ir.Store) error {
	srcReg := g.valueReg(store.Src)
	destReg := g.valueReg(store.Dest)
	fmt.Fprintf(g.w, "\tmov %s, %s\n", destReg, srcReg)
	return nil
}

// generateTerm emits assembly for terminator instructions
func (g *Generator) generateTerm(term ir.Terminator) error {
	switch t := term.(type) {
	case *ir.Return:
		// Move return value to x0
		if t.Value != nil {
			valReg := g.valueReg(t.Value)
			if valReg != "x0" {
				fmt.Fprintf(g.w, "\tmov x0, %s\n", valReg)
			}
		}

		// Epilogue: restore frame pointer and link register, return
		fmt.Fprintf(g.w, "\tldp x29, x30, [sp], #16\n")
		fmt.Fprintf(g.w, "\tret\n")

	case *ir.Branch:
		fmt.Fprintf(g.w, "\tb .L%s\n", t.Target)

	case *ir.CondBranch:
		condReg := g.valueReg(t.Cond)
		fmt.Fprintf(g.w, "\tcmp %s, #0\n", condReg)
		fmt.Fprintf(g.w, "\tb.ne .L%s\n", t.TrueBlock)
		fmt.Fprintf(g.w, "\tb .L%s\n", t.FalseBlock)

	default:
		return fmt.Errorf("unsupported terminator: %T", term)
	}

	return nil
}

// valueReg returns the register or immediate for a value
func (g *Generator) valueReg(val ir.Value) string {
	switch v := val.(type) {
	case *ir.Const:
		// Load immediate into a register
		reg := g.allocReg(v)
		fmt.Fprintf(g.w, "\tmov %s, #%d\n", reg, v.Val)
		return reg
	case *ir.Temp:
		if reg, ok := g.valRegs[v]; ok {
			return reg
		}
		return g.allocReg(v)
	case *ir.Param:
		if reg, ok := g.valRegs[v]; ok {
			return reg
		}
		return g.allocReg(v)
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

// RegAlloc implements simple register allocation
type RegAlloc struct {
	available []string
	next      int
}

func NewRegAlloc() *RegAlloc {
	return &RegAlloc{
		// Callee-saved registers: x19-x28
		// We'll use a subset for simplicity
		available: []string{"x19", "x20", "x21", "x22", "x23", "x24", "x25", "x26"},
	}
}

func (r *RegAlloc) Alloc() string {
	reg := r.available[r.next%len(r.available)]
	r.next++
	return reg
}

func (r *RegAlloc) Reset() {
	r.next = 0
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
