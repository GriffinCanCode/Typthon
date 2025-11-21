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
	Name   string
	Params []Param
	Body   []Stmt
	Return TypeAnnotation
}

func (FunctionDef) node() {}
func (FunctionDef) stmt() {}

type Return struct {
	Value Expr
}

func (Return) node() {}
func (Return) stmt() {}

// Expressions
type BinOp struct {
	Left  Expr
	Op    Operator
	Right Expr
}

func (BinOp) node() {}
func (BinOp) expr() {}

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

type Call struct {
	Func string
	Args []Expr
}

func (Call) node() {}
func (Call) expr() {}

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
)
