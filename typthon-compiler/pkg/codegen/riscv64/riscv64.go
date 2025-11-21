// Package riscv64 implements RISC-V 64-bit code generation.
//
// Design: Future-proofing for RISC-V servers and embedded systems.
package riscv64

import (
	"fmt"
	"io"
)

// Generator generates RISC-V assembly
type Generator struct {
	w io.Writer
}

func NewGenerator(w io.Writer) *Generator {
	return &Generator{w: w}
}

// Generate emits RISC-V assembly
func (g *Generator) Generate(name string) error {
	// Prologue
	fmt.Fprintf(g.w, "\t.globl %s\n", name)
	fmt.Fprintf(g.w, "%s:\n", name)
	fmt.Fprintf(g.w, "\taddi sp, sp, -16\n")
	fmt.Fprintf(g.w, "\tsd ra, 8(sp)\n")
	fmt.Fprintf(g.w, "\tsd fp, 0(sp)\n")
	fmt.Fprintf(g.w, "\taddi fp, sp, 16\n")

	// TODO: Generate body

	// Epilogue
	fmt.Fprintf(g.w, "\tld ra, 8(sp)\n")
	fmt.Fprintf(g.w, "\tld fp, 0(sp)\n")
	fmt.Fprintf(g.w, "\taddi sp, sp, 16\n")
	fmt.Fprintf(g.w, "\tret\n")

	return nil
}

// RISC-V calling convention
var (
	// Argument registers a0-a7
	ArgRegs = []string{"a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7"}
	// Return register
	RetReg = "a0"
	// Saved registers
	SavedRegs = []string{"s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11"}
)
