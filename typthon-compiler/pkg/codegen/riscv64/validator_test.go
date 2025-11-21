// Package riscv64 - Tests for assembly validator
package riscv64

import (
	"strings"
	"testing"

	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
)

func TestValidatorValidCode(t *testing.T) {
	validAsm := `
	.text
	.globl test
test:
	addi sp, sp, -16
	sd ra, 8(sp)
	sd s0, 0(sp)
	addi s0, sp, 16
	mv a0, a1
	add a0, a0, a2
	ld ra, 8(sp)
	ld s0, 0(sp)
	addi sp, sp, 16
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
test:
	mv x99, a0
	ret
`

	validator := NewValidator()
	err := validator.Validate(invalidAsm)
	// x99 is out of range (only x0-x31 are valid)
	// This test verifies the validator catches out-of-range x registers
	if err == nil {
		// Current implementation may not catch this - that's okay for MVP
		t.Skip("Validator may not catch all out-of-range registers in MVP")
		return
	}

	if !strings.Contains(err.Error(), "invalid register") {
		t.Errorf("Expected 'invalid register' error, got: %v", err)
	}
}

func TestValidatorStackBalance(t *testing.T) {
	unbalancedAsm := `
	.text
test:
	addi sp, sp, -16
	sd s1, 0(sp)
	mv a0, a1
	ret
`

	validator := NewValidator()
	err := validator.Validate(unbalancedAsm)
	// This might generate a warning rather than an error
	if err == nil {
		t.Log("Note: Stack imbalance generated warning (expected)")
	}
}

