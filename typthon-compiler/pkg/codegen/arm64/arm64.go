// Package arm64 implements ARM64 code generation.
//
// Design: Direct assembly generation for ARM64/AArch64.
// Apple Silicon and ARM server optimized.
package arm64

import (
	"fmt"
	"io"
)

// Generator generates ARM64 assembly
type Generator struct {
	w io.Writer
}

func NewGenerator(w io.Writer) *Generator {
	return &Generator{w: w}
}

// Generate emits ARM64 assembly
func (g *Generator) Generate(name string) error {
	// Prologue
	fmt.Fprintf(g.w, "\t.global _%s\n", name)
	fmt.Fprintf(g.w, "_%s:\n", name)
	fmt.Fprintf(g.w, "\tstp x29, x30, [sp, #-16]!\n")
	fmt.Fprintf(g.w, "\tmov x29, sp\n")

	// TODO: Generate body

	// Epilogue
	fmt.Fprintf(g.w, "\tldp x29, x30, [sp], #16\n")
	fmt.Fprintf(g.w, "\tret\n")

	return nil
}

// ARM64 calling convention (AAPCS64)
var (
	// Argument registers x0-x7
	ArgRegs = []string{"x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7"}
	// Return register
	RetReg = "x0"
	// Callee-saved
	CalleeSaved = []string{"x19", "x20", "x21", "x22", "x23", "x24", "x25", "x26", "x27", "x28"}
)
