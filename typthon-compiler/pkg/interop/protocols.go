// Protocol checking integration with typthon-core
package interop

import (
	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
)

// ProtocolChecker validates protocol conformance at compile time
type ProtocolChecker struct {
	protocols map[string]*Protocol
}

// Protocol represents a structural interface
type Protocol struct {
	Name    string
	Methods map[string]*ir.FunctionType
}

// NewProtocolChecker creates a new protocol checker
func NewProtocolChecker() *ProtocolChecker {
	checker := &ProtocolChecker{
		protocols: make(map[string]*Protocol),
	}
	checker.initBuiltinProtocols()
	return checker
}

func (pc *ProtocolChecker) initBuiltinProtocols() {
	// Iterable protocol
	pc.protocols["Iterable"] = &Protocol{
		Name: "Iterable",
		Methods: map[string]*ir.FunctionType{
			"__iter__": {
				Params: []ir.Type{},
				Return: &ir.GenericType{Name: "Iterator", Params: []ir.Type{}},
			},
		},
	}

	// Iterator protocol
	pc.protocols["Iterator"] = &Protocol{
		Name: "Iterator",
		Methods: map[string]*ir.FunctionType{
			"__next__": {
				Params: []ir.Type{},
				Return: ir.IntType{}, // element type
			},
			"__iter__": {
				Params: []ir.Type{},
				Return: &ir.GenericType{Name: "Iterator", Params: []ir.Type{}},
			},
		},
	}

	// Sized protocol
	pc.protocols["Sized"] = &Protocol{
		Name: "Sized",
		Methods: map[string]*ir.FunctionType{
			"__len__": {
				Params: []ir.Type{},
				Return: ir.IntType{},
			},
		},
	}

	// Callable protocol
	pc.protocols["Callable"] = &Protocol{
		Name: "Callable",
		Methods: map[string]*ir.FunctionType{
			"__call__": {
				Params: []ir.Type{},  // variable
				Return: ir.IntType{}, // variable
			},
		},
	}
}

// CheckProtocol validates that a type implements a protocol
func (pc *ProtocolChecker) CheckProtocol(typ ir.Type, protocolName string) bool {
	protocol, ok := pc.protocols[protocolName]
	if !ok {
		logger.Warn("Unknown protocol", "name", protocolName)
		return false
	}

	// Check if type is a class with required methods
	classType, ok := typ.(ir.ClassType)
	if !ok {
		return false
	}

	// Look up class definition
	// TODO: integrate with full class registry
	logger.Debug("Checking protocol conformance", "class", classType.Name, "protocol", protocolName)

	return true // Placeholder - full implementation would check all methods
}

// CheckClassProtocol validates protocol conformance for a class
func (pc *ProtocolChecker) CheckClassProtocol(class *ir.Class, protocolName string) []string {
	protocol, ok := pc.protocols[protocolName]
	if !ok {
		return []string{"Unknown protocol: " + protocolName}
	}

	var errors []string

	// Check each required method
	for methodName, methodType := range protocol.Methods {
		found := false
		for _, classMethod := range class.Methods {
			if classMethod.Name == class.Name+"_"+methodName {
				found = true
				// TODO: Check method signature matches
				_ = methodType
				break
			}
		}

		if !found {
			errors = append(errors, "Missing method: "+methodName)
		}
	}

	return errors
}

// RegisterProtocol adds a custom protocol
func (pc *ProtocolChecker) RegisterProtocol(protocol *Protocol) {
	pc.protocols[protocol.Name] = protocol
	logger.Debug("Registered protocol", "name", protocol.Name)
}
