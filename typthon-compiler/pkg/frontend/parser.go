// Package frontend - Recursive descent parser for minimal Python subset
// Design: Predictive parsing, clear error messages, zero backtracking
package frontend

import (
	"fmt"
)

type Parser struct {
	slexer  *SimpleLexer
	current Token
	errors  []string
}

func NewParser(source string) *Parser {
	slexer := NewSimpleLexer(source)
	return &Parser{
		slexer:  slexer,
		current: slexer.NextToken(),
	}
}

func (p *Parser) Parse() (*Module, error) {
	module := &Module{}

	// Skip initial newlines
	for p.match(NEWLINE) {
		p.advance()
	}

	// Parse top-level statements (functions and classes)
	for !p.check(EOF) {
		// Skip any DEDENT tokens at module level
		for p.match(DEDENT) {
			p.advance()
		}

		if p.check(EOF) {
			break
		}

		var stmt Stmt
		if p.match(DEF) {
			stmt = p.function()
		} else if p.match(CLASS) {
			stmt = p.class()
		}

		if stmt != nil {
			module.Body = append(module.Body, stmt)
		}

		// Skip newlines between definitions
		for p.match(NEWLINE) {
			p.advance()
		}
	}

	if len(p.errors) > 0 {
		return nil, fmt.Errorf("parse errors: %v", p.errors)
	}

	return module, nil
}

func (p *Parser) class() Stmt {
	p.advance() // consume 'class'

	if !p.check(NAME) {
		p.error("expected class name")
		return nil
	}
	name := p.current.Lexeme
	p.advance()

	// Parse base classes (optional)
	var bases []string
	if p.match(LPAREN) {
		p.advance()
		if !p.check(RPAREN) {
			for {
				if !p.check(NAME) {
					p.error("expected base class name")
					break
				}
				bases = append(bases, p.current.Lexeme)
				p.advance()
				if !p.match(COMMA) {
					break
				}
				p.advance()
			}
		}
		if !p.consume(RPAREN, "expected ')'") {
			return nil
		}
	}

	if !p.consume(COLON, "expected ':'") {
		return nil
	}

	if !p.consume(NEWLINE, "expected newline after ':'") {
		return nil
	}

	if !p.consume(INDENT, "expected indent") {
		return nil
	}

	// Parse class body (methods and attributes)
	var methods []*FunctionDef
	var attrs []Assign

	for !p.check(DEDENT) && !p.check(EOF) {
		if p.match(DEF) {
			if method := p.function(); method != nil {
				if fnDef, ok := method.(*FunctionDef); ok {
					methods = append(methods, fnDef)
				}
			}
		} else if p.check(NAME) {
			// Class attribute assignment
			start := p.current
			p.advance()
			if p.match(ASSIGN) {
				p.advance()
				value := p.expression()
				if p.match(NEWLINE) {
					p.advance()
				}
				attrs = append(attrs, Assign{Target: start.Lexeme, Value: value})
			}
		} else if p.match(PASS) {
			p.advance()
			if p.match(NEWLINE) {
				p.advance()
			}
		} else {
			p.error("expected method or attribute definition")
			break
		}

		// Allow multiple newlines in body
		for p.match(NEWLINE) {
			p.advance()
		}
	}

	if !p.consume(DEDENT, "expected dedent") {
		return nil
	}

	return &ClassDef{
		Name:    name,
		Bases:   bases,
		Methods: methods,
		Attrs:   attrs,
	}
}

