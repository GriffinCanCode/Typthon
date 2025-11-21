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
	locals    map[string]Value // Track local variables
	loopStack []loopContext    // Track nested loops for break/continue
}

type loopContext struct {
	breakLabel    string
	continueLabel string
}

func NewBuilder() *Builder {
	return &Builder{
		prog:   &Program{},
		locals: make(map[string]Value),
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

	// Reset builder state for new function
	b.locals = make(map[string]Value)
	b.loopStack = nil

	// Build parameter list and register them as locals
	for _, param := range fnDef.Params {
		p := &Param{
			Name: param.Name,
			Type: b.typeFromAnnotation(param.Type),
		}
		fn.Params = append(fn.Params, p)
		b.locals[param.Name] = p
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

	// Ensure last block has terminator
	if b.currentBl != nil && b.currentBl.Term == nil {
		// Add implicit return for void functions
		b.currentBl.Term = &Return{Value: nil}
	}

	return nil
}

func (b *Builder) buildStatement(stmt frontend.Stmt) error {
	if b.currentBl == nil || b.currentBl.Term != nil {
		// Skip dead code after terminator
		return nil
	}

	switch s := stmt.(type) {
	case *frontend.Return:
		val, err := b.buildExpression(s.Value)
		if err != nil {
			return err
		}
		b.currentBl.Term = &Return{Value: val}
		return nil

	case *frontend.Assign:
		val, err := b.buildExpression(s.Value)
		if err != nil {
			return err
		}
		// Store in locals map
		b.locals[s.Target] = val
		return nil

	case *frontend.If:
		return b.buildIf(s)

	case *frontend.While:
		return b.buildWhile(s)

	case *frontend.For:
		return b.buildFor(s)

	case *frontend.Break:
		if len(b.loopStack) == 0 {
			return fmt.Errorf("break outside loop")
		}
		ctx := b.loopStack[len(b.loopStack)-1]
		b.currentBl.Term = &Branch{Target: ctx.breakLabel}
		return nil

	case *frontend.Continue:
		if len(b.loopStack) == 0 {
			return fmt.Errorf("continue outside loop")
		}
		ctx := b.loopStack[len(b.loopStack)-1]
		b.currentBl.Term = &Branch{Target: ctx.continueLabel}
		return nil

	case *frontend.Pass:
		// No-op
		return nil

	default:
		return fmt.Errorf("unsupported statement type: %T", stmt)
	}
}

func (b *Builder) buildExpression(expr frontend.Expr) (Value, error) {
	switch e := expr.(type) {
	case *frontend.Num:
		return &Const{Val: e.Value, Type: IntType{}}, nil

	case *frontend.Bool:
		val := int64(0)
		if e.Value {
			val = 1
		}
		return &Const{Val: val, Type: BoolType{}}, nil

	case *frontend.Name:
		// Look up in locals first, then parameters
		if val, ok := b.locals[e.Id]; ok {
			return val, nil
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

		temp := b.newTemp(IntType{})
		b.currentBl.Insts = append(b.currentBl.Insts, &BinOp{
			Dest: temp,
			Op:   b.opFromFrontend(e.Op),
			L:    left,
			R:    right,
		})
		return temp, nil

	case *frontend.Compare:
		left, err := b.buildExpression(e.Left)
		if err != nil {
			return nil, err
		}
		right, err := b.buildExpression(e.Right)
		if err != nil {
			return nil, err
		}

		temp := b.newTemp(BoolType{})
		b.currentBl.Insts = append(b.currentBl.Insts, &BinOp{
			Dest: temp,
			Op:   b.compareOpToIR(e.Op),
			L:    left,
			R:    right,
		})
		return temp, nil

	case *frontend.BoolOp:
		// Short-circuit evaluation for and/or
		return b.buildBoolOp(e)

	case *frontend.UnaryOp:
		operand, err := b.buildExpression(e.Expr)
		if err != nil {
			return nil, err
		}
		if e.Op == frontend.Not {
			// Implement not as XOR with 1
			temp := b.newTemp(BoolType{})
			one := &Const{Val: 1, Type: BoolType{}}
			b.currentBl.Insts = append(b.currentBl.Insts, &BinOp{
				Dest: temp,
				Op:   OpXor,
				L:    operand,
				R:    one,
			})
			return temp, nil
		}
		return nil, fmt.Errorf("unsupported unary operator: %v", e.Op)

	case *frontend.Call:
		var args []Value
		for _, argExpr := range e.Args {
			arg, err := b.buildExpression(argExpr)
			if err != nil {
				return nil, err
			}
			args = append(args, arg)
		}

		temp := b.newTemp(IntType{})
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
	case "bool":
		return BoolType{}
	case "float":
		return FloatType{}
	default:
		return IntType{} // Default to int
	}
}

func (b *Builder) buildIf(ifStmt *frontend.If) error {
	cond, err := b.buildExpression(ifStmt.Cond)
	if err != nil {
		return err
	}

	thenBlock := b.newBlock("then")
	mergeBlock := b.newBlock("merge")
	elseBlock := mergeBlock

	// Handle elif/else chains
	var elifBlocks []*Block
	for range ifStmt.Elif {
		elifBlocks = append(elifBlocks, b.newBlock("elif"))
	}
	if len(ifStmt.Else) > 0 {
		elseBlock = b.newBlock("else")
	} else if len(ifStmt.Elif) > 0 {
		elseBlock = elifBlocks[0]
	}

	// Emit conditional branch
	b.currentBl.Term = &CondBranch{
		Cond:       cond,
		TrueBlock:  thenBlock.Label,
		FalseBlock: elseBlock.Label,
	}

	// Build then block
	b.currentFn.Blocks = append(b.currentFn.Blocks, thenBlock)
	b.currentBl = thenBlock
	for _, stmt := range ifStmt.Then {
		if err := b.buildStatement(stmt); err != nil {
			return err
		}
	}
	if b.currentBl.Term == nil {
		b.currentBl.Term = &Branch{Target: mergeBlock.Label}
	}

	// Build elif chains
	for i, elif := range ifStmt.Elif {
		b.currentFn.Blocks = append(b.currentFn.Blocks, elifBlocks[i])
		b.currentBl = elifBlocks[i]

		elifCond, err := b.buildExpression(elif.Cond)
		if err != nil {
			return err
		}

		elifThenBlock := b.newBlock("elif_then")
		var elifElseBlock *Block
		if i+1 < len(elifBlocks) {
			elifElseBlock = elifBlocks[i+1]
		} else if len(ifStmt.Else) > 0 {
			elifElseBlock = b.newBlock("else")
		} else {
			elifElseBlock = mergeBlock
		}

		b.currentBl.Term = &CondBranch{
			Cond:       elifCond,
			TrueBlock:  elifThenBlock.Label,
			FalseBlock: elifElseBlock.Label,
		}

		b.currentFn.Blocks = append(b.currentFn.Blocks, elifThenBlock)
		b.currentBl = elifThenBlock
		for _, stmt := range elif.Body {
			if err := b.buildStatement(stmt); err != nil {
				return err
			}
		}
		if b.currentBl.Term == nil {
			b.currentBl.Term = &Branch{Target: mergeBlock.Label}
		}

		if i == len(elifBlocks)-1 && len(ifStmt.Else) > 0 {
			elseBlock = elifElseBlock
		}
	}

	// Build else block
	if len(ifStmt.Else) > 0 {
		b.currentFn.Blocks = append(b.currentFn.Blocks, elseBlock)
		b.currentBl = elseBlock
		for _, stmt := range ifStmt.Else {
			if err := b.buildStatement(stmt); err != nil {
				return err
			}
		}
		if b.currentBl.Term == nil {
			b.currentBl.Term = &Branch{Target: mergeBlock.Label}
		}
	}

	// Set merge block as current
	b.currentFn.Blocks = append(b.currentFn.Blocks, mergeBlock)
	b.currentBl = mergeBlock
	return nil
}

func (b *Builder) buildWhile(whileStmt *frontend.While) error {
	headerBlock := b.newBlock("while_header")
	bodyBlock := b.newBlock("while_body")
	exitBlock := b.newBlock("while_exit")

	// Jump to header
	b.currentBl.Term = &Branch{Target: headerBlock.Label}

	// Build header (condition check)
	b.currentFn.Blocks = append(b.currentFn.Blocks, headerBlock)
	b.currentBl = headerBlock
	cond, err := b.buildExpression(whileStmt.Cond)
	if err != nil {
		return err
	}
	b.currentBl.Term = &CondBranch{
		Cond:       cond,
		TrueBlock:  bodyBlock.Label,
		FalseBlock: exitBlock.Label,
	}

	// Build body
	b.currentFn.Blocks = append(b.currentFn.Blocks, bodyBlock)
	b.currentBl = bodyBlock
	b.loopStack = append(b.loopStack, loopContext{
		breakLabel:    exitBlock.Label,
		continueLabel: headerBlock.Label,
	})
	for _, stmt := range whileStmt.Body {
		if err := b.buildStatement(stmt); err != nil {
			return err
		}
	}
	b.loopStack = b.loopStack[:len(b.loopStack)-1]
	if b.currentBl.Term == nil {
		b.currentBl.Term = &Branch{Target: headerBlock.Label}
	}

	// Set exit block as current
	b.currentFn.Blocks = append(b.currentFn.Blocks, exitBlock)
	b.currentBl = exitBlock
	return nil
}

func (b *Builder) buildFor(forStmt *frontend.For) error {
	// For now, implement range-based for loops
	// TODO: Add proper iterator support
	headerBlock := b.newBlock("for_header")
	bodyBlock := b.newBlock("for_body")
	exitBlock := b.newBlock("for_exit")

	// Evaluate iterator (should be range() call)
	iterVal, err := b.buildExpression(forStmt.Iter)
	if err != nil {
		return err
	}

	// Initialize loop variable
	loopVar := b.newTemp(IntType{})
	b.locals[forStmt.Target] = loopVar

	// Jump to header
	b.currentBl.Term = &Branch{Target: headerBlock.Label}

	// Build header (condition check)
	b.currentFn.Blocks = append(b.currentFn.Blocks, headerBlock)
	b.currentBl = headerBlock

	// For now, simple implementation - proper iterator in Phase 3
	cond := b.newTemp(BoolType{})
	b.currentBl.Insts = append(b.currentBl.Insts, &BinOp{
		Dest: cond,
		Op:   OpLt,
		L:    loopVar,
		R:    iterVal,
	})
	b.currentBl.Term = &CondBranch{
		Cond:       cond,
		TrueBlock:  bodyBlock.Label,
		FalseBlock: exitBlock.Label,
	}

	// Build body
	b.currentFn.Blocks = append(b.currentFn.Blocks, bodyBlock)
	b.currentBl = bodyBlock
	b.loopStack = append(b.loopStack, loopContext{
		breakLabel:    exitBlock.Label,
		continueLabel: headerBlock.Label,
	})
	for _, stmt := range forStmt.Body {
		if err := b.buildStatement(stmt); err != nil {
			return err
		}
	}
	b.loopStack = b.loopStack[:len(b.loopStack)-1]

	// Increment loop variable
	one := &Const{Val: 1, Type: IntType{}}
	nextVar := b.newTemp(IntType{})
	b.currentBl.Insts = append(b.currentBl.Insts, &BinOp{
		Dest: nextVar,
		Op:   OpAdd,
		L:    loopVar,
		R:    one,
	})
	b.locals[forStmt.Target] = nextVar

	if b.currentBl.Term == nil {
		b.currentBl.Term = &Branch{Target: headerBlock.Label}
	}

	// Set exit block as current
	b.currentFn.Blocks = append(b.currentFn.Blocks, exitBlock)
	b.currentBl = exitBlock
	return nil
}

func (b *Builder) buildBoolOp(boolOp *frontend.BoolOp) (Value, error) {
	// Short-circuit evaluation
	if boolOp.Op == frontend.And {
		// and: if left is false, result is false; else eval right
		left, err := b.buildExpression(boolOp.Left)
		if err != nil {
			return nil, err
		}

		rightBlock := b.newBlock("and_right")
		mergeBlock := b.newBlock("and_merge")
		result := b.newTemp(BoolType{})

		b.currentBl.Term = &CondBranch{
			Cond:       left,
			TrueBlock:  rightBlock.Label,
			FalseBlock: mergeBlock.Label,
		}

		// Right evaluation
		b.currentFn.Blocks = append(b.currentFn.Blocks, rightBlock)
		b.currentBl = rightBlock
		right, err := b.buildExpression(boolOp.Right)
		if err != nil {
			return nil, err
		}
		b.currentBl.Term = &Branch{Target: mergeBlock.Label}

		// Merge (will need phi node in SSA)
		b.currentFn.Blocks = append(b.currentFn.Blocks, mergeBlock)
		b.currentBl = mergeBlock

		// For now, emit simple logic (phi nodes in SSA phase)
		b.currentBl.Insts = append(b.currentBl.Insts, &BinOp{
			Dest: result,
			Op:   OpAnd,
			L:    left,
			R:    right,
		})
		return result, nil
	} else {
		// or: if left is true, result is true; else eval right
		left, err := b.buildExpression(boolOp.Left)
		if err != nil {
			return nil, err
		}

		rightBlock := b.newBlock("or_right")
		mergeBlock := b.newBlock("or_merge")
		result := b.newTemp(BoolType{})

		b.currentBl.Term = &CondBranch{
			Cond:       left,
			TrueBlock:  mergeBlock.Label,
			FalseBlock: rightBlock.Label,
		}

		// Right evaluation
		b.currentFn.Blocks = append(b.currentFn.Blocks, rightBlock)
		b.currentBl = rightBlock
		right, err := b.buildExpression(boolOp.Right)
		if err != nil {
			return nil, err
		}
		b.currentBl.Term = &Branch{Target: mergeBlock.Label}

		// Merge
		b.currentFn.Blocks = append(b.currentFn.Blocks, mergeBlock)
		b.currentBl = mergeBlock

		b.currentBl.Insts = append(b.currentBl.Insts, &BinOp{
			Dest: result,
			Op:   OpOr,
			L:    left,
			R:    right,
		})
		return result, nil
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

func (b *Builder) compareOpToIR(op frontend.CompareOp) Op {
	switch op {
	case frontend.Eq:
		return OpEq
	case frontend.Ne:
		return OpNe
	case frontend.Lt:
		return OpLt
	case frontend.Le:
		return OpLe
	case frontend.Gt:
		return OpGt
	case frontend.Ge:
		return OpGe
	default:
		return OpEq
	}
}
