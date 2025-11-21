// Package amd64 - Tests for assembly validator
package amd64

import (
	"strings"
	"testing"

	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
)

func TestValidatorValidCode(t *testing.T) {
	validAsm := `
	.text
	.globl _test
_test:
	pushq %rbp
	movq %rsp, %rbp
	movq %rdi, %rax
	addq %rsi, %rax
	popq %rbp
	retq
`

	validator := NewValidator()
	err := validator.Validate(validAsm)
	if err != nil {
		t.Errorf("Valid assembly failed validation: %v", err)
	}
}

func TestValidatorInvalidRegister(t *testing.T) {
	invalidAsm := `
	.text
_test:
	movq %invalid, %rax
	retq
`

	validator := NewValidator()
	err := validator.Validate(invalidAsm)
	if err == nil {
		t.Error("Expected error for invalid register, got nil")
	}

	if !strings.Contains(err.Error(), "invalid register") {
		t.Errorf("Expected 'invalid register' error, got: %v", err)
	}
}

func TestValidatorMemoryToMemory(t *testing.T) {
	invalidAsm := `
	.text
_test:
	movq (%rdi), (%rsi)
	retq
`

	validator := NewValidator()
	err := validator.Validate(invalidAsm)
	if err == nil {
		t.Error("Expected error for memory-to-memory move, got nil")
	}
}

func TestValidatorStackBalance(t *testing.T) {
	unbalancedAsm := `
	.text
_test:
	pushq %rbx
	pushq %r12
	movq $42, %rax
	popq %rbx
	retq
`

	validator := NewValidator()
	err := validator.Validate(unbalancedAsm)
	if err == nil {
		t.Error("Expected error for unbalanced stack, got nil")
	}
}

func TestValidatorCalleeSavedRegisters(t *testing.T) {
	validAsm := `
	.text
_test:
	pushq %rbx
	pushq %r12
	movq $42, %rax
	popq %r12
	popq %rbx
	retq
`

	validator := NewValidator()
	err := validator.Validate(validAsm)
	if err != nil {
		t.Errorf("Valid callee-saved register handling failed: %v", err)
	}
}

func TestValidatorDivisionSetup(t *testing.T) {
	validAsm := `
	.text
_test:
	movq %rdi, %rax
	cqto
	idivq %rsi
	retq
`

	validator := NewValidator()
	err := validator.Validate(validAsm)
	if err != nil {
		t.Errorf("Valid division setup failed: %v", err)
	}
}

func TestValidatorInvalidScaleFactor(t *testing.T) {
	invalidAsm := `
	.text
_test:
	movq (%rax,%rbx,3), %rcx
	retq
`

	validator := NewValidator()
	err := validator.Validate(invalidAsm)
	if err == nil {
		t.Error("Expected error for invalid scale factor, got nil")
		return
	}

	if !strings.Contains(err.Error(), "scale factor") {
		t.Errorf("Expected 'scale factor' error, got: %v", err)
	}
}

func TestValidatorImmediateAsDestination(t *testing.T) {
	invalidAsm := `
	.text
_test:
	movq %rax, $42
	retq
`

	validator := NewValidator()
	err := validator.Validate(invalidAsm)
	if err == nil {
		t.Error("Expected error for immediate as destination, got nil")
		return
	}

	if !strings.Contains(err.Error(), "immediate") {
		t.Logf("Got error: %v", err)
	}
}

func TestQuickValidate(t *testing.T) {
	validAsm := `
	.text
_test:
	movq %rdi, %rax
	retq
`

	if !QuickValidate(validAsm) {
		t.Error("QuickValidate failed on valid assembly")
	}

	invalidAsm := `
	.text
_test:
	movq %invalid, %rax
`

	if QuickValidate(invalidAsm) {
		t.Error("QuickValidate passed on invalid assembly")
	}
}

