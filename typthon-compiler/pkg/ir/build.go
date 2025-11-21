// Package ir - AST to IR conversion
// Design: Single pass, explicit control flow, typed temporaries
package ir

import (
	"fmt"

	"github.com/GriffinCanCode/typthon-compiler/pkg/frontend"
	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
)

type Builder struct {
	prog      *Program
	currentFn *Function
	currentBl *Block
	tempID    int
	labelID   int
}

func NewBuilder() *Builder {
	return &Builder{
		prog: &Program{},
	}
}

func (b *Builder) Build(module *frontend.Module) (*Program, error) {
	logger.Debug("Building IR from AST", "statements", len(module.Body))
	for _, stmt := range module.Body {
		if fnDef, ok := stmt.(*frontend.FunctionDef); ok {
			logger.Debug("Building function", "name", fnDef.Name)
			if err := b.buildFunction(fnDef); err != nil {
				logger.Error("Failed to build function", "name", fnDef.Name, "error", err)
				return nil, err
			}
		}
	}
	logger.Info("IR build complete", "functions", len(b.prog.Functions))
	return b.prog, nil
}

func (b *Builder) buildFunction(fnDef *frontend.FunctionDef) error {
	fn := &Function{
		Name:       fnDef.Name,
		ReturnType: b.typeFromAnnotation(fnDef.Return),
	}

	// Build parameter list
	for _, param := range fnDef.Params {
		fn.Params = append(fn.Params, &Param{
			Name: param.Name,
			Type: b.typeFromAnnotation(param.Type),
		})
	}

	b.currentFn = fn
	b.prog.Functions = append(b.prog.Functions, fn)

	// Create entry block
	entry := b.newBlock("entry")
	b.currentFn.Blocks = append(b.currentFn.Blocks, entry)
	b.currentBl = entry

	// Build function body
	for _, stmt := range fnDef.Body {
		if err := b.buildStatement(stmt); err != nil {
			return err
		}
	}

	return nil
}

func (b *Builder) buildStatement(stmt frontend.Stmt) error {
	switch s := stmt.(type) {
	case *frontend.Return:
		val, err := b.buildExpression(s.Value)
		if err != nil {
			return err
		}
		b.currentBl.Term = &Return{Value: val}
		return nil
	default:
		return fmt.Errorf("unsupported statement type: %T", stmt)
	}
}

func (b *Builder) buildExpression(expr frontend.Expr) (Value, error) {
	switch e := expr.(type) {
	case *frontend.Num:
		return &Const{
			Val:  e.Value,
			Type: IntType{},
		}, nil

	case *frontend.Name:
		// Look up parameter or local variable
		for _, param := range b.currentFn.Params {
			if param.Name == e.Id {
				return param, nil
			}
		}
		return nil, fmt.Errorf("undefined variable: %s", e.Id)

	case *frontend.BinOp:
		left, err := b.buildExpression(e.Left)
		if err != nil {
			return nil, err
		}

		right, err := b.buildExpression(e.Right)
		if err != nil {
			return nil, err
		}

		// Create temporary for result
		temp := b.newTemp(IntType{})

		// Emit binary operation
		b.currentBl.Insts = append(b.currentBl.Insts, &BinOp{
			Dest: temp,
			Op:   b.opFromFrontend(e.Op),
			L:    left,
			R:    right,
		})

		return temp, nil

	case *frontend.Call:
		var args []Value
		for _, argExpr := range e.Args {
			arg, err := b.buildExpression(argExpr)
			if err != nil {
				return nil, err
			}
			args = append(args, arg)
		}

		// Create temporary for result
		temp := b.newTemp(IntType{})

		// Emit call
		b.currentBl.Insts = append(b.currentBl.Insts, &Call{
			Dest:     temp,
			Function: e.Func,
			Args:     args,
		})

		return temp, nil

	default:
		return nil, fmt.Errorf("unsupported expression type: %T", expr)
	}
}

func (b *Builder) newTemp(typ Type) *Temp {
	temp := &Temp{
		ID:   b.tempID,
		Type: typ,
	}
	b.tempID++
	return temp
}

func (b *Builder) newBlock(name string) *Block {
	label := fmt.Sprintf("%s_%d", name, b.labelID)
	b.labelID++
	return &Block{Label: label}
}

func (b *Builder) typeFromAnnotation(ann frontend.TypeAnnotation) Type {
	switch ann.Name {
	case "int":
		return IntType{}
	case "float":
		return FloatType{}
	default:
		return IntType{} // Default to int
	}
}

func (b *Builder) opFromFrontend(op frontend.Operator) Op {
	switch op {
	case frontend.Add:
		return OpAdd
	case frontend.Sub:
		return OpSub
	case frontend.Mul:
		return OpMul
	case frontend.Div:
		return OpDiv
	default:
		return OpAdd
	}
}
