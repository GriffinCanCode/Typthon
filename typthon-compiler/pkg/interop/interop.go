// Package interop implements foreign function interface and language interoperability.
//
// Design: Zero-cost C calling convention, automatic binding generation.
// Call any compiled language with minimal boilerplate.
package interop

// CFunc represents a C function signature
type CFunc struct {
	Name       string
	ReturnType Type
	Params     []Param
	Library    string
}

// Param is a function parameter
type Param struct {
	Name string
	Type Type
}

// Type represents a C-compatible type
type Type int

const (
	Int Type = iota
	Float
	Ptr
	Void
)

// GenerateBinding creates FFI bindings for external functions
func GenerateBinding(fn *CFunc) (string, error) {
	// TODO: Generate FFI shim code
	// 1. Marshal Python types to C types
	// 2. Call C function with correct calling convention
	// 3. Marshal return value back
	return "", nil
}

// ExternDecl parses extern declarations
// Example: @extern("libc") def malloc(size: int) -> pointer: ...
type ExternDecl struct {
	Library  string
	Function string
	Params   []Param
	Return   Type
}

func ParseExtern(source string) (*ExternDecl, error) {
	// TODO: Parse @extern decorator
	return nil, nil
}