func TestValidateAndReport(t *testing.T) {
	validAsm := `
	.text
	.globl _add
_add:
	pushq %rbp
	movq %rsp, %rbp
	movq %rdi, %rax
	addq %rsi, %rax
	popq %rbp
	retq
`

	passed, report := ValidateAndReport(validAsm)
	if !passed {
		t.Errorf("ValidateAndReport failed on valid assembly:\n%s", report)
	}

	if !strings.Contains(report, "PASSED") {
		t.Errorf("Report doesn't contain PASSED status:\n%s", report)
	}

	if !strings.Contains(report, "Statistics") {
		t.Errorf("Report doesn't contain statistics:\n%s", report)
	}
}

func TestValidatorWithGeneratedCode(t *testing.T) {
	// Test with actual code from our generator
	paramA := &ir.Param{Name: "a", Type: ir.IntType{}}
	paramB := &ir.Param{Name: "b", Type: ir.IntType{}}
	temp := &ir.Temp{ID: 0, Type: ir.IntType{}}

	fn := &ir.Function{
		Name:       "add",
		Params:     []*ir.Param{paramA, paramB},
		ReturnType: ir.IntType{},
		Blocks: []*ir.Block{
			{
				Label: "entry",
				Insts: []ir.Inst{
					&ir.BinOp{Dest: temp, Op: ir.OpAdd, L: paramA, R: paramB},
				},
				Term: &ir.Return{Value: temp},
			},
		},
	}

	asm := generateFunctionTest(fn)

	validator := NewValidator()
	err := validator.Validate(asm)
	if err != nil {
		t.Errorf("Generated code failed validation:\n%s\nError: %v", asm, err)
	}
}

func TestValidatorComplexFunction(t *testing.T) {
	// Test with a complex function including calls
	paramA := &ir.Param{Name: "a", Type: ir.IntType{}}
	paramB := &ir.Param{Name: "b", Type: ir.IntType{}}
	temp0 := &ir.Temp{ID: 0, Type: ir.IntType{}}
	temp1 := &ir.Temp{ID: 1, Type: ir.IntType{}}
	temp2 := &ir.Temp{ID: 2, Type: ir.IntType{}}

	fn := &ir.Function{
		Name:       "complex",
		Params:     []*ir.Param{paramA, paramB},
		ReturnType: ir.IntType{},
		Blocks: []*ir.Block{
			{
				Label: "entry",
				Insts: []ir.Inst{
					&ir.BinOp{Dest: temp0, Op: ir.OpAdd, L: paramA, R: paramB},
					&ir.BinOp{Dest: temp1, Op: ir.OpMul, L: temp0, R: &ir.Const{Val: 2, Type: ir.IntType{}}},
					&ir.Call{Dest: temp2, Function: "helper", Args: []ir.Value{temp1}},
				},
				Term: &ir.Return{Value: temp2},
			},
		},
	}

	asm := generateFunctionTest(fn)

	passed, report := ValidateAndReport(asm)
	if !passed {
		t.Errorf("Complex generated code failed validation:\n%s\nReport:\n%s", asm, report)
	}
}

// Benchmark validator performance
func BenchmarkValidator(b *testing.B) {
	asm := `
	.text
	.globl _test
_test:
	pushq %rbp
	movq %rsp, %rbp
	movq %rdi, %rax
	addq %rsi, %rax
	imulq %rdx, %rax
	popq %rbp
	retq
`

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		validator := NewValidator()
		_ = validator.Validate(asm)
	}
}

func BenchmarkQuickValidate(b *testing.B) {
	asm := `
	.text
_test:
	movq %rdi, %rax
	addq %rsi, %rax
	retq
`

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = QuickValidate(asm)
	}
}

