// Package arm64 - Comprehensive unit tests for ARM64/AArch64 code generation
package arm64

import (
	"bytes"
	"strings"
	"testing"

	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/ssa"
)

// TestArithmeticOperations tests all arithmetic operations
func TestArithmeticOperations(t *testing.T) {
	tests := []struct {
		name     string
		op       ir.Op
		wantInst []string // Expected assembly instructions
	}{
		{
			name:     "addition",
			op:       ir.OpAdd,
			wantInst: []string{"add"},
		},
		{
			name:     "subtraction",
			op:       ir.OpSub,
			wantInst: []string{"sub"},
		},
		{
			name:     "multiplication",
			op:       ir.OpMul,
			wantInst: []string{"mul"},
		},
		{
			name:     "division",
			op:       ir.OpDiv,
			wantInst: []string{"sdiv"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			asm := generateBinOpTest(tt.op)
			for _, inst := range tt.wantInst {
				if !strings.Contains(asm, inst) {
					t.Errorf("expected instruction %q not found in:\n%s", inst, asm)
				}
			}
		})
	}
}

// TestComparisonOperations tests all comparison operations
func TestComparisonOperations(t *testing.T) {
	tests := []struct {
		name     string
		op       ir.Op
		wantInst []string
	}{
		{
			name:     "equal",
			op:       ir.OpEq,
			wantInst: []string{"cmp", "cset"},
		},
		{
			name:     "not_equal",
			op:       ir.OpNe,
			wantInst: []string{"cmp", "cset"},
		},
		{
			name:     "less_than",
			op:       ir.OpLt,
			wantInst: []string{"cmp", "cset"},
		},
		{
			name:     "less_equal",
			op:       ir.OpLe,
			wantInst: []string{"cmp", "cset"},
		},
		{
			name:     "greater_than",
			op:       ir.OpGt,
			wantInst: []string{"cmp", "cset"},
		},
		{
			name:     "greater_equal",
			op:       ir.OpGe,
			wantInst: []string{"cmp", "cset"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			asm := generateBinOpTest(tt.op)
			for _, inst := range tt.wantInst {
				if !strings.Contains(asm, inst) {
					t.Errorf("expected instruction %q not found in:\n%s", inst, asm)
				}
			}
		})
	}
}

// TestBooleanOperations tests boolean logic operations
func TestBooleanOperations(t *testing.T) {
	tests := []struct {
		name     string
		op       ir.Op
		wantInst []string
	}{
		{
			name:     "and",
			op:       ir.OpAnd,
			wantInst: []string{"and"},
		},
		{
			name:     "or",
			op:       ir.OpOr,
			wantInst: []string{"orr"},
		},
		{
			name:     "xor",
			op:       ir.OpXor,
			wantInst: []string{"eor"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			asm := generateBinOpTest(tt.op)
			for _, inst := range tt.wantInst {
				if !strings.Contains(asm, inst) {
					t.Errorf("expected instruction %q not found in:\n%s", inst, asm)
				}
			}
		})
	}
}

// TestFunctionCall tests function call generation with register preservation
func TestFunctionCall(t *testing.T) {
	paramA := &ir.Param{Name: "a", Type: ir.IntType{}}
	paramB := &ir.Param{Name: "b", Type: ir.IntType{}}
	temp0 := &ir.Temp{ID: 0, Type: ir.IntType{}}
	temp1 := &ir.Temp{ID: 1, Type: ir.IntType{}}

	fn := &ir.Function{
		Name:       "test_call",
		Params:     []*ir.Param{paramA, paramB},
		ReturnType: ir.IntType{},
		Blocks: []*ir.Block{
			{
				Label: "entry",
				Insts: []ir.Inst{
					&ir.BinOp{Dest: temp0, Op: ir.OpAdd, L: paramA, R: paramB},
					&ir.Call{Dest: temp1, Function: "helper", Args: []ir.Value{temp0}},
				},
				Term: &ir.Return{Value: temp1},
			},
		},
	}

	prog := &ir.Program{Functions: []*ir.Function{fn}}
	ssaProg := ssa.Convert(prog)

	var buf bytes.Buffer
	gen := NewGenerator(&buf)
	if err := gen.Generate(ssaProg); err != nil {
		t.Fatalf("Generate failed: %v", err)
	}

	asm := buf.String()

	// Verify call instruction exists
	if !strings.Contains(asm, "bl") {
		t.Error("expected bl instruction not found")
	}

	// Verify frame setup (stp with x29, x30)
	if !strings.Contains(asm, "stp") || !strings.Contains(asm, "x29") {
		t.Error("frame setup not found")
	}
}

