// Package riscv64 implements RISC-V 64-bit code generation.
//
// Design: Future-proofing for RISC-V servers and embedded systems.
// RISC-V RV64I base instruction set with standard calling convention.
package riscv64

import (
	"fmt"
	"io"

	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
	"github.com/GriffinCanCode/typthon-compiler/pkg/ssa"
)

// Generator generates RISC-V 64-bit assembly
type Generator struct {
	w        io.Writer
	regs     *RegAlloc
	valRegs  map[ir.Value]string
	paramMap map[*ir.Param]int
	stackOff int
}

func NewGenerator(w io.Writer) *Generator {
	return &Generator{
		w:        w,
		regs:     NewRegAlloc(),
		valRegs:  make(map[ir.Value]string),
		paramMap: make(map[*ir.Param]int),
	}
}

// Generate emits assembly for an SSA program
func (g *Generator) Generate(prog *ssa.Program) error {
	logger.Debug("Generating riscv64 assembly", "functions", len(prog.Functions))

	// Emit assembly header
	fmt.Fprintf(g.w, "\t.text\n")

	for _, fn := range prog.Functions {
		logger.Debug("Generating function assembly", "arch", "riscv64", "name", fn.Name)
		if err := g.generateFunction(fn); err != nil {
			logger.Error("Failed to generate function", "arch", "riscv64", "name", fn.Name, "error", err)
			return err
		}
	}

	logger.Info("riscv64 code generation complete", "functions", len(prog.Functions))
	return nil
}

