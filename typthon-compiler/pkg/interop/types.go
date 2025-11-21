// Package interop - Bridge between Go compiler and Rust type checker
// Design: CGO-based FFI for calling typthon-core from Go
package interop

/*
#cgo LDFLAGS: -L../../typthon-core/target/release -ltypthon_core
#include <stdint.h>
#include <stdlib.h>

// Forward declarations for Rust FFI functions
extern int typthon_check_file(const char* filename);
extern int typthon_check_source(const char* source, int len);
extern void typthon_init_checker();
extern void typthon_cleanup_checker();
*/
import "C"
import (
	"fmt"
	"unsafe"

	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
)

// TypeChecker wraps the Rust type checker
type TypeChecker struct {
	initialized bool
}

// NewTypeChecker creates a new type checker instance
func NewTypeChecker() *TypeChecker {
	return &TypeChecker{}
}

// Init initializes the type checker
func (tc *TypeChecker) Init() error {
	if tc.initialized {
		return nil
	}

	logger.Debug("Initializing Rust type checker via FFI")
	C.typthon_init_checker()
	tc.initialized = true
	logger.Info("Type checker initialized")
	return nil
}

// Cleanup cleans up type checker resources
func (tc *TypeChecker) Cleanup() {
	if !tc.initialized {
		return
	}

	logger.Debug("Cleaning up type checker")
	C.typthon_cleanup_checker()
	tc.initialized = false
}

// CheckFile type-checks a Python file
func (tc *TypeChecker) CheckFile(filename string) error {
	if !tc.initialized {
		if err := tc.Init(); err != nil {
			return err
		}
	}

	logger.Debug("Type checking file", "filename", filename)

	cFilename := C.CString(filename)
	defer C.free(unsafe.Pointer(cFilename))

	result := C.typthon_check_file(cFilename)
	if result != 0 {
		return fmt.Errorf("type check failed for %s with code %d", filename, result)
	}

	logger.Debug("Type check passed", "filename", filename)
	return nil
}

// CheckSource type-checks Python source code
func (tc *TypeChecker) CheckSource(source string) error {
	if !tc.initialized {
		if err := tc.Init(); err != nil {
			return err
		}
	}

	logger.Debug("Type checking source code", "length", len(source))

	cSource := C.CString(source)
	defer C.free(unsafe.Pointer(cSource))

	result := C.typthon_check_source(cSource, C.int(len(source)))
	if result != 0 {
		return fmt.Errorf("type check failed with code %d", result)
	}

	logger.Debug("Type check passed")
	return nil
}

// TypeInfo represents type information from the checker
type TypeInfo struct {
	Name       string
	Kind       TypeKind
	Parameters []TypeInfo
}

// TypeKind represents the kind of type
type TypeKind int

const (
	TypeInt TypeKind = iota
	TypeFloat
	TypeString
	TypeBool
	TypeList
	TypeDict
	TypeTuple
	TypeFunction
	TypeClass
	TypeGeneric
	TypeUnion
	TypeAny
)

// GetTypeInfo retrieves type information for a variable
func (tc *TypeChecker) GetTypeInfo(varName string) (*TypeInfo, error) {
	// TODO: Implement FFI call to get type info from Rust
	logger.Debug("Getting type info", "variable", varName)

	// For now, return a placeholder
	return &TypeInfo{
		Name: varName,
		Kind: TypeInt,
	}, nil
}