// TestMemoryOperations tests load and store instructions
func TestMemoryOperations(t *testing.T) {
	t.Run("direct_load", func(t *testing.T) {
		param := &ir.Param{Name: "x", Type: ir.IntType{}}
		temp0 := &ir.Temp{ID: 0, Type: ir.IntType{}}

		fn := &ir.Function{
			Name:       "test_load",
			Params:     []*ir.Param{param},
			ReturnType: ir.IntType{},
			Blocks: []*ir.Block{
				{
					Label: "entry",
					Insts: []ir.Inst{
						&ir.Load{Dest: temp0, Src: param},
					},
					Term: &ir.Return{Value: temp0},
				},
			},
		}

		asm := generateFunctionTest(fn)
		// ARM64 uses mov, ldr, or str for memory operations
		hasMemOp := strings.Contains(asm, "mov") || strings.Contains(asm, "ldr") || strings.Contains(asm, "str")
		if !hasMemOp {
			t.Error("expected memory operation not found")
		}
	})

	t.Run("indirect_load", func(t *testing.T) {
		ptrParam := &ir.Param{Name: "ptr", Type: ir.PtrType{Elem: ir.IntType{}}}
		dest := &ir.Temp{ID: 0, Type: ir.IntType{}}

		fn := &ir.Function{
			Name:       "test_indirect_load",
			Params:     []*ir.Param{ptrParam},
			ReturnType: ir.IntType{},
			Blocks: []*ir.Block{
				{
					Label: "entry",
					Insts: []ir.Inst{
						&ir.Load{Dest: dest, Src: ptrParam},
					},
					Term: &ir.Return{Value: dest},
				},
			},
		}

		asm := generateFunctionTest(fn)
		// Should generate load or move instruction
		hasMemOp := strings.Contains(asm, "ldr") || strings.Contains(asm, "mov")
		if !hasMemOp {
			t.Error("expected load/move instruction not found")
		}
	})
}

// TestRegisterAllocation tests register allocation strategy
func TestRegisterAllocation(t *testing.T) {
	// Test by generating functions with many temporaries
	params := make([]*ir.Param, 3)
	for i := range params {
		params[i] = &ir.Param{Name: string(rune('a' + i)), Type: ir.IntType{}}
	}

	temps := make([]*ir.Temp, 10)
	for i := range temps {
		temps[i] = &ir.Temp{ID: i, Type: ir.IntType{}}
	}

	fn := &ir.Function{
		Name:       "test_reg_alloc",
		Params:     params,
		ReturnType: ir.IntType{},
		Blocks: []*ir.Block{
			{
				Label: "entry",
				Insts: []ir.Inst{
					&ir.BinOp{Dest: temps[0], Op: ir.OpAdd, L: params[0], R: params[1]},
					&ir.BinOp{Dest: temps[1], Op: ir.OpMul, L: temps[0], R: params[2]},
					&ir.BinOp{Dest: temps[2], Op: ir.OpSub, L: temps[1], R: params[0]},
					&ir.BinOp{Dest: temps[3], Op: ir.OpAdd, L: temps[2], R: temps[1]},
				},
				Term: &ir.Return{Value: temps[3]},
			},
		},
	}

	asm := generateFunctionTest(fn)

	// Verify registers are being used
	if !strings.Contains(asm, "x") {
		t.Error("expected register allocation, no registers found")
	}
}

// TestCallingConvention tests AAPCS64 calling convention
func TestCallingConvention(t *testing.T) {
	// Test argument register order
	expectedArgRegs := []string{"x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7"}
	if len(ArgRegs) != len(expectedArgRegs) {
		t.Errorf("expected %d argument registers, got %d", len(expectedArgRegs), len(ArgRegs))
	}

	for i, reg := range ArgRegs {
		if reg != expectedArgRegs[i] {
			t.Errorf("arg register %d: expected %s, got %s", i, expectedArgRegs[i], reg)
		}
	}

	// Test return register
	if RetReg != "x0" {
		t.Errorf("expected return register x0, got %s", RetReg)
	}

	// Test frame pointer
	if FramePointer != "x29" {
		t.Errorf("expected frame pointer x29, got %s", FramePointer)
	}

	// Test link register
	if LinkReg != "x30" {
		t.Errorf("expected link register x30, got %s", LinkReg)
	}
}