// generateFunction emits assembly for a single function
func (g *Generator) generateFunction(fn *ssa.Function) error {
	g.regs.Reset()
	g.valRegs = make(map[ir.Value]string)
	g.paramMap = make(map[*ir.Param]int)
	g.stackOff = 0

	instCount := 0
	for _, block := range fn.Blocks {
		instCount += len(block.Insts)
	}
	logger.LogCodeGen("riscv64", fn.Name, instCount)

	// Map parameters to their argument registers
	if err := g.mapParameters(fn); err != nil {
		return err
	}

	// Prologue
	fmt.Fprintf(g.w, "\t.globl %s\n", fn.Name)
	fmt.Fprintf(g.w, "%s:\n", fn.Name)
	fmt.Fprintf(g.w, "\taddi sp, sp, -16\n")
	fmt.Fprintf(g.w, "\tsd ra, 8(sp)\n")
	fmt.Fprintf(g.w, "\tsd s0, 0(sp)\n")
	fmt.Fprintf(g.w, "\taddi s0, sp, 16\n")

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
func (g *Generator) generatePhi(phi *ssa.Phi, block *ssa.Block) error {
	destReg := g.allocReg(phi.Dest)

	logger.Debug("Processing phi node", "dest", destReg, "values", len(phi.Values))

	// If only one value (degenerate phi), just do a simple move
	if len(phi.Values) == 1 {
		srcReg := g.valueReg(phi.Values[0].Value)
		fmt.Fprintf(g.w, "\tmv %s, %s\n", destReg, srcReg)
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
		fmt.Fprintf(g.w, "\tadd %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpSub:
		fmt.Fprintf(g.w, "\tsub %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpMul:
		fmt.Fprintf(g.w, "\tmul %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpDiv:
		fmt.Fprintf(g.w, "\tdiv %s, %s, %s\n", destReg, leftReg, rightReg)

	// Comparisons - use slt/sltu and xor tricks for RISC-V
	case ir.OpEq:
		fmt.Fprintf(g.w, "\txor %s, %s, %s\n", destReg, leftReg, rightReg)
		fmt.Fprintf(g.w, "\tsltiu %s, %s, 1\n", destReg, destReg)
	case ir.OpNe:
		fmt.Fprintf(g.w, "\txor %s, %s, %s\n", destReg, leftReg, rightReg)
		fmt.Fprintf(g.w, "\tsltu %s, zero, %s\n", destReg, destReg)
	case ir.OpLt:
		fmt.Fprintf(g.w, "\tslt %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpLe:
		// a <= b  ==  !(a > b)  ==  !(b < a)
		fmt.Fprintf(g.w, "\tslt %s, %s, %s\n", destReg, rightReg, leftReg)
		fmt.Fprintf(g.w, "\txori %s, %s, 1\n", destReg, destReg)
	case ir.OpGt:
		fmt.Fprintf(g.w, "\tslt %s, %s, %s\n", destReg, rightReg, leftReg)
	case ir.OpGe:
		// a >= b  ==  !(a < b)
		fmt.Fprintf(g.w, "\tslt %s, %s, %s\n", destReg, leftReg, rightReg)
		fmt.Fprintf(g.w, "\txori %s, %s, 1\n", destReg, destReg)

	// Boolean operations
	case ir.OpAnd:
		fmt.Fprintf(g.w, "\tand %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpOr:
		fmt.Fprintf(g.w, "\tor %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpXor:
		fmt.Fprintf(g.w, "\txor %s, %s, %s\n", destReg, leftReg, rightReg)

	default:
		return fmt.Errorf("unsupported operation: %v", binop.Op)
	}

	return nil
}

// generateCall emits assembly for function calls
func (g *Generator) generateCall(call *ir.Call) error {
	// Move arguments to registers (RISC-V ABI: a0-a7)
	for i, arg := range call.Args {
		if i >= len(ArgRegs) {
			return fmt.Errorf("too many arguments (stack args not yet supported)")
		}
		argReg := g.valueReg(arg)
		if argReg != ArgRegs[i] {
			fmt.Fprintf(g.w, "\tmv %s, %s\n", ArgRegs[i], argReg)
		}
	}

	// Call function
	fmt.Fprintf(g.w, "\tcall %s\n", call.Function)

	// Move result to destination
	destReg := g.allocReg(call.Dest)
	if destReg != "a0" {
		fmt.Fprintf(g.w, "\tmv %s, a0\n", destReg)
	}

	return nil
}

// generateLoad emits assembly for load instructions
func (g *Generator) generateLoad(load *ir.Load) error {
	srcReg := g.valueReg(load.Src)
	destReg := g.allocReg(load.Dest)
	fmt.Fprintf(g.w, "\tmv %s, %s\n", destReg, srcReg)
	return nil
}

// generateStore emits assembly for store instructions
func (g *Generator) generateStore(store *ir.Store) error {
	srcReg := g.valueReg(store.Src)
	destReg := g.valueReg(store.Dest)
	fmt.Fprintf(g.w, "\tmv %s, %s\n", destReg, srcReg)
	return nil
}

// generateTerm emits assembly for terminator instructions
func (g *Generator) generateTerm(term ir.Terminator) error {
	switch t := term.(type) {
	case *ir.Return:
		// Move return value to a0
		if t.Value != nil {
			valReg := g.valueReg(t.Value)
			if valReg != "a0" {
				fmt.Fprintf(g.w, "\tmv a0, %s\n", valReg)
			}
		}

		// Epilogue
		fmt.Fprintf(g.w, "\tld ra, 8(sp)\n")
		fmt.Fprintf(g.w, "\tld s0, 0(sp)\n")
		fmt.Fprintf(g.w, "\taddi sp, sp, 16\n")
		fmt.Fprintf(g.w, "\tret\n")

	case *ir.Branch:
		fmt.Fprintf(g.w, "\tj .L%s\n", t.Target)

	case *ir.CondBranch:
		condReg := g.valueReg(t.Cond)
		fmt.Fprintf(g.w, "\tandi %s, %s, 1\n", condReg, condReg)
		fmt.Fprintf(g.w, "\tbnez %s, .L%s\n", condReg, t.TrueBlock)
		fmt.Fprintf(g.w, "\tj .L%s\n", t.FalseBlock)

	default:
		return fmt.Errorf("unsupported terminator: %T", term)
	}

	return nil
}

// valueReg returns the register or immediate for a value
func (g *Generator) valueReg(val ir.Value) string {
	switch v := val.(type) {
	case *ir.Const:
		// Check if already loaded
		if reg, ok := g.valRegs[v]; ok {
			return reg
		}
		// Load immediate into a register
		reg := g.allocReg(v)
		// RISC-V: use li pseudo-instruction for loading immediates
		fmt.Fprintf(g.w, "\tli %s, %d\n", reg, v.Val)
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
		// Parameters arrive in a0-a7
		if idx, ok := g.paramMap[v]; ok && idx < len(ArgRegs) {
			g.valRegs[v] = ArgRegs[idx]
			return ArgRegs[idx]
		}
		// Fallback: allocate new register
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
		// Saved registers: s1-s11 (s0 is frame pointer)
		available: []string{"s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11"},
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

// RISC-V calling convention (RV64I)
var (
	// Argument registers a0-a7
	ArgRegs = []string{"a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7"}
	// Return register
	RetReg = "a0"
	// Saved registers (callee-saved)
	SavedRegs = []string{"s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11"}
	// Temporary registers (caller-saved)
	TempRegs = []string{"t0", "t1", "t2", "t3", "t4", "t5", "t6"}
	// Zero register
	Zero = "zero"
	// Return address
	RetAddr = "ra"
	// Stack pointer
	StackPointer = "sp"
	// Frame pointer
	FramePointer = "s0"
)
