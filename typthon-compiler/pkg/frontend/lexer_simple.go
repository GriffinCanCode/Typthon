// Simple lexer - rewritten to avoid infinite loops
package frontend

import (
	"fmt"
	"unicode"
)

type SimpleLexer struct {
	source      []rune
	pos         int
	line        int
	col         int
	indents     []int
	atLineStart bool
}

func NewSimpleLexer(source string) *SimpleLexer {
	return &SimpleLexer{
		source:      []rune(source),
		line:        1,
		col:         1,
		indents:     []int{0},
		atLineStart: true,
	}
}

func (l *SimpleLexer) NextToken() Token {
	// Skip whitespace (except at line start where it matters)
	if !l.atLineStart {
		l.skipSpaces()
	}

	// Handle EOF
	if l.pos >= len(l.source) {
		if len(l.indents) > 1 {
			l.indents = l.indents[:len(l.indents)-1]
			return Token{Type: DEDENT, Line: l.line, Col: l.col}
		}
		return Token{Type: EOF, Line: l.line, Col: l.col}
	}

	// Handle indentation at line start
	if l.atLineStart {
		return l.handleLineStart()
	}

	startPos := l.pos
	c := l.advance()

	switch c {
	case '\n':
		l.atLineStart = true
		l.line++
		l.col = 1
		return Token{Type: NEWLINE, Lexeme: "\n", Line: l.line - 1}
	case '+':
		return Token{Type: PLUS, Lexeme: "+", Line: l.line, Col: l.col - 1}
	case '-':
		if l.peek() == '>' {
			l.advance()
			return Token{Type: ARROW, Lexeme: "->", Line: l.line, Col: l.col - 2}
		}
		return Token{Type: MINUS, Lexeme: "-", Line: l.line, Col: l.col - 1}
	case '*':
		return Token{Type: STAR, Lexeme: "*", Line: l.line, Col: l.col - 1}
	case '/':
		return Token{Type: SLASH, Lexeme: "/", Line: l.line, Col: l.col - 1}
	case '(':
		return Token{Type: LPAREN, Lexeme: "(", Line: l.line, Col: l.col - 1}
	case ')':
		return Token{Type: RPAREN, Lexeme: ")", Line: l.line, Col: l.col - 1}
	case '[':
		return Token{Type: LBRACKET, Lexeme: "[", Line: l.line, Col: l.col - 1}
	case ']':
		return Token{Type: RBRACKET, Lexeme: "]", Line: l.line, Col: l.col - 1}
	case ':':
		return Token{Type: COLON, Lexeme: ":", Line: l.line, Col: l.col - 1}
	case ',':
		return Token{Type: COMMA, Lexeme: ",", Line: l.line, Col: l.col - 1}
	case '.':
		return Token{Type: DOT, Lexeme: ".", Line: l.line, Col: l.col - 1}
	case '=':
		if l.peek() == '=' {
			l.advance()
			return Token{Type: EQ, Lexeme: "==", Line: l.line, Col: l.col - 2}
		}
		return Token{Type: ASSIGN, Lexeme: "=", Line: l.line, Col: l.col - 1}
	case '!':
		if l.peek() == '=' {
			l.advance()
			return Token{Type: NE, Lexeme: "!=", Line: l.line, Col: l.col - 2}
		}
	case '<':
		if l.peek() == '=' {
			l.advance()
			return Token{Type: LE, Lexeme: "<=", Line: l.line, Col: l.col - 2}
		}
		return Token{Type: LT, Lexeme: "<", Line: l.line, Col: l.col - 1}
	case '>':
		if l.peek() == '=' {
			l.advance()
			return Token{Type: GE, Lexeme: ">=", Line: l.line, Col: l.col - 2}
		}
		return Token{Type: GT, Lexeme: ">", Line: l.line, Col: l.col - 1}
	}

	if unicode.IsDigit(c) {
		l.pos = startPos
		l.col -= 1
		return l.scanNumber()
	}

	if unicode.IsLetter(c) || c == '_' {
		l.pos = startPos
		l.col -= 1
		return l.scanIdentifier()
	}

	return Token{Type: EOF, Lexeme: fmt.Sprintf("error: unexpected char %c", c), Line: l.line, Col: l.col}
}

func (l *SimpleLexer) handleLineStart() Token {
	// Count spaces
	spaces := 0
	for l.pos < len(l.source) && (l.source[l.pos] == ' ' || l.source[l.pos] == '\t') {
		if l.source[l.pos] == '\t' {
			spaces += 4
		} else {
			spaces++
		}
		l.pos++
		l.col++
	}

	// Check for empty line or comment
	if l.pos >= len(l.source) || l.source[l.pos] == '\n' || l.source[l.pos] == '#' {
		// Skip empty line
		if l.pos < len(l.source) && l.source[l.pos] == '\n' {
			l.pos++
			l.line++
			l.col = 1
		}
		return l.NextToken()
	}

	l.atLineStart = false
	current := l.indents[len(l.indents)-1]

	if spaces > current {
		l.indents = append(l.indents, spaces)
		return Token{Type: INDENT, Line: l.line, Col: 1}
	} else if spaces < current {
		l.indents = l.indents[:len(l.indents)-1]
		return Token{Type: DEDENT, Line: l.line, Col: 1}
	}

	// Same level, continue
	return l.NextToken()
}

func (l *SimpleLexer) scanNumber() Token {
	start := l.pos
	startCol := l.col

	for l.pos < len(l.source) && unicode.IsDigit(l.source[l.pos]) {
		l.pos++
		l.col++
	}

	return Token{
		Type:   INT,
		Lexeme: string(l.source[start:l.pos]),
		Line:   l.line,
		Col:    startCol,
	}
}

func (l *SimpleLexer) scanIdentifier() Token {
	start := l.pos
	startCol := l.col

	for l.pos < len(l.source) {
		c := l.source[l.pos]
		if unicode.IsLetter(c) || unicode.IsDigit(c) || c == '_' {
			l.pos++
			l.col++
		} else {
			break
		}
	}

	text := string(l.source[start:l.pos])
	typ := NAME

	switch text {
	case "def":
		typ = DEF
	case "class":
		typ = CLASS
	case "return":
		typ = RETURN
	case "yield":
		typ = YIELD
	case "if":
		typ = IF
	case "elif":
		typ = ELIF
	case "else":
		typ = ELSE
	case "while":
		typ = WHILE
	case "for":
		typ = FOR
	case "lambda":
		typ = LAMBDA
	case "self":
		typ = SELF
	case "in":
		typ = IN
	case "break":
		typ = BREAK
	case "continue":
		typ = CONTINUE
	case "pass":
		typ = PASS
	case "match":
		typ = MATCH
	case "case":
		typ = CASE
	case "True":
		typ = TRUE
	case "False":
		typ = FALSE
	case "and":
		typ = AND
	case "or":
		typ = OR
	case "not":
		typ = NOT
	}

	return Token{
		Type:   typ,
		Lexeme: text,
		Line:   l.line,
		Col:    startCol,
	}
}

func (l *SimpleLexer) skipSpaces() {
	for l.pos < len(l.source) && (l.source[l.pos] == ' ' || l.source[l.pos] == '\t') {
		l.pos++
		l.col++
	}
}

func (l *SimpleLexer) peek() rune {
	if l.pos >= len(l.source) {
		return '\x00'
	}
	return l.source[l.pos]
}

func (l *SimpleLexer) advance() rune {
	c := l.source[l.pos]
	l.pos++
	l.col++
	return c
}
