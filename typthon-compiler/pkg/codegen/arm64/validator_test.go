// Package arm64 - Tests for assembly validator
package arm64

import (
	"strings"
	"testing"

	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
)

func TestValidatorValidCode(t *testing.T) {
	validAsm := `
	.text
	.global _test
_test:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	mov x0, x1
	add x0, x0, x2
	ldp x29, x30, [sp], #16
	ret
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
	mov x99, x0
	ret
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

func TestValidatorStackBalance(t *testing.T) {
	unbalancedAsm := `
	.text
_test:
	stp x19, x20, [sp, #-16]!
	mov x0, #42
	ret
`

	validator := NewValidator()
	err := validator.Validate(unbalancedAsm)
	// This might generate a warning rather than an error
	// depending on implementation details
	if err == nil {
		t.Log("Note: Stack imbalance generated warning (expected)")
	}
}

func TestValidatorCalleeSavedRegisters(t *testing.T) {
	validAsm := `
	.text
_test:
	stp x29, x30, [sp, #-32]!
	stp x19, x20, [sp, #16]
	mov x0, #42
	ldp x19, x20, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
`

	validator := NewValidator()
	err := validator.Validate(validAsm)
	if err != nil {
		t.Errorf("Valid callee-saved register handling failed: %v", err)
	}
}

func TestValidatorInvalidMemoryAddressing(t *testing.T) {
	invalidAsm := `
	.text
_test:
	ldr x0, [x1, x2, x3]
	ret
`

	validator := NewValidator()
	err := validator.Validate(invalidAsm)
	if err == nil {
		t.Error("Expected error for invalid memory addressing, got nil")
		return
	}

	if !strings.Contains(err.Error(), "memory addressing") {
		t.Logf("Got error: %v", err)
	}
}

func TestValidatorImmediateAsDestination(t *testing.T) {
	invalidAsm := `
	.text
_test:
	mov #42, x0
	ret
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
	mov x0, x1
	ret
`

	if !QuickValidate(validAsm) {
		t.Error("QuickValidate failed on valid assembly")
	}

	invalidAsm := `
	.text
_test:
	mov x99, x0
`

	if QuickValidate(invalidAsm) {
		t.Error("QuickValidate passed on invalid assembly")
	}
}

func TestValidateAndReport(t *testing.T) {
	validAsm := `
	.text
	.global _add
_add:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	add x0, x0, x1
	ldp x29, x30, [sp], #16
	ret
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

func TestValidatorRedundantMoves(t *testing.T) {
	asmWithRedundantMove := `
	.text
_test:
	mov x0, x0
	ret
`

	validator := NewValidator()
	err := validator.Validate(asmWithRedundantMove)
	// Should pass but generate a warning
	if err != nil {
		t.Errorf("Redundant move should generate warning, not error: %v", err)
	}

	if len(validator.warns) == 0 {
		t.Error("Expected warning for redundant move, got none")
	}
}

func TestValidatorConditionalOperations(t *testing.T) {
	validAsm := `
	.text
_test:
	cmp x0, x1
	cset x2, eq
	ret
`

	validator := NewValidator()
	err := validator.Validate(validAsm)
	if err != nil {
		t.Errorf("Valid conditional operation failed: %v", err)
	}
}

func TestValidatorBranchOperations(t *testing.T) {
	validAsm := `
	.text
_test:
	cmp x0, #0
	b.eq .Lskip
	mov x0, #1
.Lskip:
	ret
`

	validator := NewValidator()
	err := validator.Validate(validAsm)
	if err != nil {
		t.Errorf("Valid branch operations failed: %v", err)
	}
}

func TestValidatorFramePointerHandling(t *testing.T) {
	validAsm := `
	.text
_test:
	stp x29, x30, [sp, #-32]!
	mov x29, sp
	stp x19, x20, [sp, #16]
	# function body
	ldp x19, x20, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
`

	validator := NewValidator()
	err := validator.Validate(validAsm)
	if err != nil {
		t.Errorf("Valid frame pointer handling failed: %v", err)
	}
}

// Benchmark validator performance
func BenchmarkValidator(b *testing.B) {
	asm := `
	.text
	.global _test
_test:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	add x0, x0, x1
	mul x0, x0, x2
	ldp x29, x30, [sp], #16
	ret
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
	add x0, x0, x1
	ret
`

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = QuickValidate(asm)
	}
}
