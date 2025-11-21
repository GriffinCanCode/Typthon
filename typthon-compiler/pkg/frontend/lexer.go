// Package frontend - Lexer for minimal Python subset
// Design: Hand-written scanner, zero allocations in hot path
package frontend

import (
	"fmt"
	"unicode"
)

type TokenType int

const (
	EOF TokenType = iota
	NEWLINE
	INDENT
	DEDENT

	// Literals
	INT
	NAME

	// Keywords
	DEF
	CLASS
	RETURN
	IF
	ELIF
	ELSE
	WHILE
	FOR
	IN
	BREAK
	CONTINUE
	PASS
	TRUE
	FALSE
	LAMBDA
	SELF

	// Operators
	PLUS
	MINUS
	STAR
	SLASH
	EQ     // ==
	NE     // !=
	LT     // <
	LE     // <=
	GT     // >
	GE     // >=
	AND    // and
	OR     // or
	NOT    // not
	ASSIGN // =

	// Delimiters
	LPAREN
	RPAREN
	LBRACKET
	RBRACKET
	COLON
	COMMA
	ARROW
	DOT
)

type Token struct {
	Type   TokenType
	Lexeme string
	Line   int
	Col    int
}

type Lexer struct {
	source []rune
	start  int
	pos    int
	line   int
	col    int

	// Indentation stack for Python's significant whitespace
	indents []int
	pending []Token
}

func NewLexer(source string) *Lexer {
	return &Lexer{
		source:  []rune(source),
		line:    1,
		col:     1,
		indents: []int{0},
	}
}

func (l *Lexer) Next() Token {
	for {
		if len(l.pending) > 0 {
			tok := l.pending[0]
			l.pending = l.pending[1:]
			return tok
		}

		l.skipWhitespace()

		if l.isAtEnd() {
			// Emit pending DEDENTs at EOF
			if len(l.indents) > 1 {
				l.indents = l.indents[:len(l.indents)-1]
				return Token{Type: DEDENT, Line: l.line, Col: l.col}
			}
			return Token{Type: EOF, Line: l.line, Col: l.col}
		}

		// Handle indentation at line start (before consuming any character)
		if l.col == 1 && (l.peek() == ' ' || l.peek() == '\t') {
			if tok := l.handleIndent(); tok.Type != EOF {
				if tok.Type != 0 || tok.Lexeme != "continue" {
					return tok
				}
				// If handleIndent returns continue signal, loop again
				continue
			}
		}

		l.start = l.pos
		c := l.advance()

		switch c {
		case '\n':
			l.line++
			l.col = 1
			return Token{Type: NEWLINE, Lexeme: "\n", Line: l.line - 1}
		case '+':
			return l.makeToken(PLUS, "+")
		case '-':
			if l.match('>') {
				return l.makeToken(ARROW, "->")
			}
			return l.makeToken(MINUS, "-")
		case '*':
			return l.makeToken(STAR, "*")
		case '/':
			return l.makeToken(SLASH, "/")
		case '(':
			return l.makeToken(LPAREN, "(")
		case ')':
			return l.makeToken(RPAREN, ")")
		case ':':
			return l.makeToken(COLON, ":")
		case ',':
			return l.makeToken(COMMA, ",")
		case '=':
			if l.match('=') {
				return l.makeToken(EQ, "==")
			}
			return l.makeToken(ASSIGN, "=")
		case '!':
			if l.match('=') {
				return l.makeToken(NE, "!=")
			}
		case '<':
			if l.match('=') {
				return l.makeToken(LE, "<=")
			}
			return l.makeToken(LT, "<")
		case '>':
			if l.match('=') {
				return l.makeToken(GE, ">=")
			}
			return l.makeToken(GT, ">")
		}

		if unicode.IsDigit(c) {
			return l.number()
		}

		if unicode.IsLetter(c) || c == '_' {
			return l.identifier()
		}

		return l.error(fmt.Sprintf("unexpected character: %c", c))
	}
}

