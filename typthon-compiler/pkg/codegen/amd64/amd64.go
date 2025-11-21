// Package amd64 implements x86-64 code generation.
//
// Design: Direct assembly generation, no LLVM dependencies.
// System V calling convention for Unix/macOS.
// Fast compilation over perfect code - optimize for compile speed.
package amd64

import (
	"fmt"
	"io"
)

// Generator generates x86-64 assembly
type Generator struct {
	w    io.Writer
	regs *RegAlloc
}

func NewGenerator(w io.Writer) *Generator {
	return &Generator{
		w:    w,
		regs: NewRegAlloc(),
	}
}

// Generate emits assembly for a function
func (g *Generator) Generate(name string) error {
	// Prologue
	fmt.Fprintf(g.w, "\t.globl %s\n", name)
	fmt.Fprintf(g.w, "%s:\n", name)
	fmt.Fprintf(g.w, "\tpush %%rbp\n")
	fmt.Fprintf(g.w, "\tmov %%rsp, %%rbp\n")

	// TODO: Generate body

	// Epilogue
	fmt.Fprintf(g.w, "\tpop %%rbp\n")
	fmt.Fprintf(g.w, "\tret\n")

	return nil
}

// Emit simple instructions
func (g *Generator) EmitAdd(dest, src1, src2 string) {
	fmt.Fprintf(g.w, "\tmov %s, %s\n", src1, dest)
	fmt.Fprintf(g.w, "\tadd %s, %s\n", src2, dest)
}

func (g *Generator) EmitReturn(val string) {
	fmt.Fprintf(g.w, "\tmov %s, %%rax\n", val)
}

// RegAlloc implements linear scan register allocation
type RegAlloc struct {
	available []string
	used      map[string]bool
}

func NewRegAlloc() *RegAlloc {
	return &RegAlloc{
		// System V ABI registers (callee-saved)
		available: []string{"rbx", "r12", "r13", "r14", "r15"},
		used:      make(map[string]bool),
	}
}

func (r *RegAlloc) Alloc() string {
	for _, reg := range r.available {
		if !r.used[reg] {
			r.used[reg] = true
			return "%" + reg
		}
	}
	// TODO: Spill to stack
	panic("out of registers")
}

func (r *RegAlloc) Free(reg string) {
	r.used[reg] = false
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