func (p *Parser) function() Stmt {
	if !p.consume(DEF, "expected 'def'") {
		return nil
	}

	if !p.check(NAME) {
		p.error("expected function name")
		return nil
	}
	name := p.current.Lexeme
	p.advance()

	if !p.consume(LPAREN, "expected '('") {
		return nil
	}

	// Parse parameters
	var params []Param
	if !p.check(RPAREN) {
		params = p.parameters()
	}

	if !p.consume(RPAREN, "expected ')'") {
		return nil
	}

	// Parse return type annotation
	var returnType TypeAnnotation
	if p.match(ARROW) {
		p.advance()
		returnType = p.typeAnnotation()
	}

	if !p.consume(COLON, "expected ':'") {
		return nil
	}

	if !p.consume(NEWLINE, "expected newline after ':'") {
		return nil
	}

	if !p.consume(INDENT, "expected indent") {
		return nil
	}

	// Parse function body
	var body []Stmt
	for !p.check(DEDENT) && !p.check(EOF) {
		if stmt := p.statement(); stmt != nil {
			body = append(body, stmt)
		}

		// Allow multiple newlines in body
		for p.match(NEWLINE) {
			p.advance()
		}
	}

	if !p.consume(DEDENT, "expected dedent") {
		return nil
	}

	return &FunctionDef{
		Name:   name,
		Params: params,
		Body:   body,
		Return: returnType,
	}
}

func (p *Parser) parameters() []Param {
	var params []Param

	for {
		if !p.check(NAME) {
			p.error("expected parameter name")
			break
		}
		paramName := p.current.Lexeme
		p.advance()

		// Type annotation
		var paramType TypeAnnotation
		if p.match(COLON) {
			p.advance()
			paramType = p.typeAnnotation()
		}

		params = append(params, Param{
			Name: paramName,
			Type: paramType,
		})

		if !p.match(COMMA) {
			break
		}
		p.advance()
	}

	return params
}

func (p *Parser) typeAnnotation() TypeAnnotation {
	if !p.check(NAME) {
		p.error("expected type name")
		return TypeAnnotation{}
	}

	typeName := p.current.Lexeme
	p.advance()

	return TypeAnnotation{Name: typeName}
}

func (p *Parser) statement() Stmt {
	if p.match(RETURN) {
		p.advance()
		expr := p.expression()
		if p.match(NEWLINE) {
			p.advance()
		}
		return &Return{Value: expr}
	}

	if p.match(IF) {
		return p.ifStatement()
	}

	if p.match(WHILE) {
		return p.whileStatement()
	}

	if p.match(FOR) {
		return p.forStatement()
	}

	if p.match(BREAK) {
		p.advance()
		if p.match(NEWLINE) {
			p.advance()
		}
		return &Break{}
	}

	if p.match(CONTINUE) {
		p.advance()
		if p.match(NEWLINE) {
			p.advance()
		}
		return &Continue{}
	}

	if p.match(PASS) {
		p.advance()
		if p.match(NEWLINE) {
			p.advance()
		}
		return &Pass{}
	}

	// Assignment
	if p.check(NAME) {
		// Peek ahead for assignment
		start := p.current
		p.advance()
		if p.match(ASSIGN) {
			p.advance()
			value := p.expression()
			if p.match(NEWLINE) {
				p.advance()
			}
			return &Assign{Target: start.Lexeme, Value: value}
		}
		// Not an assignment, error
		p.error(fmt.Sprintf("unexpected identifier: %s", start.Lexeme))
		return nil
	}

	p.error("expected statement")
	return nil
}

