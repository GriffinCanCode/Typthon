// Package arm64 - Assembly validation and correctness verification
package arm64

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

// Validator validates generated ARM64 assembly
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
	v.detectRedundantMoves(lines)

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
		if line == "" || strings.HasPrefix(line, "#") || strings.HasPrefix(line, "//") {
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
		// General purpose 64-bit registers
		"x0": true, "x1": true, "x2": true, "x3": true,
		"x4": true, "x5": true, "x6": true, "x7": true,
		"x8": true, "x9": true, "x10": true, "x11": true,
		"x12": true, "x13": true, "x14": true, "x15": true,
		"x16": true, "x17": true, "x18": true, "x19": true,
		"x20": true, "x21": true, "x22": true, "x23": true,
		"x24": true, "x25": true, "x26": true, "x27": true,
		"x28": true, "x29": true, "x30": true,
		// 32-bit registers
		"w0": true, "w1": true, "w2": true, "w3": true,
		"w4": true, "w5": true, "w6": true, "w7": true,
		"w8": true, "w9": true, "w10": true, "w11": true,
		"w12": true, "w13": true, "w14": true, "w15": true,
		// Special registers
		"sp": true, "xzr": true, "wzr": true, "lr": true, "fp": true,
	}

	// Extended pattern for SIMD/SVE registers
	regPattern := regexp.MustCompile(`\b(x[0-9]+|w[0-9]+|v[0-9]+|z[0-9]+|p[0-9]+|sp|xzr|wzr|lr|fp)\b`)

	for i, line := range lines {
		regs := regPattern.FindAllString(line, -1)
		for _, reg := range regs {
			// Check general purpose registers
			if !validRegs[reg] {
				// Check NEON/SIMD registers (v0-v31)
				if strings.HasPrefix(reg, "v") {
					// Valid NEON register
					continue
				}
				// Check SVE registers (z0-z31 for vectors, p0-p15 for predicates)
				if strings.HasPrefix(reg, "z") || strings.HasPrefix(reg, "p") {
					// Valid SVE register
					continue
				}
				v.addError(i+1, fmt.Sprintf("invalid register: %s", reg), line)
			}
		}
	}
}

// validateCallingConvention checks AAPCS64 compliance
func (v *Validator) validateCallingConvention(lines []string) {
	inFunction := false
	functionName := ""
	savedRegs := make(map[string]bool)
	fpLrSaved := false

	for i, line := range lines {
		line = strings.TrimSpace(line)

		// Track function boundaries
		if strings.HasSuffix(line, ":") && !strings.HasPrefix(line, ".L") {
			inFunction = true
			functionName = strings.TrimSuffix(line, ":")
			savedRegs = make(map[string]bool)
			fpLrSaved = false
		}

		if !inFunction {
			continue
		}

		// Track stp/ldp of x29, x30 (frame pointer and link register)
		if strings.Contains(line, "stp") && strings.Contains(line, "x29") && strings.Contains(line, "x30") {
			fpLrSaved = true
		}

		// Track stp of callee-saved registers
		if strings.Contains(line, "stp") {
			parts := strings.Fields(line)
			if len(parts) >= 3 {
				// Extract registers from stp instruction
				regs := strings.Split(strings.TrimRight(parts[1], ","), ",")
				for _, reg := range regs {
					reg = strings.TrimSpace(reg)
					if isCalleeSaved(reg) {
						savedRegs[reg] = true
					}
				}
			}
		}

		// Track str of callee-saved registers
		if strings.Contains(line, "str") && !strings.Contains(line, "[") {
			parts := strings.Fields(line)
			if len(parts) >= 2 {
				reg := strings.TrimRight(parts[1], ",")
				if isCalleeSaved(reg) {
					savedRegs[reg] = true
				}
			}
		}

		// Track ldp of callee-saved registers (restoration)
		if strings.Contains(line, "ldp") {
			parts := strings.Fields(line)
			if len(parts) >= 3 {
				regs := strings.Split(strings.TrimRight(parts[1], ","), ",")
				for _, reg := range regs {
					reg = strings.TrimSpace(reg)
					if savedRegs[reg] {
						delete(savedRegs, reg)
					}
				}
			}
			// Check for x29, x30 restoration
			if strings.Contains(line, "x29") && strings.Contains(line, "x30") {
				fpLrSaved = false
			}
		}

		// Track ldr restoration
		if strings.Contains(line, "ldr") && !strings.Contains(line, "=") {
			parts := strings.Fields(line)
			if len(parts) >= 2 {
				reg := strings.TrimRight(parts[1], ",")
				if savedRegs[reg] {
					delete(savedRegs, reg)
				}
			}
		}

		// Check for function epilogue
		if strings.Contains(line, "ret") {
			// Verify all saved registers were restored
			if len(savedRegs) > 0 {
				v.addError(i+1, fmt.Sprintf("callee-saved registers not restored in %s: %v", functionName, savedRegs), line)
			}
			if fpLrSaved {
				v.addWarn(i+1, "frame pointer and link register may not be properly restored", line)
			}
			inFunction = false
		}
	}
}