func TestValidatorCallerSavedPreservation(t *testing.T) {
	// Test: caller-saved register used before call but not preserved
	asmWithIssue := `
	.text
	.globl main
main:
	pushq %rbp
	movq %rsp, %rbp
	movq $42, %r10        # r10 is caller-saved, gets a value
	call some_function     # call without saving r10
	addq %r10, %rax       # using r10 after call (may be clobbered)
	popq %rbp
	retq
`

	validator := NewValidator()
	err := validator.Validate(asmWithIssue)

	// Debug output
	t.Logf("Errors: %d, Warnings: %d", len(validator.errors), len(validator.warns))
	for _, w := range validator.warns {
		t.Logf("Warning: %s", w.Message)
	}

	// Should have a warning about caller-saved preservation
	if len(validator.warns) == 0 {
		t.Error("expected warning about caller-saved register preservation")
	}

	// Test: properly preserved caller-saved register
	asmValid := `
	.text
	.globl main
main:
	pushq %rbp
	movq %rsp, %rbp
	movq $42, %rcx
	pushq %rcx             # save before call
	call some_function
	popq %rcx              # restore after call
	addq %rcx, %rax
	popq %rbp
	retq
`

	validator2 := NewValidator()
	err = validator2.Validate(asmValid)
	if err != nil {
		t.Errorf("valid code should not produce errors: %v", err)
	}
}

func TestValidatorRedundantMoves(t *testing.T) {
	// Test: same register move
	asmSameReg := `
	.text
	.globl test
test:
	movq %rax, %rax       # redundant: same source and dest
	retq
`

	validator := NewValidator()
	_ = validator.Validate(asmSameReg)

	t.Logf("Same reg test - Warnings: %d", len(validator.warns))
	for _, w := range validator.warns {
		t.Logf("  Warning: %s", w.Message)
	}

	if len(validator.warns) == 0 {
		t.Error("expected warning about redundant move (same register)")
	}

	found := false
	for _, warn := range validator.warns {
		if strings.Contains(warn.Message, "redundant move") {
			found = true
			break
		}
	}
	if !found {
		t.Error("expected redundant move warning")
	}

	// Test: swap pattern
	asmSwap := `
	.text
	.globl test
test:
	movq %rax, %rbx
	movq %rbx, %rax       # swap pattern
	retq
`

	validator2 := NewValidator()
	_ = validator2.Validate(asmSwap)

	foundSwap := false
	for _, warn := range validator2.warns {
		if strings.Contains(warn.Message, "swap pattern") {
			foundSwap = true
			break
		}
	}
	if !foundSwap {
		t.Error("expected swap pattern warning")
	}

	// Test: duplicate move
	asmDuplicate := `
	.text
	.globl test
test:
	movq %rax, %rbx
	movq %rax, %rbx       # exact duplicate
	retq
`

	validator3 := NewValidator()
	_ = validator3.Validate(asmDuplicate)

	foundDup := false
	for _, warn := range validator3.warns {
		if strings.Contains(warn.Message, "duplicate move") {
			foundDup = true
			break
		}
	}
	if !foundDup {
		t.Error("expected duplicate move warning")
	}
}

func TestValidatorOptimizedCode(t *testing.T) {
	// Test that clean, optimized code passes without warnings
	asmClean := `
	.text
	.globl factorial
factorial:
	pushq %rbp
	movq %rsp, %rbp
	cmpq $1, %rdi
	jle .L_base
	pushq %rdi            # save before recursion
	subq $1, %rdi
	call factorial
	popq %rdi
	imulq %rdi, %rax
	popq %rbp
	retq
.L_base:
	movq $1, %rax
	popq %rbp
	retq
`

	validator := NewValidator()
	err := validator.Validate(asmClean)

	if err != nil {
		t.Errorf("clean optimized code should not have errors: %v", err)
	}

	// This clean code should have minimal warnings
	if len(validator.warns) > 1 {
		t.Errorf("clean code should have few warnings, got %d", len(validator.warns))
		for _, w := range validator.warns {
			t.Logf("  Warning: %s", w.Message)
		}
	}
}
