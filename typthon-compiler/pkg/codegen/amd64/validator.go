// Package amd64 - Assembly validation and correctness verification
package amd64

import (
	"bufio"
	"fmt"
	"regexp"
	"strings"

	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
)

// ValidationError represents an assembly validation error
type ValidationError struct {
	Line    int
	Message string
	Code    string
}

func (e *ValidationError) Error() string {
	return fmt.Sprintf("line %d: %s\n  %s", e.Line, e.Message, e.Code)
}

// Validator validates generated x86-64 assembly
type Validator struct {
	errors []ValidationError
	warns  []ValidationError
}

// NewValidator creates a new assembly validator
func NewValidator() *Validator {
	return &Validator{
		errors: make([]ValidationError, 0),
		warns:  make([]ValidationError, 0),
	}
}

// Validate performs comprehensive validation on assembly code
func (v *Validator) Validate(assembly string) error {
	lines := strings.Split(assembly, "\n")

	v.validateSyntax(lines)
	v.validateRegisters(lines)
	v.validateCallingConvention(lines)
	v.validateStackBalance(lines)
	v.validateInstructionValidity(lines)
	v.validateMemoryAddressing(lines)

	if len(v.errors) > 0 {
		return v.formatErrors()
	}

	if len(v.warns) > 0 {
		v.logWarnings()
	}

	return nil
}

// validateSyntax checks for basic syntax errors
func (v *Validator) validateSyntax(lines []string) {
	for i, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}

		// Check for malformed instructions
		if strings.HasPrefix(line, "\t") && !isValidInstruction(line) {
			v.addError(i+1, "malformed instruction", line)
		}

		// Check for invalid label format
		if strings.HasSuffix(line, ":") && strings.Contains(line, " ") {
			v.addError(i+1, "invalid label format (contains spaces)", line)
		}
	}
}

// validateRegisters checks register usage correctness
func (v *Validator) validateRegisters(lines []string) {
	validRegs := map[string]bool{
		// 64-bit registers
		"%rax": true, "%rbx": true, "%rcx": true, "%rdx": true,
		"%rsi": true, "%rdi": true, "%rbp": true, "%rsp": true,
		"%r8": true, "%r9": true, "%r10": true, "%r11": true,
		"%r12": true, "%r13": true, "%r14": true, "%r15": true,
		// 32-bit registers
		"%eax": true, "%ebx": true, "%ecx": true, "%edx": true,
		"%esi": true, "%edi": true, "%ebp": true, "%esp": true,
		// 8-bit registers
		"%al": true, "%bl": true, "%cl": true, "%dl": true,
	}

	regPattern := regexp.MustCompile(`%[a-z0-9]+`)

	for i, line := range lines {
		regs := regPattern.FindAllString(line, -1)
		for _, reg := range regs {
			if !validRegs[reg] {
				v.addError(i+1, fmt.Sprintf("invalid register: %s", reg), line)
			}
		}
	}
}

// validateCallingConvention checks System V ABI compliance
func (v *Validator) validateCallingConvention(lines []string) {
	inFunction := false
	functionName := ""
	savedRegs := make(map[string]bool)

	for i, line := range lines {
		line = strings.TrimSpace(line)

		// Track function boundaries
		if strings.HasSuffix(line, ":") && !strings.HasPrefix(line, ".L") {
			inFunction = true
			functionName = strings.TrimSuffix(line, ":")
			savedRegs = make(map[string]bool)
		}

		if !inFunction {
			continue
		}

		// Track push/pop of callee-saved registers
		if strings.Contains(line, "pushq") {
			parts := strings.Fields(line)
			if len(parts) >= 2 {
				reg := parts[1]
				if isCalleeSaved(reg) {
					savedRegs[reg] = true
				}
			}
		}

		if strings.Contains(line, "popq") {
			parts := strings.Fields(line)
			if len(parts) >= 2 {
				reg := parts[1]
				if savedRegs[reg] {
					delete(savedRegs, reg)
				}
			}
		}

		// Track leave instruction (equivalent to movq %rbp, %rsp; popq %rbp)
		if strings.Contains(line, "leave") {
			// leave restores rbp automatically
			delete(savedRegs, "%rbp")
		}

		// Check for function epilogue
		if strings.Contains(line, "retq") || strings.Contains(line, "ret") {
			// Verify all saved registers were restored
			if len(savedRegs) > 0 {
				v.addError(i+1, fmt.Sprintf("callee-saved registers not restored in %s: %v", functionName, savedRegs), line)
			}
			inFunction = false
		}
	}
}