// validateStackBalance checks stack push/pop balance
func (v *Validator) validateStackBalance(lines []string) {
	inFunction := false
	stackAdjustments := 0

	for i, line := range lines {
		line = strings.TrimSpace(line)

		// Track function boundaries
		if strings.HasSuffix(line, ":") && !strings.HasPrefix(line, ".L") {
			inFunction = true
			stackAdjustments = 0
		}

		if !inFunction {
			continue
		}

		// Track stack pointer adjustments
		if strings.Contains(line, "sub") && strings.Contains(line, "sp") {
			stackAdjustments++
		}
		if strings.Contains(line, "add") && strings.Contains(line, "sp") && !strings.Contains(line, "sp, sp") {
			stackAdjustments--
		}

		// Track stp with pre-decrement (pushes)
		if strings.Contains(line, "stp") && strings.Contains(line, "[sp,") && strings.Contains(line, "]!") {
			stackAdjustments++
		}

		// Track ldp with post-increment (pops)
		if strings.Contains(line, "ldp") && strings.Contains(line, "[sp]") {
			stackAdjustments--
		}

		// Check balance at function exit
		if strings.Contains(line, "ret") {
			if stackAdjustments > 1 {
				v.addWarn(i+1, fmt.Sprintf("potential stack imbalance: adjustments=%d", stackAdjustments), line)
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
				dest := strings.TrimSpace(parts[0][strings.Index(parts[0], " ")+1:])
				if strings.HasPrefix(dest, "#") {
					v.addError(i+1, "immediate value cannot be destination", line)
				}
			}
		}

		// Check for mov with same source and destination
		if strings.HasPrefix(line, "mov") {
			parts := strings.Split(line, ",")
			if len(parts) == 2 {
				src := strings.TrimSpace(parts[1])
				dest := strings.TrimSpace(parts[0][strings.Index(parts[0], " ")+1:])
				if src == dest {
					v.addWarn(i+1, "redundant move: source equals destination", line)
				}
			}
		}

		// Check for cmp followed by inappropriate instructions
		if strings.HasPrefix(line, "cmp") && i+1 < len(lines) {
			nextLine := strings.TrimSpace(lines[i+1])
			if !strings.HasPrefix(nextLine, "cset") && !strings.HasPrefix(nextLine, "b.") && !strings.HasPrefix(nextLine, "b ") {
				v.addWarn(i+1, "cmp instruction not followed by conditional operation", line)
			}
		}
	}
}

// validateMemoryAddressing checks memory addressing mode correctness
func (v *Validator) validateMemoryAddressing(lines []string) {
	// ARM64 addressing modes: [base], [base, #offset], [base, index], [base, #offset]!
	validAddrPattern := regexp.MustCompile(`\[(x[0-9]+|sp)(,\s*#-?[0-9]+)?\]!?`)

	for i, line := range lines {
		// Find all memory operands
		if strings.Contains(line, "[") {
			// Extract memory operand
			start := strings.Index(line, "[")
			end := strings.Index(line[start:], "]")
			if end == -1 {
				v.addError(i+1, "malformed memory addressing: unclosed bracket", line)
				continue
			}
			memOp := line[start : start+end+1]

			// Check for post-index marker
			if start+end+1 < len(line) && line[start+end+1] == '!' {
				memOp += "!"
			}

			if !validAddrPattern.MatchString(memOp) {
				v.addError(i+1, fmt.Sprintf("invalid memory addressing mode: %s", memOp), line)
			}
		}
	}
}