func (p *Parser) ifStatement() Stmt {
	p.advance() // consume 'if'
	cond := p.expression()
	if !p.consume(COLON, "expected ':' after if condition") {
		return nil
	}
	if !p.consume(NEWLINE, "expected newline") {
		return nil
	}
	if !p.consume(INDENT, "expected indent") {
		return nil
	}

	var thenBody []Stmt
	for !p.check(DEDENT) && !p.check(EOF) {
		if stmt := p.statement(); stmt != nil {
			thenBody = append(thenBody, stmt)
		}
		for p.match(NEWLINE) {
			p.advance()
		}
	}
	if !p.consume(DEDENT, "expected dedent") {
		return nil
	}

	// Skip newlines between blocks
	for p.match(NEWLINE) {
		p.advance()
	}

	var elifClauses []ElifClause
	for p.match(ELIF) {
		p.advance()
		elifCond := p.expression()
		if !p.consume(COLON, "expected ':' after elif condition") {
			return nil
		}
		if !p.consume(NEWLINE, "expected newline") {
			return nil
		}
		if !p.consume(INDENT, "expected indent") {
			return nil
		}

		var elifBody []Stmt
		for !p.check(DEDENT) && !p.check(EOF) {
			if stmt := p.statement(); stmt != nil {
				elifBody = append(elifBody, stmt)
			}
			for p.match(NEWLINE) {
				p.advance()
			}
		}
		if !p.consume(DEDENT, "expected dedent") {
			return nil
		}

		// Skip newlines between blocks
		for p.match(NEWLINE) {
			p.advance()
		}

		elifClauses = append(elifClauses, ElifClause{Cond: elifCond, Body: elifBody})
	}

	var elseBody []Stmt
	if p.match(ELSE) {
		p.advance()
		if !p.consume(COLON, "expected ':' after else") {
			return nil
		}
		if !p.consume(NEWLINE, "expected newline") {
			return nil
		}
		if !p.consume(INDENT, "expected indent") {
			return nil
		}

		for !p.check(DEDENT) && !p.check(EOF) {
			if stmt := p.statement(); stmt != nil {
				elseBody = append(elseBody, stmt)
			}
			for p.match(NEWLINE) {
				p.advance()
			}
		}
		if !p.consume(DEDENT, "expected dedent") {
			return nil
		}
	}

	return &If{Cond: cond, Then: thenBody, Elif: elifClauses, Else: elseBody}
}

func (p *Parser) whileStatement() Stmt {
	p.advance() // consume 'while'
	cond := p.expression()
	p.consume(COLON, "expected ':' after while condition")
	p.consume(NEWLINE, "expected newline")
	p.consume(INDENT, "expected indent")

	var body []Stmt
	for !p.check(DEDENT) && !p.check(EOF) {
		if stmt := p.statement(); stmt != nil {
			body = append(body, stmt)
		}
		for p.match(NEWLINE) {
			p.advance()
		}
	}
	p.consume(DEDENT, "expected dedent")

	return &While{Cond: cond, Body: body}
}

func (p *Parser) forStatement() Stmt {
	p.advance() // consume 'for'
	if !p.check(NAME) {
		p.error("expected variable name in for loop")
		return nil
	}
	target := p.current.Lexeme
	p.advance()
	p.consume(IN, "expected 'in' in for loop")
	iter := p.expression()
	p.consume(COLON, "expected ':' after for clause")
	p.consume(NEWLINE, "expected newline")
	p.consume(INDENT, "expected indent")

	var body []Stmt
	for !p.check(DEDENT) && !p.check(EOF) {
		if stmt := p.statement(); stmt != nil {
			body = append(body, stmt)
		}
		for p.match(NEWLINE) {
			p.advance()
		}
	}
	p.consume(DEDENT, "expected dedent")

	return &For{Target: target, Iter: iter, Body: body}
}

func (p *Parser) expression() Expr {
	return p.orExpr()
}

func (p *Parser) orExpr() Expr {
	expr := p.andExpr()
	for p.match(OR) {
		p.advance()
		right := p.andExpr()
		expr = &BoolOp{Left: expr, Op: Or, Right: right}
	}
	return expr
}

func (p *Parser) andExpr() Expr {
	expr := p.notExpr()
	for p.match(AND) {
		p.advance()
		right := p.notExpr()
		expr = &BoolOp{Left: expr, Op: And, Right: right}
	}
	return expr
}

func (p *Parser) notExpr() Expr {
	if p.match(NOT) {
		p.advance()
		expr := p.notExpr()
		return &UnaryOp{Op: Not, Expr: expr}
	}
	return p.comparison()
}

func (p *Parser) comparison() Expr {
	expr := p.additive()
	if p.match(EQ, NE, LT, LE, GT, GE) {
		op := p.compareOpFromToken(p.current.Type)
		p.advance()
		right := p.additive()
		return &Compare{Left: expr, Op: op, Right: right}
	}
	return expr
}

func (p *Parser) additive() Expr {
	expr := p.multiplicative()

	for p.match(PLUS) || p.match(MINUS) {
		op := p.operatorFromToken(p.current.Type)
		p.advance()
		right := p.multiplicative()
		expr = &BinOp{
			Left:  expr,
			Op:    op,
			Right: right,
		}
	}

	return expr
}

