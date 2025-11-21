// Package optimizer - Peephole optimization pass
// Recognizes and optimizes common instruction patterns
package optimizer

import (
	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
)

// PeepholeOptimize applies pattern-based peephole optimizations
func PeepholeOptimize(prog *ir.Program) *ir.Program {
	logger.Debug("Running peephole optimizer")

	for _, fn := range prog.Functions {
		for _, block := range fn.Blocks {
			block.Insts = optimizeInstSequence(block.Insts)
		}
	}

	logger.Info("Peephole optimization complete")
	return prog
}

// optimizeInstSequence optimizes a sequence of instructions
func optimizeInstSequence(insts []ir.Inst) []ir.Inst {
	if len(insts) == 0 {
		return insts
	}

	result := make([]ir.Inst, 0, len(insts))
	i := 0

	for i < len(insts) {
		// Try two-instruction patterns first
		if i+1 < len(insts) {
			if optimized := tryTwoInstPattern(insts[i], insts[i+1]); optimized != nil {
				result = append(result, optimized...)
				i += 2
				continue
			}
		}

		// Try single-instruction patterns
		if optimized := trySingleInstPattern(insts[i]); optimized != nil {
			result = append(result, optimized)
			i++
			continue
		}

		// No optimization found, keep original
		result = append(result, insts[i])
		i++
	}

	return result
}

// trySingleInstPattern tries to optimize a single instruction
func trySingleInstPattern(inst ir.Inst) ir.Inst {
	binop, ok := inst.(*ir.BinOp)
	if !ok {
		return inst
	}

	// Pattern: x = a + 0  =>  x = a
	if binop.Op == ir.OpAdd {
		if c, ok := binop.R.(*ir.Const); ok && c.Val == 0 {
			logger.Debug("Peephole: eliminated add-by-zero")
			return &ir.Load{Dest: binop.Dest, Src: binop.L}
		}
		if c, ok := binop.L.(*ir.Const); ok && c.Val == 0 {
			logger.Debug("Peephole: eliminated add-by-zero")
			return &ir.Load{Dest: binop.Dest, Src: binop.R}
		}
	}

	// Pattern: x = a - 0  =>  x = a
	if binop.Op == ir.OpSub {
		if c, ok := binop.R.(*ir.Const); ok && c.Val == 0 {
			logger.Debug("Peephole: eliminated subtract-by-zero")
			return &ir.Load{Dest: binop.Dest, Src: binop.L}
		}
	}

	// Pattern: x = a * 0  =>  x = 0
	if binop.Op == ir.OpMul {
		if c, ok := binop.R.(*ir.Const); ok && c.Val == 0 {
			logger.Debug("Peephole: eliminated multiply-by-zero")
			return &ir.Load{Dest: binop.Dest, Src: &ir.Const{Val: 0, Type: binop.L.(*ir.Temp).Type}}
		}
		if c, ok := binop.L.(*ir.Const); ok && c.Val == 0 {
			logger.Debug("Peephole: eliminated multiply-by-zero")
			return &ir.Load{Dest: binop.Dest, Src: &ir.Const{Val: 0, Type: binop.R.(*ir.Temp).Type}}
		}
	}

	// Pattern: x = a * 1  =>  x = a
	if binop.Op == ir.OpMul {
		if c, ok := binop.R.(*ir.Const); ok && c.Val == 1 {
			logger.Debug("Peephole: eliminated multiply-by-one")
			return &ir.Load{Dest: binop.Dest, Src: binop.L}
		}
		if c, ok := binop.L.(*ir.Const); ok && c.Val == 1 {
			logger.Debug("Peephole: eliminated multiply-by-one")
			return &ir.Load{Dest: binop.Dest, Src: binop.R}
		}
	}

	// Pattern: x = a * 2  =>  x = a + a (faster on some architectures)
	if binop.Op == ir.OpMul {
		if c, ok := binop.R.(*ir.Const); ok && c.Val == 2 {
			logger.Debug("Peephole: converted multiply-by-2 to add")
			return &ir.BinOp{Dest: binop.Dest, Op: ir.OpAdd, L: binop.L, R: binop.L}
		}
		if c, ok := binop.L.(*ir.Const); ok && c.Val == 2 {
			logger.Debug("Peephole: converted multiply-by-2 to add")
			return &ir.BinOp{Dest: binop.Dest, Op: ir.OpAdd, L: binop.R, R: binop.R}
		}
	}

	// Pattern: x = a * power_of_2  =>  x = a << log2(n) (shift is faster)
	if binop.Op == ir.OpMul {
		if c, ok := binop.R.(*ir.Const); ok && isPowerOfTwo(c.Val) {
			shift := log2(c.Val)
			logger.Debug("Peephole: converted multiply to shift", "value", c.Val, "shift", shift)
			// Note: Would need shift instruction in IR, keeping multiplication for now
		}
	}

	// Pattern: x = a / 1  =>  x = a
	if binop.Op == ir.OpDiv {
		if c, ok := binop.R.(*ir.Const); ok && c.Val == 1 {
			logger.Debug("Peephole: eliminated divide-by-one")
			return &ir.Load{Dest: binop.Dest, Src: binop.L}
		}
	}

	// Pattern: x = a & 0  =>  x = 0
	if binop.Op == ir.OpAnd {
		if c, ok := binop.R.(*ir.Const); ok && c.Val == 0 {
			logger.Debug("Peephole: eliminated and-with-zero")
			return &ir.Load{Dest: binop.Dest, Src: &ir.Const{Val: 0, Type: ir.IntType{}}}
		}
		if c, ok := binop.L.(*ir.Const); ok && c.Val == 0 {
			logger.Debug("Peephole: eliminated and-with-zero")
			return &ir.Load{Dest: binop.Dest, Src: &ir.Const{Val: 0, Type: ir.IntType{}}}
		}
	}

	// Pattern: x = a | 0  =>  x = a
	if binop.Op == ir.OpOr {
		if c, ok := binop.R.(*ir.Const); ok && c.Val == 0 {
			logger.Debug("Peephole: eliminated or-with-zero")
			return &ir.Load{Dest: binop.Dest, Src: binop.L}
		}
		if c, ok := binop.L.(*ir.Const); ok && c.Val == 0 {
			logger.Debug("Peephole: eliminated or-with-zero")
			return &ir.Load{Dest: binop.Dest, Src: binop.R}
		}
	}

	// Pattern: x = a ^ 0  =>  x = a
	if binop.Op == ir.OpXor {
		if c, ok := binop.R.(*ir.Const); ok && c.Val == 0 {
			logger.Debug("Peephole: eliminated xor-with-zero")
			return &ir.Load{Dest: binop.Dest, Src: binop.L}
		}
		if c, ok := binop.L.(*ir.Const); ok && c.Val == 0 {
			logger.Debug("Peephole: eliminated xor-with-zero")
			return &ir.Load{Dest: binop.Dest, Src: binop.R}
		}
	}

	return inst
}

