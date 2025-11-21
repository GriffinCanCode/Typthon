// Package ir implements the intermediate representation.
//
// Design: Three-address code, explicit control flow, strongly typed.
// Simple enough to reason about, powerful enough to optimize.
package ir

// Program is the top-level IR container
type Program struct {
	Functions []*Function
	Classes   []*Class
}

// Class represents a compiled class
type Class struct {
	Name    string
	Bases   []string
	Methods []*Function
	Attrs   map[string]Type
	VTable  []*Function // Virtual method table
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

type AllocObject struct {
	Dest      Value
	ClassName string
}

func (AllocObject) inst() {}

type GetAttr struct {
	Dest Value
	Obj  Value
	Attr string
}

func (GetAttr) inst() {}

type SetAttr struct {
	Obj   Value
	Attr  string
	Value Value
}

func (SetAttr) inst() {}

type GetItem struct {
	Dest  Value
	Obj   Value
	Index Value
}

func (GetItem) inst() {}

type SetItem struct {
	Obj   Value
	Index Value
	Value Value
}

func (SetItem) inst() {}

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

type MethodCall struct {
	Dest   Value
	Obj    Value
	Method string
	Args   []Value
}

func (MethodCall) inst() {}

type MakeClosure struct {
	Dest     Value
	Function string
	Captures []Value
}

func (MakeClosure) inst() {}

type ClosureCall struct {
	Dest    Value
	Closure Value
	Args    []Value
}

func (ClosureCall) inst() {}

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

type BoolType struct{}

func (BoolType) typ() {}

type FloatType struct{}

func (FloatType) typ() {}

type StringType struct{}

func (StringType) typ() {}

type ListType struct {
	Elem Type
}

func (ListType) typ() {}

type DictType struct {
	Key   Type
	Value Type
}

func (DictType) typ() {}

type ClassType struct {
	Name string
}

func (ClassType) typ() {}

type FunctionType struct {
	Params []Type
	Return Type
}

func (FunctionType) typ() {}

type ClosureType struct {
	Function FunctionType
	Captures []Type
}

func (ClosureType) typ() {}

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
	OpNe
	OpLt
	OpLe
	OpGt
	OpGe
	OpAnd
	OpOr
	OpXor
)