// validateStackBalance checks stack push/pop balance
func (v *Validator) validateStackBalance(lines []string) {
	stackDepth := 0
	inFunction := false

	for i, line := range lines {
		line = strings.TrimSpace(line)

		// Track function boundaries
		if strings.HasSuffix(line, ":") && !strings.HasPrefix(line, ".L") {
			inFunction = true
			stackDepth = 0
		}

		if !inFunction {
			continue
		}

		// Track stack operations
		if strings.Contains(line, "pushq") {
			stackDepth++
		}
		if strings.Contains(line, "popq") {
			stackDepth--
		}

		// Check for explicit stack pointer adjustments
		if strings.Contains(line, "subq") && strings.Contains(line, "%rsp") {
			// Extract amount: subq $N, %rsp
			re := regexp.MustCompile(`\$(\d+)`)
			matches := re.FindStringSubmatch(line)
			if len(matches) > 1 {
				// Subtracting from rsp increases stack depth
				// (Not tracking exact amounts, just noting modification)
				stackDepth++
			}
		}
		if strings.Contains(line, "addq") && strings.Contains(line, "%rsp") {
			// Adding to rsp decreases stack depth
			stackDepth--
		}

		// Check balance at function exit
		if strings.Contains(line, "retq") || strings.Contains(line, "ret") {
			if stackDepth < 0 {
				v.addError(i+1, "stack underflow detected", line)
			}
			// Note: Small imbalances might be OK due to frame setup
			// Only flag significant issues
			if stackDepth > 2 {
				v.addWarn(i+1, fmt.Sprintf("potential stack imbalance: depth=%d", stackDepth), line)
			}
			inFunction = false
		}
	}
}

// validateInstructionValidity checks for invalid instruction combinations
func (v *Validator) validateInstructionValidity(lines []string) {
	for i, line := range lines {
		line = strings.TrimSpace(line)

		// Check for invalid immediate values as destinations
		if isInstructionWithDestination(line) {
			parts := strings.Split(line, ",")
			if len(parts) >= 2 {
				dest := strings.TrimSpace(parts[len(parts)-1])
				if strings.HasPrefix(dest, "$") {
					v.addError(i+1, "immediate value cannot be destination", line)
				}
			}
		}

		// Check for invalid memory-to-memory moves
		if strings.HasPrefix(line, "movq") || strings.HasPrefix(line, "mov") {
			parts := strings.Split(line, ",")
			if len(parts) == 2 {
				src := strings.TrimSpace(parts[0][strings.Index(parts[0], " ")+1:])
				dest := strings.TrimSpace(parts[1])

				if isMemoryOperand(src) && isMemoryOperand(dest) {
					v.addError(i+1, "x86-64 doesn't support memory-to-memory moves", line)
				}
			}
		}

		// Check division without proper setup
		if strings.Contains(line, "idivq") || strings.Contains(line, "divq") {
			if i == 0 || !strings.Contains(lines[i-1], "cqto") {
				v.addWarn(i+1, "division without cqto may cause incorrect results", line)
			}
		}
	}
}

// validateMemoryAddressing checks memory addressing mode correctness
func (v *Validator) validateMemoryAddressing(lines []string) {
	// Pattern for memory operands with explicit scale: (%base,%index,scale)
	scaledPattern := regexp.MustCompile(`\(%[a-z0-9]+,%[a-z0-9]+,(\d+)\)`)

	for i, line := range lines {
		matches := scaledPattern.FindAllStringSubmatch(line, -1)
		for _, match := range matches {
			if len(match) > 1 {
				scale := match[1]
				if scale != "1" && scale != "2" && scale != "4" && scale != "8" {
					v.addError(i+1, fmt.Sprintf("invalid scale factor: %s (must be 1, 2, 4, or 8)", scale), line)
				}
			}
		}
	}
}

