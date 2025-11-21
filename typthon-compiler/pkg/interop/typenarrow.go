// Type narrowing for union types
package interop

import (
	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
)

// TypeNarrower performs type narrowing for union types
type TypeNarrower struct {
	typeEnv map[ir.Value]ir.Type
}

// NewTypeNarrower creates a new type narrower
func NewTypeNarrower() *TypeNarrower {
	return &TypeNarrower{
		typeEnv: make(map[ir.Value]ir.Type),
	}
}

// NarrowType applies type narrowing based on control flow
func (tn *TypeNarrower) NarrowType(val ir.Value, test ir.Value, branch bool) ir.Type {
	// Get current type
	currentType := tn.typeEnv[val]
	if currentType == nil {
		return nil
	}

	// Check if current type is a union
	unionType, ok := currentType.(*ir.UnionType)
	if !ok {
		return currentType
	}

	logger.Debug("Narrowing union type", "types", len(unionType.Types), "branch", branch)

	// Narrow based on test (isinstance, comparison, etc.)
	narrowed := tn.narrowUnion(unionType, test, branch)

	if narrowed != nil {
		tn.typeEnv[val] = narrowed
	}

	return narrowed
}

func (tn *TypeNarrower) narrowUnion(union *ir.UnionType, test ir.Value, branch bool) ir.Type {
	// Extract narrowing predicate from test
	// e.g., isinstance(x, int) -> narrow to int on true branch

	// Simplified implementation - full version would analyze test expression
	if len(union.Types) == 2 {
		if branch {
			return union.Types[0] // True branch
		}
		return union.Types[1] // False branch
	}

	return union // No narrowing
}

// NarrowOnComparison narrows type based on comparison
func (tn *TypeNarrower) NarrowOnComparison(val ir.Value, op ir.Op, rhs ir.Value, branch bool) ir.Type {
	currentType := tn.typeEnv[val]
	if currentType == nil {
		return nil
	}

	logger.Debug("Narrowing on comparison", "op", op, "branch", branch)

	// Example: if x is not None -> narrow to non-None type
	if op == ir.OpNe {
		// Check if comparing against None
		if _, ok := rhs.(*ir.Const); ok {
			// Remove None from union
			if unionType, ok := currentType.(*ir.UnionType); ok {
				var newTypes []ir.Type
				for _, t := range unionType.Types {
					// TODO: check if type is None
					newTypes = append(newTypes, t)
				}
				if len(newTypes) == 1 {
					return newTypes[0]
				}
				return &ir.UnionType{Types: newTypes}
			}
		}
	}

	return currentType
}

// MergeTypes merges types from different control flow paths
func (tn *TypeNarrower) MergeTypes(types ...ir.Type) ir.Type {
	if len(types) == 0 {
		return nil
	}

	if len(types) == 1 {
		return types[0]
	}

	// Check if all types are the same
	allSame := true
	first := types[0]
	for _, t := range types[1:] {
		if !typesEqual(first, t) {
			allSame = false
			break
		}
	}

	if allSame {
		return first
	}

	// Create union type
	return &ir.UnionType{Types: types}
}

func typesEqual(a, b ir.Type) bool {
	// Simplified equality check
	switch at := a.(type) {
	case ir.IntType:
		_, ok := b.(ir.IntType)
		return ok
	case ir.BoolType:
		_, ok := b.(ir.BoolType)
		return ok
	case ir.FloatType:
		_, ok := b.(ir.FloatType)
		return ok
	case ir.StringType:
		_, ok := b.(ir.StringType)
		return ok
	case ir.ClassType:
		bt, ok := b.(ir.ClassType)
		return ok && at.Name == bt.Name
	default:
		return false
	}
}

// ApplyNarrowing applies type narrowing to a function's control flow
func ApplyNarrowing(fn *ir.Function) *ir.Function {
	narrower := NewTypeNarrower()

	for _, block := range fn.Blocks {
		for _, inst := range block.Insts {
			// Apply narrowing based on instruction type
			if binop, ok := inst.(*ir.BinOp); ok {
				// Check for comparisons that enable narrowing
				if isComparisonOp(binop.Op) {
					narrower.NarrowOnComparison(binop.L, binop.Op, binop.R, true)
				}
			}
		}
	}

	return fn
}

func isComparisonOp(op ir.Op) bool {
	return op == ir.OpEq || op == ir.OpNe || op == ir.OpLt ||
		op == ir.OpLe || op == ir.OpGt || op == ir.OpGe
}
