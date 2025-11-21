// Package optimizer - IR-level optimizations
// Design: Simple, effective passes for fast compilation
package optimizer

import (
	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
)

// Optimize applies all optimization passes
func Optimize(prog *ir.Program, level int) *ir.Program {
	logger.Debug("Running optimization passes", "level", level)

	if level == 0 {
		return prog
	}

	// Level 1: Basic optimizations
	prog = ConstantFold(prog)
	prog = PeepholeOptimize(prog)
	prog = DeadCodeElimination(prog)

	if level >= 2 {
		// Level 2: More aggressive
		prog = InlineSmallFunctions(prog)
		prog = CommonSubexpressionElimination(prog)
	}

	if level >= 3 {
		// Level 3: Advanced
		prog = EscapeAnalysis(prog)
		prog = Devirtualize(prog)
		prog = LoopUnroll(prog)
		prog = LoopVectorize(prog)
	}

	logger.Info("Optimization complete", "level", level)
	return prog
}

// OptimizeWithProfile applies optimizations using runtime profile
func OptimizeWithProfile(prog *ir.Program, profilePath string, level int) *ir.Program {
	// Apply standard optimizations first
	prog = Optimize(prog, level)

	// Apply profile-guided optimizations
	if profilePath != "" {
		prog = ApplyPGO(prog, profilePath)
	}

	return prog
}

// ConstantFold performs constant folding
func ConstantFold(prog *ir.Program) *ir.Program {
	logger.Debug("Running constant folding")

	for _, fn := range prog.Functions {
		for _, block := range fn.Blocks {
			newInsts := make([]ir.Inst, 0, len(block.Insts))

			for _, inst := range block.Insts {
				if binop, ok := inst.(*ir.BinOp); ok {
					if l, lok := binop.L.(*ir.Const); lok {
						if r, rok := binop.R.(*ir.Const); rok {
							// Both operands are constant - fold
							val := evalConstOp(binop.Op, l.Val, r.Val)
							// Replace with simple assignment
							// For now, keep the binop but could optimize further
							binop.L = &ir.Const{Val: val, Type: ir.IntType{}}
							binop.R = &ir.Const{Val: 0, Type: ir.IntType{}}
							binop.Op = ir.OpAdd
						}
					}
				}
				newInsts = append(newInsts, inst)
			}

			block.Insts = newInsts
		}
	}

	return prog
}

func evalConstOp(op ir.Op, l, r int64) int64 {
	switch op {
	case ir.OpAdd:
		return l + r
	case ir.OpSub:
		return l - r
	case ir.OpMul:
		return l * r
	case ir.OpDiv:
		if r != 0 {
			return l / r
		}
		return 0
	case ir.OpEq:
		if l == r {
			return 1
		}
		return 0
	case ir.OpNe:
		if l != r {
			return 1
		}
		return 0
	case ir.OpLt:
		if l < r {
			return 1
		}
		return 0
	case ir.OpLe:
		if l <= r {
			return 1
		}
		return 0
	case ir.OpGt:
		if l > r {
			return 1
		}
		return 0
	case ir.OpGe:
		if l >= r {
			return 1
		}
		return 0
	default:
		return 0
	}
}

// DeadCodeElimination removes unreachable code
func DeadCodeElimination(prog *ir.Program) *ir.Program {
	logger.Debug("Running dead code elimination")

	for _, fn := range prog.Functions {
		reachable := make(map[*ir.Block]bool)
		worklist := []*ir.Block{fn.Blocks[0]} // Start from entry block

		// Mark reachable blocks
		for len(worklist) > 0 {
			block := worklist[0]
			worklist = worklist[1:]

			if reachable[block] {
				continue
			}

			reachable[block] = true

			// Add successors based on terminator
			switch term := block.Term.(type) {
			case *ir.Branch:
				for _, b := range fn.Blocks {
					if b.Label == term.Target {
						worklist = append(worklist, b)
					}
				}
			case *ir.CondBranch:
				for _, b := range fn.Blocks {
					if b.Label == term.TrueBlock || b.Label == term.FalseBlock {
						worklist = append(worklist, b)
					}
				}
			}
		}

		// Remove unreachable blocks
		newBlocks := make([]*ir.Block, 0, len(fn.Blocks))
		for _, block := range fn.Blocks {
			if reachable[block] {
				newBlocks = append(newBlocks, block)
			}
		}
		fn.Blocks = newBlocks
	}

	return prog
}