// Helper functions

func (v *Validator) addError(line int, msg, code string) {
	v.errors = append(v.errors, ValidationError{Line: line, Message: msg, Code: code})
}

func (v *Validator) addWarn(line int, msg, code string) {
	v.warns = append(v.warns, ValidationError{Line: line, Message: msg, Code: code})
}

func (v *Validator) formatErrors() error {
	var sb strings.Builder
	sb.WriteString("Assembly validation failed:\n")
	for _, err := range v.errors {
		sb.WriteString("  " + err.Error() + "\n")
	}
	return fmt.Errorf("%s", sb.String())
}

func (v *Validator) logWarnings() {
	for _, warn := range v.warns {
		logger.Warn("Assembly validation warning", "line", warn.Line, "msg", warn.Message)
	}
}

func isValidInstruction(line string) bool {
	validInsts := []string{
		"mov", "push", "pop", "add", "sub", "imul", "idiv", "cqto",
		"cmp", "test", "set", "jmp", "jnz", "jz", "je", "jne",
		"call", "ret", "lea", "and", "or", "xor", "not", "neg",
		"shl", "shr", "sal", "sar", "inc", "dec", "leave", "enter",
	}

	line = strings.TrimSpace(line)
	for _, inst := range validInsts {
		if strings.HasPrefix(line, inst) {
			return true
		}
	}

	// Also check for directives
	if strings.HasPrefix(line, ".") {
		return true
	}

	return false
}

func isCalleeSaved(reg string) bool {
	for _, r := range CalleeSaved {
		if r == reg {
			return true
		}
	}
	return reg == "%rbp" || reg == "%rsp"
}

func isInstructionWithDestination(line string) bool {
	destInsts := []string{"mov", "add", "sub", "imul", "lea", "and", "or", "xor"}
	line = strings.TrimSpace(line)
	for _, inst := range destInsts {
		if strings.HasPrefix(line, inst) && strings.Contains(line, ",") {
			return true
		}
	}
	return false
}

func isMemoryOperand(operand string) bool {
	return strings.Contains(operand, "(") && strings.Contains(operand, ")")
}

// ValidateProgram validates an entire generated program
func ValidateProgram(assembly string) error {
	validator := NewValidator()
	return validator.Validate(assembly)
}

// QuickValidate performs fast basic validation for development
func QuickValidate(assembly string) bool {
	validator := NewValidator()
	lines := strings.Split(assembly, "\n")

	// Just check syntax and registers for quick feedback
	validator.validateSyntax(lines)
	validator.validateRegisters(lines)

	return len(validator.errors) == 0
}

// ValidateAndReport validates assembly and returns a detailed report
func ValidateAndReport(assembly string) (bool, string) {
	validator := NewValidator()
	err := validator.Validate(assembly)

	var report strings.Builder
	report.WriteString("=== Assembly Validation Report ===\n\n")

	if err != nil {
		report.WriteString(fmt.Sprintf("Status: FAILED\n\nErrors:\n%s\n", err.Error()))
		return false, report.String()
	}

	report.WriteString("Status: PASSED\n\n")

	if len(validator.warns) > 0 {
		report.WriteString("Warnings:\n")
		for _, warn := range validator.warns {
			report.WriteString(fmt.Sprintf("  Line %d: %s\n", warn.Line, warn.Message))
		}
	} else {
		report.WriteString("No warnings.\n")
	}

	// Count instructions
	lineCount := len(strings.Split(assembly, "\n"))
	instCount := 0
	scanner := bufio.NewScanner(strings.NewReader(assembly))
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if strings.HasPrefix(line, "\t") && !strings.HasPrefix(line, "\t.") {
			instCount++
		}
	}

	report.WriteString("\nStatistics:\n")
	report.WriteString(fmt.Sprintf("  Total lines: %d\n", lineCount))
	report.WriteString(fmt.Sprintf("  Instructions: %d\n", instCount))

	logger.Info("Assembly validation passed", "instructions", instCount, "warnings", len(validator.warns))

	return true, report.String()
}
