// Pattern matching parser
package frontend

func (p *Parser) matchStatement() Stmt {
	p.advance() // consume 'match'

	// Parse subject expression
	subject := p.expression()

	if !p.consume(COLON, "expected ':'") {
		return nil
	}

	if !p.consume(NEWLINE, "expected newline") {
		return nil
	}

	if !p.consume(INDENT, "expected indent") {
		return nil
	}

	// Parse cases
	var cases []MatchCase

	for p.match(CASE) {
		p.advance() // consume 'case'

		// Parse pattern
		pattern := p.parsePattern()
		if pattern == nil {
			p.error("expected pattern")
			continue
		}

		// Parse optional guard
		var guard Expr
		if p.match(IF) {
			p.advance()
			guard = p.expression()
		}

		if !p.consume(COLON, "expected ':'") {
			continue
		}

		if !p.consume(NEWLINE, "expected newline") {
			continue
		}

		if !p.consume(INDENT, "expected indent") {
			continue
		}

		// Parse case body
		var body []Stmt
		for !p.check(DEDENT) && !p.check(EOF) {
			if stmt := p.statement(); stmt != nil {
				body = append(body, stmt)
			}

			for p.match(NEWLINE) {
				p.advance()
			}
		}

		if !p.consume(DEDENT, "expected dedent") {
			continue
		}

		cases = append(cases, MatchCase{
			Pattern: pattern,
			Guard:   guard,
			Body:    body,
		})

		// Allow newlines between cases
		for p.match(NEWLINE) {
			p.advance()
		}
	}

	if !p.consume(DEDENT, "expected dedent") {
		return nil
	}

	return &Match{
		Subject: subject,
		Cases:   cases,
	}
}

func (p *Parser) parsePattern() Pattern {
	// Literal pattern
	if p.check(INT) {
		val := p.current.Lexeme
		p.advance()
		return &LiteralPattern{Value: &Num{Value: parseInt64(val)}}
	}

	if p.check(TRUE) || p.check(FALSE) {
		val := p.current.Type == TRUE
		p.advance()
		return &LiteralPattern{Value: &Bool{Value: val}}
	}

	// Capture pattern (variable name)
	if p.check(NAME) {
		name := p.current.Lexeme
		p.advance()

		// Check for class pattern: ClassName(args...)
		if p.match(LPAREN) {
			p.advance()
			var args []Pattern

			if !p.check(RPAREN) {
				for {
					arg := p.parsePattern()
					if arg == nil {
						break
					}
					args = append(args, arg)

					if !p.match(COMMA) {
						break
					}
					p.advance()
				}
			}

			if !p.consume(RPAREN, "expected ')'") {
				return nil
			}

			return &ClassPattern{
				Class: name,
				Args:  args,
			}
		}

		// Simple capture
		return &CapturePattern{Name: name}
	}

	// Or pattern: pattern1 | pattern2
	patterns := []Pattern{}
	first := p.parsePattern()
	if first != nil {
		patterns = append(patterns, first)

		for p.match(OR) {
			p.advance()
			pat := p.parsePattern()
			if pat != nil {
				patterns = append(patterns, pat)
			}
		}

		if len(patterns) > 1 {
			return &OrPattern{Patterns: patterns}
		}
		return first
	}

	return nil
}

func parseInt64(s string) int64 {
	var result int64
	for _, c := range s {
		if c >= '0' && c <= '9' {
			result = result*10 + int64(c-'0')
		}
	}
	return result
}