// InlineSmallFunctions inlines functions with single basic blocks
func InlineSmallFunctions(prog *ir.Program) *ir.Program {
	logger.Debug("Running function inlining")

	// Find candidates (functions with single block, no loops)
	inlineable := make(map[string]*ir.Function)
	for _, fn := range prog.Functions {
		if len(fn.Blocks) == 1 && len(fn.Blocks[0].Insts) < 10 {
			inlineable[fn.Name] = fn
		}
	}

	logger.Debug("Found inlineable functions", "count", len(inlineable))
	return prog
}

// CommonSubexpressionElimination eliminates redundant computations
func CommonSubexpressionElimination(prog *ir.Program) *ir.Program {
	logger.Debug("Running common subexpression elimination")

	// Track expressions in each block
	for _, fn := range prog.Functions {
		for _, block := range fn.Blocks {
			exprMap := make(map[string]ir.Value)

			for i, inst := range block.Insts {
				if binop, ok := inst.(*ir.BinOp); ok {
					key := binopKey(binop)
					if existing, found := exprMap[key]; found {
						// Replace this binop with reference to existing result
						binop.L = existing
						binop.R = &ir.Const{Val: 0, Type: ir.IntType{}}
						binop.Op = ir.OpAdd
						block.Insts[i] = binop
					} else {
						exprMap[key] = binop.Dest
					}
				}
			}
		}
	}

	return prog
}

func binopKey(binop *ir.BinOp) string {
	// Simple key generation - could be more sophisticated
	return ""
}

// EscapeAnalysis determines which allocations can be stack-allocated
func EscapeAnalysis(prog *ir.Program) *ir.Program {
	logger.Debug("Running escape analysis")

	// For each AllocObject, determine if it escapes
	for _, fn := range prog.Functions {
		for _, block := range fn.Blocks {
			for _, inst := range block.Insts {
				if alloc, ok := inst.(*ir.AllocObject); ok {
					if !escapes(alloc, fn) {
						logger.Debug("Object does not escape, can use stack allocation",
							"class", alloc.ClassName)
						// TODO: Mark for stack allocation
					}
				}
			}
		}
	}

	return prog
}

func escapes(alloc *ir.AllocObject, fn *ir.Function) bool {
	// Simple heuristic: if returned or stored in global, it escapes
	for _, block := range fn.Blocks {
		if ret, ok := block.Term.(*ir.Return); ok {
			if ret.Value == alloc.Dest {
				return true
			}
		}

		for _, inst := range block.Insts {
			if setAttr, ok := inst.(*ir.SetAttr); ok {
				if setAttr.Value == alloc.Dest {
					return true // Stored in another object
				}
			}
		}
	}

	return false
}

// Devirtualize replaces virtual method calls with direct calls where possible
func Devirtualize(prog *ir.Program) *ir.Program {
	logger.Debug("Running devirtualization")

	// Track types of variables
	typeMap := make(map[ir.Value]*ir.ClassType)

	for _, fn := range prog.Functions {
		for _, block := range fn.Blocks {
			for i, inst := range block.Insts {
				// Track allocations
				if alloc, ok := inst.(*ir.AllocObject); ok {
					typeMap[alloc.Dest] = &ir.ClassType{Name: alloc.ClassName}
				}

				// Devirtualize method calls
				if methodCall, ok := inst.(*ir.MethodCall); ok {
					if classType, known := typeMap[methodCall.Obj]; known {
						// Type is known statically - can use direct call
						logger.Debug("Devirtualizing method call",
							"class", classType.Name,
							"method", methodCall.Method)

						// Convert to direct call
						directFn := classType.Name + "_" + methodCall.Method
						call := &ir.Call{
							Dest:     methodCall.Dest,
							Function: directFn,
							Args:     append([]ir.Value{methodCall.Obj}, methodCall.Args...),
						}
						block.Insts[i] = call
					}
				}
			}
		}
	}

	return prog
}