func (p *Parser) multiplicative() Expr {
	expr := p.primary()

	for p.match(STAR) || p.match(SLASH) {
		op := p.operatorFromToken(p.current.Type)
		p.advance()
		right := p.primary()
		expr = &BinOp{
			Left:  expr,
			Op:    op,
			Right: right,
		}
	}

	return expr
}

func (p *Parser) postfix(expr Expr) Expr {
	for {
		if p.match(DOT) {
			// Attribute access
			p.advance()
			if !p.check(NAME) {
				p.error("expected attribute name")
				return nil
			}
			attr := p.current.Lexeme
			p.advance()
			expr = &Attribute{Value: expr, Attr: attr}
		} else if p.match(LBRACKET) {
			// Subscript
			p.advance()
			index := p.expression()
			if !p.consume(RBRACKET, "expected ']'") {
				return nil
			}
			expr = &Subscript{Value: expr, Index: index}
		} else if p.match(LPAREN) {
			// Function call
			p.advance()
			var args []Expr
			if !p.check(RPAREN) {
				for {
					args = append(args, p.expression())
					if !p.match(COMMA) {
						break
					}
					p.advance()
				}
			}
			if !p.consume(RPAREN, "expected ')'") {
				return nil
			}

			// Convert Name to Call
			if nameExpr, ok := expr.(*Name); ok {
				expr = &Call{Func: nameExpr.Id, Args: args}
			} else {
				// Method call - TODO: implement properly
				expr = &Call{Func: "method", Args: args}
			}
		} else {
			break
		}
	}
	return expr
}

func (p *Parser) primary() Expr {
	if p.match(INT) {
		lexeme := p.current.Lexeme
		p.advance()
		var val int64
		fmt.Sscanf(lexeme, "%d", &val)
		return &Num{Value: val}
	}

	if p.match(TRUE) {
		p.advance()
		return &Bool{Value: true}
	}

	if p.match(FALSE) {
		p.advance()
		return &Bool{Value: false}
	}

	if p.match(NAME) {
		name := p.current.Lexeme
		p.advance()

		return p.postfix(&Name{Id: name})
	}

	if p.match(LBRACKET) {
		// List literal
		p.advance()
		var elements []Expr
		if !p.check(RBRACKET) {
			for {
				elements = append(elements, p.expression())
				if !p.match(COMMA) {
					break
				}
				p.advance()
			}
		}
		if !p.consume(RBRACKET, "expected ']'") {
			return nil
		}
		return &ListComp{} // TODO: proper list literal type
	}

	if p.match(LPAREN) {
		p.advance()
		expr := p.expression()
		p.consume(RPAREN, "expected ')'")
		return expr
	}

	p.error(fmt.Sprintf("unexpected token: %v", p.current))
	return nil
}

func (p *Parser) operatorFromToken(tok TokenType) Operator {
	switch tok {
	case PLUS:
		return Add
	case MINUS:
		return Sub
	case STAR:
		return Mul
	case SLASH:
		return Div
	}
	return Add
}

func (p *Parser) match(types ...TokenType) bool {
	for _, t := range types {
		if p.check(t) {
			return true
		}
	}
	return false
}

func (p *Parser) check(typ TokenType) bool {
	return p.current.Type == typ
}

func (p *Parser) advance() Token {
	prev := p.current
	p.current = p.slexer.NextToken()
	return prev
}

func (p *Parser) consume(typ TokenType, msg string) bool {
	if p.check(typ) {
		p.advance()
		return true
	}
	p.error(msg)
	return false
}

func (p *Parser) compareOpFromToken(tok TokenType) CompareOp {
	switch tok {
	case EQ:
		return Eq
	case NE:
		return Ne
	case LT:
		return Lt
	case LE:
		return Le
	case GT:
		return Gt
	case GE:
		return Ge
	}
	return Eq
}

func (p *Parser) error(msg string) {
	errMsg := fmt.Sprintf("line %d, col %d: %s", p.current.Line, p.current.Col, msg)
	p.errors = append(p.errors, errMsg)
}
