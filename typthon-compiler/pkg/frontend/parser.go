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

	// Parse top-level functions
	for !p.check(EOF) {
		// Skip any DEDENT tokens at module level
		for p.match(DEDENT) {
			p.advance()
		}

		if p.check(EOF) {
			break
		}

		if stmt := p.function(); stmt != nil {
			module.Body = append(module.Body, stmt)
		}

		// Skip newlines between functions
		for p.match(NEWLINE) {
			p.advance()
		}
	}

	if len(p.errors) > 0 {
		return nil, fmt.Errorf("parse errors: %v", p.errors)
	}

	return module, nil
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

		// Consume trailing newline
		if p.match(NEWLINE) {
			p.advance()
		}

		return &Return{Value: expr}
	}

	p.error("expected statement")
	return nil
}

func (p *Parser) expression() Expr {
	return p.additive()
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

func (p *Parser) primary() Expr {
	if p.match(INT) {
		lexeme := p.current.Lexeme
		p.advance()

		// Parse integer value
		var val int64
		fmt.Sscanf(lexeme, "%d", &val)

		return &Num{Value: val}
	}

	if p.match(NAME) {
		name := p.current.Lexeme
		p.advance()

		// Check for function call
		if p.match(LPAREN) {
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

			return &Call{
				Func: name,
				Args: args,
			}
		}

		return &Name{Id: name}
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

func (p *Parser) error(msg string) {
	errMsg := fmt.Sprintf("line %d, col %d: %s", p.current.Line, p.current.Col, msg)
	p.errors = append(p.errors, errMsg)
}
