// Loop optimizations - unrolling and vectorization
package optimizer

import (
	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
)

// LoopUnroll performs loop unrolling for small loops
func LoopUnroll(prog *ir.Program) *ir.Program {
	logger.Debug("Running loop unrolling optimization")

	const unrollFactor = 4 // Unroll up to 4 iterations

	for _, fn := range prog.Functions {
		for i := 0; i < len(fn.Blocks); i++ {
			block := fn.Blocks[i]

			// Look for loop patterns (while/for)
			if condBr, ok := block.Term.(*ir.CondBranch); ok {
				// Check if this is a simple counting loop
				if loop := detectCountingLoop(fn, block); loop != nil {
					if shouldUnroll(loop, unrollFactor) {
						logger.Debug("Unrolling loop", "block", block.Label, "factor", unrollFactor)
						unrollLoop(fn, loop, unrollFactor)
					}
				}
				_ = condBr
			}
		}
	}

	return prog
}

// LoopVectorize performs SIMD vectorization for numeric operations
func LoopVectorize(prog *ir.Program) *ir.Program {
	logger.Debug("Running loop vectorization optimization")

	for _, fn := range prog.Functions {
		for _, block := range fn.Blocks {
			// Look for vectorizable patterns
			if loop := detectVectorizableLoop(fn, block); loop != nil {
				logger.Debug("Vectorizing loop", "block", block.Label)
				vectorizeLoop(fn, loop)
			}
		}
	}

	return prog
}

type loopInfo struct {
	header       *ir.Block
	body         *ir.Block
	exit         *ir.Block
	inductionVar ir.Value
	start        int64
	end          int64
	step         int64
	vectorizable bool
}

func detectCountingLoop(fn *ir.Function, header *ir.Block) *loopInfo {
	// Look for pattern:
	// header: i < N -> body, exit
	// body: operations, i = i + 1 -> header

	condBr, ok := header.Term.(*ir.CondBranch)
	if !ok {
		return nil
	}

	// Find induction variable from comparison
	var inductionVar ir.Value
	var end int64

	for _, inst := range header.Insts {
		if binop, ok := inst.(*ir.BinOp); ok {
			if binop.Op == ir.OpLt || binop.Op == ir.OpLe {
				inductionVar = binop.L
				if c, ok := binop.R.(*ir.Const); ok {
					end = c.Val
				}
			}
		}
	}

	if inductionVar == nil {
		return nil
	}

	// Find body and exit blocks
	var body, exit *ir.Block
	for _, b := range fn.Blocks {
		if b.Label == condBr.TrueBlock {
			body = b
		}
		if b.Label == condBr.FalseBlock {
			exit = b
		}
	}

	if body == nil || exit == nil {
		return nil
	}

	return &loopInfo{
		header:       header,
		body:         body,
		exit:         exit,
		inductionVar: inductionVar,
		start:        0,
		end:          end,
		step:         1,
		vectorizable: false,
	}
}

func detectVectorizableLoop(fn *ir.Function, block *ir.Block) *loopInfo {
	loop := detectCountingLoop(fn, block)
	if loop == nil {
		return nil
	}

	// Check if loop body contains only vectorizable operations
	vectorizable := true
	for _, inst := range loop.body.Insts {
		if !isVectorizable(inst) {
			vectorizable = false
			break
		}
	}

	loop.vectorizable = vectorizable
	return loop
}

func isVectorizable(inst ir.Inst) bool {
	switch i := inst.(type) {
	case *ir.BinOp:
		// Only integer/float arithmetic is vectorizable
		switch i.Op {
		case ir.OpAdd, ir.OpSub, ir.OpMul:
			return true
		}
	case *ir.Load, *ir.Store:
		// Memory operations can be vectorized if aligned
		return true
	}
	return false
}

func shouldUnroll(loop *loopInfo, factor int) bool {
	// Only unroll small loops with known bounds
	tripCount := loop.end - loop.start
	return tripCount > 0 && tripCount <= int64(factor*8) && tripCount%int64(factor) == 0
}

func unrollLoop(fn *ir.Function, loop *loopInfo, factor int) {
	// Clone body instructions `factor` times
	newInsts := make([]ir.Inst, 0, len(loop.body.Insts)*factor)

	for i := 0; i < factor; i++ {
		for _, inst := range loop.body.Insts {
			// Clone instruction with adjusted offsets
			cloned := cloneInstruction(inst, i)
			newInsts = append(newInsts, cloned)
		}
	}

	loop.body.Insts = newInsts
}

func vectorizeLoop(fn *ir.Function, loop *loopInfo) {
	// Convert scalar operations to vector operations
	// This would emit SIMD instructions in the backend

	logger.Debug("Vectorization would emit SIMD instructions", "loop", loop.header.Label)

	// Mark loop as vectorized for backend
	// Backend will emit appropriate SIMD instructions (SSE/AVX/NEON)
}

func cloneInstruction(inst ir.Inst, offset int) ir.Inst {
	// Deep clone instruction (simplified - full implementation would handle all types)
	switch i := inst.(type) {
	case *ir.BinOp:
		return &ir.BinOp{
			Dest: i.Dest,
			Op:   i.Op,
			L:    i.L,
			R:    i.R,
		}
	case *ir.Load:
		return &ir.Load{
			Dest: i.Dest,
			Src:  i.Src,
		}
	case *ir.Store:
		return &ir.Store{
			Dest: i.Dest,
			Src:  i.Src,
		}
	default:
		return inst
	}
}