// detectRedundantMoves identifies and warns about redundant move instructions
func (v *Validator) detectRedundantMoves(lines []string) {
	for i, line := range lines {
		line = strings.TrimSpace(line)

		if !strings.HasPrefix(line, "mov") {
			continue
		}

		parts := strings.Split(line, ",")
		if len(parts) != 2 {
			continue
		}

		src := strings.TrimSpace(parts[1])
		dest := strings.TrimSpace(parts[0][strings.Index(parts[0], " ")+1:])

		// Check for mov reg, reg (same register)
		if src == dest {
			v.addWarn(i+1, fmt.Sprintf("redundant move: source and destination are identical (%s)", src), line)
			continue
		}

		// Check for repeated identical moves
		if i+1 < len(lines) {
			nextLine := strings.TrimSpace(lines[i+1])
			if nextLine == line {
				v.addWarn(i+2, "duplicate move instruction", nextLine)
			}
		}

		// Check for move followed by immediate overwrite
		if i+1 < len(lines) {
			nextLine := strings.TrimSpace(lines[i+1])
			if strings.HasPrefix(nextLine, "mov") {
				nextParts := strings.Split(nextLine, ",")
				if len(nextParts) == 2 {
					nextDest := strings.TrimSpace(nextParts[0][strings.Index(nextParts[0], " ")+1:])
					if dest == nextDest {
						v.addWarn(i+1, "move immediately overwritten by next instruction", line)
					}
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
		"mov", "mvn", "add", "sub", "mul", "sdiv", "udiv",
		"ldr", "str", "ldp", "stp",
		"cmp", "cmn", "tst", "cset",
		"b", "bl", "ret", "br", "blr",
		"and", "orr", "eor", "bic",
		"lsl", "lsr", "asr", "ror",
		"sxtb", "sxth", "sxtw", "uxtb", "uxth",
		"madd", "msub", "smull", "umull",
		"adrp", "adr",
		// NEON SIMD instructions
		"ld1", "st1", "dup", "mla", "mls",
		"cmeq", "cmgt", "cmge", "cmlt", "cmle",
		"fadd", "fsub", "fmul",
		// SVE instructions
		"whilelt", "incs", "incd", "cntd",
		"addv", "mulv",
		// Pointer authentication
		"pacia", "pacib", "pacda", "pacdb",
		"autia", "autib", "autda", "autdb",
		"paciasp", "pacibsp", "autiasp", "autibsp",
		"retaa", "retab", "blraa", "blrab",
		"xpac", "xpaci", "xpacd",
		// Prefetch
		"prfm", "prfum",
	}

	line = strings.TrimSpace(line)
	for _, inst := range validInsts {
		if strings.HasPrefix(line, inst) {
			return true
		}
	}

	// Check for conditional branches
	if strings.HasPrefix(line, "b.") {
		return true
	}

	// Check for directives
	if strings.HasPrefix(line, ".") {
		return true
	}

	return false
}

func isCalleeSaved(reg string) bool {
	calleeSaved := []string{
		"x19", "x20", "x21", "x22", "x23", "x24", "x25", "x26", "x27", "x28",
	}
	for _, r := range calleeSaved {
		if r == reg {
			return true
		}
	}
	return reg == "x29" || reg == "x30" || reg == "fp" || reg == "lr"
}

func isInstructionWithDestination(line string) bool {
	destInsts := []string{"mov", "add", "sub", "mul", "and", "orr", "eor", "ldr", "lsl", "lsr"}
	line = strings.TrimSpace(line)
	for _, inst := range destInsts {
		if strings.HasPrefix(line, inst) && strings.Contains(line, ",") {
			return true
		}
	}
	return false
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
	report.WriteString("=== ARM64 Assembly Validation Report ===\n\n")

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

	logger.Info("ARM64 assembly validation passed", "instructions", instCount, "warnings", len(validator.warns))

	return true, report.String()
}