// TestStackOperations tests stack frame management
func TestStackOperations(t *testing.T) {
	// Create a simple function that would trigger stack operations
	paramA := &ir.Param{Name: "a", Type: ir.IntType{}}
	temp0 := &ir.Temp{ID: 0, Type: ir.IntType{}}
	temp1 := &ir.Temp{ID: 1, Type: ir.IntType{}}

	fn := &ir.Function{
		Name:       "test_stack",
		Params:     []*ir.Param{paramA},
		ReturnType: ir.IntType{},
		Blocks: []*ir.Block{
			{
				Label: "entry",
				Insts: []ir.Inst{
					// Multiple operations to potentially trigger spilling
					&ir.BinOp{Dest: temp0, Op: ir.OpAdd, L: paramA, R: &ir.Const{Val: 1, Type: ir.IntType{}}},
					&ir.Call{Dest: temp1, Function: "helper", Args: []ir.Value{temp0}},
				},
				Term: &ir.Return{Value: temp1},
			},
		},
	}

	asm := generateFunctionTest(fn)

	// Verify stack frame setup exists (stp/ldp of x29, x30)
	if !strings.Contains(asm, "stp") || !strings.Contains(asm, "ldp") {
		t.Error("stack frame operations not found")
	}
}

// TestBranchOperations tests control flow instructions
func TestBranchOperations(t *testing.T) {
	t.Run("unconditional_branch", func(t *testing.T) {
		temp0 := &ir.Temp{ID: 0, Type: ir.IntType{}}

		fn := &ir.Function{
			Name:       "test_branch",
			Params:     []*ir.Param{},
			ReturnType: ir.IntType{},
			Blocks: []*ir.Block{
				{
					Label: "entry",
					Insts: []ir.Inst{
						&ir.Load{Dest: temp0, Src: &ir.Const{Val: 42, Type: ir.IntType{}}},
					},
					Term: &ir.Branch{Target: "exit"},
				},
				{
					Label: "exit",
					Insts: []ir.Inst{},
					Term:  &ir.Return{Value: temp0},
				},
			},
		}

		asm := generateFunctionTest(fn)
		if !strings.Contains(asm, "b ") || !strings.Contains(asm, ".L") {
			t.Error("expected unconditional branch instruction not found")
		}
	})

	t.Run("conditional_branch", func(t *testing.T) {
		paramA := &ir.Param{Name: "a", Type: ir.IntType{}}
		paramB := &ir.Param{Name: "b", Type: ir.IntType{}}
		temp0 := &ir.Temp{ID: 0, Type: ir.IntType{}}
		temp1 := &ir.Temp{ID: 1, Type: ir.IntType{}}
		temp2 := &ir.Temp{ID: 2, Type: ir.IntType{}}

		fn := &ir.Function{
			Name:       "test_cond_branch",
			Params:     []*ir.Param{paramA, paramB},
			ReturnType: ir.IntType{},
			Blocks: []*ir.Block{
				{
					Label: "entry",
					Insts: []ir.Inst{
						&ir.BinOp{Dest: temp0, Op: ir.OpEq, L: paramA, R: paramB},
					},
					Term: &ir.CondBranch{Cond: temp0, TrueBlock: "then", FalseBlock: "else"},
				},
				{
					Label: "then",
					Insts: []ir.Inst{
						&ir.Load{Dest: temp1, Src: &ir.Const{Val: 1, Type: ir.IntType{}}},
					},
					Term: &ir.Branch{Target: "exit"},
				},
				{
					Label: "else",
					Insts: []ir.Inst{
						&ir.Load{Dest: temp1, Src: &ir.Const{Val: 0, Type: ir.IntType{}}},
					},
					Term: &ir.Branch{Target: "exit"},
				},
				{
					Label: "exit",
					Insts: []ir.Inst{
						&ir.Load{Dest: temp2, Src: temp1},
					},
					Term: &ir.Return{Value: temp2},
				},
			},
		}

		asm := generateFunctionTest(fn)
		// Should contain conditional branch (b.ne or tst + b.ne)
		hasCondBranch := strings.Contains(asm, "b.ne") || (strings.Contains(asm, "tst") && strings.Contains(asm, "b "))
		if !hasCondBranch {
			t.Error("expected conditional branch instruction not found")
		}
	})
}