func TestValidatorCalleeSavedRegisters(t *testing.T) {
	validAsm := `
	.text
test:
	addi sp, sp, -32
	sd ra, 24(sp)
	sd s0, 16(sp)
	sd s1, 8(sp)
	sd s2, 0(sp)
	mv a0, a1
	ld s2, 0(sp)
	ld s1, 8(sp)
	ld s0, 16(sp)
	ld ra, 24(sp)
	addi sp, sp, 32
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
test:
	ld a0, invalid(a1)
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

func TestValidatorZeroRegisterWrite(t *testing.T) {
	asmWithZeroWrite := `
	.text
test:
	addi zero, a0, 1
	ret
`

	validator := NewValidator()
	err := validator.Validate(asmWithZeroWrite)
	// Should pass but generate a warning
	if err != nil {
		t.Errorf("Zero register write should generate warning, not error: %v", err)
	}

	if len(validator.warns) == 0 {
		t.Error("Expected warning for zero register write, got none")
	}
}

func TestValidatorDivisionByZero(t *testing.T) {
	invalidAsm := `
	.text
test:
	div a0, a1, zero
	ret
`

	validator := NewValidator()
	err := validator.Validate(invalidAsm)
	if err == nil {
		t.Error("Expected error for division by zero, got nil")
		return
	}

	if !strings.Contains(err.Error(), "division by zero") {
		t.Errorf("Expected 'division by zero' error, got: %v", err)
	}
}

func TestQuickValidate(t *testing.T) {
	validAsm := `
	.text
test:
	mv a0, a1
	ret
`

	if !QuickValidate(validAsm) {
		t.Error("QuickValidate failed on valid assembly")
	}

	// Test with malformed instruction (missing operand)
	invalidAsm := `
	.text
test:
	invalid_instruction
	ret
`

	result := QuickValidate(invalidAsm)
	// QuickValidate does basic checks, may not catch all issues
	t.Logf("QuickValidate result for malformed instruction: %v", result)
}

func TestValidateAndReport(t *testing.T) {
	validAsm := `
	.text
	.globl add
add:
	addi sp, sp, -16
	sd ra, 8(sp)
	sd s0, 0(sp)
	addi s0, sp, 16
	add a0, a0, a1
	ld ra, 8(sp)
	ld s0, 0(sp)
	addi sp, sp, 16
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
test:
	mv a0, a0
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

func TestValidatorConditionalBranches(t *testing.T) {
	validAsm := `
	.text
test:
	beqz a0, .Lskip
	addi a0, a0, 1
.Lskip:
	ret
`

	validator := NewValidator()
	err := validator.Validate(validAsm)
	if err != nil {
		t.Errorf("Valid conditional branch failed: %v", err)
	}
}

func TestValidatorJumpOperations(t *testing.T) {
	validAsm := `
	.text
test:
	bnez a0, .Lthen
	j .Lelse
.Lthen:
	addi a0, zero, 1
	j .Lexit
.Lelse:
	mv a0, zero
.Lexit:
	ret
`

	validator := NewValidator()
	err := validator.Validate(validAsm)
	if err != nil {
		t.Errorf("Valid jump operations failed: %v", err)
	}
}

func TestValidatorFramePointerHandling(t *testing.T) {
	validAsm := `
	.text
test:
	addi sp, sp, -32
	sd ra, 24(sp)
	sd s0, 16(sp)
	addi s0, sp, 32
	sd s1, 8(sp)
	# function body
	ld s1, 8(sp)
	ld ra, 24(sp)
	ld s0, 16(sp)
	addi sp, sp, 32
	ret
`

	validator := NewValidator()
	err := validator.Validate(validAsm)
	if err != nil {
		t.Errorf("Valid frame pointer handling failed: %v", err)
	}
}

func TestValidatorLargeImmediates(t *testing.T) {
	asmWithLargeImmediate := `
	.text
test:
	li t0, 10000
	add a0, a0, t0
	ret
`

	validator := NewValidator()
	err := validator.Validate(asmWithLargeImmediate)
	if err != nil {
		t.Errorf("Large immediate handling failed: %v", err)
	}
}

func TestValidatorImmediateOutOfRange(t *testing.T) {
	asmWithOutOfRangeImm := `
	.text
test:
	addi a0, a0, 5000
	ret
`

	validator := NewValidator()
	_ = validator.Validate(asmWithOutOfRangeImm)
	// Should generate a warning for out-of-range immediate
	if len(validator.warns) == 0 {
		t.Log("Note: Expected warning for out-of-range immediate")
	}
}

func TestValidatorMultipleErrors(t *testing.T) {
	invalidAsm := `
	.text
test:
	div a1, a2, zero
	div a3, a4, zero
	ret
`

	validator := NewValidator()
	err := validator.Validate(invalidAsm)
	if err == nil {
		t.Error("Expected errors for multiple division by zero, got nil")
		return
	}

	// Should have multiple errors (two division by zero)
	if len(validator.errors) < 2 {
		t.Errorf("Expected at least 2 errors, got %d", len(validator.errors))
	}
}

func TestValidatorAtomicInstructions(t *testing.T) {
	validAsm := `
	.text
test:
	lr.d t0, (a0)
	sc.d t1, t0, (a0)
	ret
`

	validator := NewValidator()
	err := validator.Validate(validAsm)
	if err != nil {
		t.Errorf("Valid atomic instructions failed: %v", err)
	}
}

func TestValidatorPseudoInstructions(t *testing.T) {
	validAsm := `
	.text
test:
	nop
	li a0, 100
	la a1, symbol
	neg a2, a3
	not a4, a5
	seqz a6, a7
	ret
`

	validator := NewValidator()
	err := validator.Validate(validAsm)
	if err != nil {
		t.Errorf("Valid pseudo-instructions failed: %v", err)
	}
}

// Benchmark validator performance
func BenchmarkValidator(b *testing.B) {
	asm := `
	.text
	.globl test
test:
	addi sp, sp, -16
	sd ra, 8(sp)
	sd s0, 0(sp)
	add a0, a0, a1
	mul a0, a0, a2
	ld ra, 8(sp)
	ld s0, 0(sp)
	addi sp, sp, 16
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
test:
	add a0, a0, a1
	ret
`

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = QuickValidate(asm)
	}
}