// tryTwoInstPattern tries to optimize a pair of instructions
func tryTwoInstPattern(inst1, inst2 ir.Inst) []ir.Inst {
	// Pattern: load followed by load of same value
	if load1, ok := inst1.(*ir.Load); ok {
		if load2, ok := inst2.(*ir.Load); ok {
			if load1.Src == load2.Src {
				logger.Debug("Peephole: eliminated redundant load")
				// Replace second load with copy from first dest
				return []ir.Inst{
					load1,
					&ir.Load{Dest: load2.Dest, Src: load1.Dest},
				}
			}
		}
	}

	// Pattern: x = a op b; y = x  =>  y = a op b (if x not used elsewhere)
	if binop, ok := inst1.(*ir.BinOp); ok {
		if load, ok := inst2.(*ir.Load); ok {
			if load.Src == binop.Dest {
				logger.Debug("Peephole: eliminated intermediate load")
				return []ir.Inst{
					&ir.BinOp{Dest: load.Dest, Op: binop.Op, L: binop.L, R: binop.R},
				}
			}
		}
	}

	// Pattern: store followed by load of same location
	if store, ok := inst1.(*ir.Store); ok {
		if load, ok := inst2.(*ir.Load); ok {
			if store.Dest == load.Src {
				logger.Debug("Peephole: forwarded store to load")
				return []ir.Inst{
					store,
					&ir.Load{Dest: load.Dest, Src: store.Src},
				}
			}
		}
	}

	return nil
}

// isPowerOfTwo checks if n is a power of 2
func isPowerOfTwo(n int64) bool {
	return n > 0 && (n&(n-1)) == 0
}

// log2 returns log2 of n (assumes n is power of 2)
func log2(n int64) int {
	shift := 0
	for n > 1 {
		n >>= 1
		shift++
	}
	return shift
}