func (l *Lexer) skipWhitespace() {
	for !l.isAtEnd() {
		c := l.peek()
		// Don't skip spaces at line start (they're indentation)
		// Don't skip newlines (they're significant)
		if c == ' ' && l.col != 1 {
			l.advance()
		} else if c == '\t' && l.col != 1 {
			l.advance()
		} else if c == '#' { // Comments
			for !l.isAtEnd() && l.peek() != '\n' {
				l.advance()
			}
		} else {
			break
		}
	}
}

func (l *Lexer) handleIndent() Token {
	spaces := 0
	startCol := l.col
	for l.peek() == ' ' || l.peek() == '\t' {
		if l.peek() == '\t' {
			spaces += 4 // Treat tab as 4 spaces
		} else {
			spaces++
		}
		l.advance()
	}

	// Skip empty lines and comments
	if l.peek() == '\n' || l.peek() == '#' || l.peek() == '\x00' {
		// Advance past newline if present
		if l.peek() == '\n' {
			l.advance()
			l.line++
			l.col = 1
		}
		return Token{Type: 0, Lexeme: "continue"}
	}

	current := l.indents[len(l.indents)-1]

	if spaces > current {
		l.indents = append(l.indents, spaces)
		return Token{Type: INDENT, Line: l.line, Col: startCol}
	} else if spaces < current {
		// Find matching indent level
		for len(l.indents) > 1 && l.indents[len(l.indents)-1] > spaces {
			l.indents = l.indents[:len(l.indents)-1]
			l.pending = append(l.pending, Token{Type: DEDENT, Line: l.line, Col: startCol})
		}
		if len(l.pending) > 0 {
			tok := l.pending[0]
			l.pending = l.pending[1:]
			return tok
		}
	}

	// Same indent level - skip and continue lexing
	return Token{Type: 0, Lexeme: "continue"}
}

func (l *Lexer) number() Token {
	for unicode.IsDigit(l.peek()) {
		l.advance()
	}
	return l.makeToken(INT, string(l.source[l.start:l.pos]))
}

func (l *Lexer) identifier() Token {
	for unicode.IsLetter(l.peek()) || unicode.IsDigit(l.peek()) || l.peek() == '_' {
		l.advance()
	}

	text := string(l.source[l.start:l.pos])

	// Keywords
	switch text {
	case "def":
		return l.makeToken(DEF, text)
	case "return":
		return l.makeToken(RETURN, text)
	case "if":
		return l.makeToken(IF, text)
	case "elif":
		return l.makeToken(ELIF, text)
	case "else":
		return l.makeToken(ELSE, text)
	case "while":
		return l.makeToken(WHILE, text)
	case "for":
		return l.makeToken(FOR, text)
	case "in":
		return l.makeToken(IN, text)
	case "break":
		return l.makeToken(BREAK, text)
	case "continue":
		return l.makeToken(CONTINUE, text)
	case "pass":
		return l.makeToken(PASS, text)
	case "True":
		return l.makeToken(TRUE, text)
	case "False":
		return l.makeToken(FALSE, text)
	case "and":
		return l.makeToken(AND, text)
	case "or":
		return l.makeToken(OR, text)
	case "not":
		return l.makeToken(NOT, text)
	}

	return l.makeToken(NAME, text)
}

func (l *Lexer) peek() rune {
	if l.isAtEnd() {
		return '\x00'
	}
	return l.source[l.pos]
}

func (l *Lexer) advance() rune {
	c := l.source[l.pos]
	l.pos++
	l.col++
	return c
}

func (l *Lexer) match(expected rune) bool {
	if l.isAtEnd() || l.source[l.pos] != expected {
		return false
	}
	l.pos++
	l.col++
	return true
}

func (l *Lexer) isAtEnd() bool {
	return l.pos >= len(l.source)
}

func (l *Lexer) makeToken(typ TokenType, lexeme string) Token {
	return Token{
		Type:   typ,
		Lexeme: lexeme,
		Line:   l.line,
		Col:    l.col - len([]rune(lexeme)),
	}
}

func (l *Lexer) error(msg string) Token {
	return Token{
		Type:   EOF,
		Lexeme: fmt.Sprintf("error: %s", msg),
		Line:   l.line,
		Col:    l.col,
	}
}
