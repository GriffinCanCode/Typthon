// Package amd64 - Parameter handling for function calls
package amd64

import (
	"fmt"

	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/ssa"
)

// setupParameters generates code to move parameters from calling convention
// registers to the locations we'll use them
func (g *Generator) setupParameters(fn *ssa.Function) error {
	// For Phase 1: We'll fetch parameters directly from calling registers
	// In IR, parameters are tracked in the function
	// We need to extract them properly

	// This is called after the prologue
	// Parameters arrive in: %rdi, %rsi, %rdx, %rcx, %r8, %r9

	// For now, we'll handle this implicitly in valueReg for Param values
	// Future: explicit parameter setup

	return nil
}

// getParamReg returns the register for a function parameter
func getParamReg(index int) (string, error) {
	if index >= len(ArgRegs) {
		return "", fmt.Errorf("parameter index %d out of range", index)
	}
	return ArgRegs[index], nil
}

// buildParamMap creates a mapping from Param values to their registers
func buildParamMap(params []*ir.Param) map[*ir.Param]string {
	paramMap := make(map[*ir.Param]string)
	for i, param := range params {
		if i < len(ArgRegs) {
			paramMap[param] = ArgRegs[i]
		}
	}
	return paramMap
}