// Helper functions for test generation

// generateBinOpTest generates assembly for a binary operation
func generateBinOpTest(op ir.Op) string {
	paramA := &ir.Param{Name: "a", Type: ir.IntType{}}
	paramB := &ir.Param{Name: "b", Type: ir.IntType{}}
	temp := &ir.Temp{ID: 0, Type: ir.IntType{}}

	fn := &ir.Function{
		Name:       "test_binop",
		Params:     []*ir.Param{paramA, paramB},
		ReturnType: ir.IntType{},
		Blocks: []*ir.Block{
			{
				Label: "entry",
				Insts: []ir.Inst{
					&ir.BinOp{Dest: temp, Op: op, L: paramA, R: paramB},
				},
				Term: &ir.Return{Value: temp},
			},
		},
	}

	return generateFunctionTest(fn)
}

// generateFunctionTest generates assembly for a test function
func generateFunctionTest(fn *ir.Function) string {
	prog := &ir.Program{Functions: []*ir.Function{fn}}
	ssaProg := ssa.Convert(prog)

	var buf bytes.Buffer
	gen := NewGenerator(&buf)
	if err := gen.Generate(ssaProg); err != nil {
		return ""
	}

	return buf.String()
}

// Benchmark tests for performance validation

func BenchmarkCodeGeneration(b *testing.B) {
	paramA := &ir.Param{Name: "a", Type: ir.IntType{}}
	paramB := &ir.Param{Name: "b", Type: ir.IntType{}}
	temp := &ir.Temp{ID: 0, Type: ir.IntType{}}

	fn := &ir.Function{
		Name:       "benchmark",
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

	prog := &ir.Program{Functions: []*ir.Function{fn}}
	ssaProg := ssa.Convert(prog)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		var buf bytes.Buffer
		gen := NewGenerator(&buf)
		_ = gen.Generate(ssaProg)
	}
}

func BenchmarkComplexFunction(b *testing.B) {
	// Create a more complex function with multiple operations
	paramA := &ir.Param{Name: "a", Type: ir.IntType{}}
	paramB := &ir.Param{Name: "b", Type: ir.IntType{}}
	paramC := &ir.Param{Name: "c", Type: ir.IntType{}}

	temps := make([]*ir.Temp, 10)
	for i := range temps {
		temps[i] = &ir.Temp{ID: i, Type: ir.IntType{}}
	}

	fn := &ir.Function{
		Name:       "complex_benchmark",
		Params:     []*ir.Param{paramA, paramB, paramC},
		ReturnType: ir.IntType{},
		Blocks: []*ir.Block{
			{
				Label: "entry",
				Insts: []ir.Inst{
					&ir.BinOp{Dest: temps[0], Op: ir.OpAdd, L: paramA, R: paramB},
					&ir.BinOp{Dest: temps[1], Op: ir.OpMul, L: temps[0], R: paramC},
					&ir.BinOp{Dest: temps[2], Op: ir.OpSub, L: temps[1], R: paramA},
					&ir.BinOp{Dest: temps[3], Op: ir.OpDiv, L: temps[2], R: paramB},
					&ir.BinOp{Dest: temps[4], Op: ir.OpEq, L: temps[3], R: paramC},
					&ir.BinOp{Dest: temps[5], Op: ir.OpAnd, L: temps[4], R: &ir.Const{Val: 1, Type: ir.IntType{}}},
				},
				Term: &ir.Return{Value: temps[5]},
			},
		},
	}

	prog := &ir.Program{Functions: []*ir.Function{fn}}
	ssaProg := ssa.Convert(prog)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		var buf bytes.Buffer
		gen := NewGenerator(&buf)
		_ = gen.Generate(ssaProg)
	}
}
