// Package frontend implements Python parsing and AST construction.
//
// Design: Minimal, focused on correctness. No fancy optimizations here.
package frontend

// AST node types - minimal for Phase 1
type Node interface {
	node()
}

type Module struct {
	Body []Stmt
}

func (Module) node() {}

type Stmt interface {
	Node
	stmt()
}

type Expr interface {
	Node
	expr()
}

// Statements
type FunctionDef struct {
	Name        string
	Params      []Param
	Body        []Stmt
	Return      TypeAnnotation
	IsGenerator bool
	Decorators  []string
}

func (FunctionDef) node() {}
func (FunctionDef) stmt() {}

type ClassDef struct {
	Name    string
	Bases   []string
	Methods []*FunctionDef
	Attrs   []Assign
}

func (ClassDef) node() {}
func (ClassDef) stmt() {}

type Return struct {
	Value Expr
}

func (Return) node() {}
func (Return) stmt() {}

type If struct {
	Cond Expr
	Then []Stmt
	Elif []ElifClause
	Else []Stmt
}

func (If) node() {}
func (If) stmt() {}

type ElifClause struct {
	Cond Expr
	Body []Stmt
}

type While struct {
	Cond Expr
	Body []Stmt
}

func (While) node() {}
func (While) stmt() {}

type For struct {
	Target string
	Iter   Expr
	Body   []Stmt
}

func (For) node() {}
func (For) stmt() {}

type Break struct{}

func (Break) node() {}
func (Break) stmt() {}

type Continue struct{}

func (Continue) node() {}
func (Continue) stmt() {}

type Pass struct{}

func (Pass) node() {}
func (Pass) stmt() {}

type Yield struct {
	Value Expr
}

func (Yield) node() {}
func (Yield) stmt() {}

type Assign struct {
	Target string
	Value  Expr
}

func (Assign) node() {}
func (Assign) stmt() {}

// Expressions
type BinOp struct {
	Left  Expr
	Op    Operator
	Right Expr
}

func (BinOp) node() {}
func (BinOp) expr() {}

type UnaryOp struct {
	Op   Operator
	Expr Expr
}

func (UnaryOp) node() {}
func (UnaryOp) expr() {}

type Compare struct {
	Left  Expr
	Op    CompareOp
	Right Expr
}

func (Compare) node() {}
func (Compare) expr() {}

type BoolOp struct {
	Left  Expr
	Op    BoolOperator
	Right Expr
}

func (BoolOp) node() {}
func (BoolOp) expr() {}

type Name struct {
	Id string
}

func (Name) node() {}
func (Name) expr() {}

type Num struct {
	Value int64
}

func (Num) node() {}
func (Num) expr() {}

type Bool struct {
	Value bool
}

func (Bool) node() {}
func (Bool) expr() {}

type Call struct {
	Func string
	Args []Expr
}

func (Call) node() {}
func (Call) expr() {}

type ListComp struct {
	Elt    Expr
	Target string
	Iter   Expr
	Conds  []Expr
}

func (ListComp) node() {}
func (ListComp) expr() {}

type DictComp struct {
	Key    Expr
	Value  Expr
	Target string
	Iter   Expr
	Conds  []Expr
}

func (DictComp) node() {}
func (DictComp) expr() {}

type Lambda struct {
	Params []Param
	Body   Expr
}

func (Lambda) node() {}
func (Lambda) expr() {}

type Attribute struct {
	Value Expr
	Attr  string
}

func (Attribute) node() {}
func (Attribute) expr() {}

type Subscript struct {
	Value Expr
	Index Expr
}

func (Subscript) node() {}
func (Subscript) expr() {}

type Match struct {
	Subject Expr
	Cases   []MatchCase
}

func (Match) node() {}
func (Match) stmt() {}

type MatchCase struct {
	Pattern Pattern
	Guard   Expr
	Body    []Stmt
}

type Pattern interface {
	Node
	pattern()
}

type LiteralPattern struct {
	Value Expr
}

func (LiteralPattern) node()    {}
func (LiteralPattern) pattern() {}

type CapturePattern struct {
	Name string
}

func (CapturePattern) node()    {}
func (CapturePattern) pattern() {}

type OrPattern struct {
	Patterns []Pattern
}

func (OrPattern) node()    {}
func (OrPattern) pattern() {}

type ClassPattern struct {
	Class string
	Args  []Pattern
}

func (ClassPattern) node()    {}
func (ClassPattern) pattern() {}

// Supporting types
type Param struct {
	Name string
	Type TypeAnnotation
}

type TypeAnnotation struct {
	Name string
}

type Operator int

const (
	Add Operator = iota
	Sub
	Mul
	Div
	Not
)

type CompareOp int

const (
	Eq CompareOp = iota
	Ne
	Lt
	Le
	Gt
	Ge
)

type BoolOperator int

const (
	And BoolOperator = iota
	Or
)
