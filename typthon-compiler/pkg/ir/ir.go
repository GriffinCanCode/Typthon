// Package ir implements the intermediate representation.
//
// Design: Three-address code, explicit control flow, strongly typed.
// Simple enough to reason about, powerful enough to optimize.
package ir

// Program is the top-level IR container
type Program struct {
	Functions []*Function
}

// Function represents a compiled function
type Function struct {
	Name       string
	Params     []*Param
	ReturnType Type
	Blocks     []*Block
}

// Block is a basic block - straight-line code ending in a terminator
type Block struct {
	Label string
	Insts []Inst
	Term  Terminator
}

// Inst is a three-address code instruction
type Inst interface {
	inst()
}

// Terminator ends a basic block (branch, return, etc.)
type Terminator interface {
	term()
}

// Instructions
type Alloc struct {
	Dest Value
	Type Type
}

func (Alloc) inst() {}

type Load struct {
	Dest Value
	Src  Value
}

func (Load) inst() {}

type Store struct {
	Dest Value
	Src  Value
}

func (Store) inst() {}

type BinOp struct {
	Dest Value
	Op   Op
	L    Value
	R    Value
}

func (BinOp) inst() {}

type Call struct {
	Dest     Value
	Function string
	Args     []Value
}

func (Call) inst() {}

// Terminators
type Return struct {
	Value Value
}

func (Return) term() {}

type Branch struct {
	Target string
}

func (Branch) term() {}

type CondBranch struct {
	Cond       Value
	TrueBlock  string
	FalseBlock string
}

func (CondBranch) term() {}

// Values and types
type Value interface {
	value()
}

type Temp struct {
	ID   int
	Type Type
}

func (Temp) value() {}

type Const struct {
	Val  int64
	Type Type
}

func (Const) value() {}

type Param struct {
	Name string
	Type Type
}

func (Param) value() {}

type Type interface {
	typ()
}

type IntType struct{}

func (IntType) typ() {}

type FloatType struct{}

func (FloatType) typ() {}

type PtrType struct {
	Elem Type
}

func (PtrType) typ() {}

// Operations
type Op int

const (
	OpAdd Op = iota
	OpSub
	OpMul
	OpDiv
	OpEq
	OpLt
	OpGt
)
