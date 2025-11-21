// Package riscv64 - Assembly validation and correctness verification
package riscv64

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

// Validator validates generated RISC-V assembly
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
		// Zero register
		"zero": true,
		// Return address
		"ra": true,
		// Stack pointer and frame pointer
		"sp": true, "s0": true, "fp": true,
		// Argument/return registers
		"a0": true, "a1": true, "a2": true, "a3": true,
		"a4": true, "a5": true, "a6": true, "a7": true,
		// Saved registers
		"s1": true, "s2": true, "s3": true, "s4": true,
		"s5": true, "s6": true, "s7": true, "s8": true,
		"s9": true, "s10": true, "s11": true,
		// Temporary registers
		"t0": true, "t1": true, "t2": true, "t3": true,
		"t4": true, "t5": true, "t6": true,
		// Alternative names
		"x0": true, "x1": true, "x2": true, "x3": true,
		"x4": true, "x5": true, "x6": true, "x7": true,
		"x8": true, "x9": true, "x10": true, "x11": true,
		"x12": true, "x13": true, "x14": true, "x15": true,
		"x16": true, "x17": true, "x18": true, "x19": true,
		"x20": true, "x21": true, "x22": true, "x23": true,
		"x24": true, "x25": true, "x26": true, "x27": true,
		"x28": true, "x29": true, "x30": true, "x31": true,
	}

	regPattern := regexp.MustCompile(`\b(zero|ra|sp|s[0-9]+|a[0-7]|t[0-6]|x[0-9]+|fp)\b`)

	for i, line := range lines {
		regs := regPattern.FindAllString(line, -1)
		for _, reg := range regs {
			if !validRegs[reg] {
				v.addError(i+1, fmt.Sprintf("invalid register: %s", reg), line)
			}
		}
	}
}

// validateCallingConvention checks RISC-V ABI compliance
func (v *Validator) validateCallingConvention(lines []string) {
	inFunction := false
	functionName := ""
	savedRegs := make(map[string]bool)
	raS0Saved := false

	for i, line := range lines {
		line = strings.TrimSpace(line)

		// Track function boundaries
		if strings.HasSuffix(line, ":") && !strings.HasPrefix(line, ".L") {
			inFunction = true
			functionName = strings.TrimSuffix(line, ":")
			savedRegs = make(map[string]bool)
			raS0Saved = false
		}

		if !inFunction {
			continue
		}

		// Track sd instructions for ra and s0
		if strings.Contains(line, "sd ra") {
			raS0Saved = true
		}
		if strings.Contains(line, "sd s0") {
			raS0Saved = true
		}

		// Track sd of callee-saved registers
		if strings.Contains(line, "sd") {
			parts := strings.Fields(line)
			if len(parts) >= 2 {
				reg := strings.TrimRight(parts[1], ",")
				if isCalleeSaved(reg) {
					savedRegs[reg] = true
				}
			}
		}

		// Track ld restoration
		if strings.Contains(line, "ld") {
			parts := strings.Fields(line)
			if len(parts) >= 2 {
				reg := strings.TrimRight(parts[1], ",")
				if savedRegs[reg] {
					delete(savedRegs, reg)
				}
				if reg == "ra" || reg == "s0" {
					raS0Saved = false
				}
			}
		}

		// Check for function epilogue
		if strings.Contains(line, "ret") {
			// Verify all saved registers were restored
			if len(savedRegs) > 0 {
				v.addError(i+1, fmt.Sprintf("callee-saved registers not restored in %s: %v", functionName, savedRegs), line)
			}
			if raS0Saved {
				v.addWarn(i+1, "ra/s0 may not be properly restored before ret", line)
			}
			inFunction = false
		}
	}
}

// validateStackBalance checks stack push/pop balance
func (v *Validator) validateStackBalance(lines []string) {
	inFunction := false
	stackAdjustments := 0
	stackAllocPattern := regexp.MustCompile(`addi\s+sp,\s*sp,\s*[0-9]`)

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
		// addi sp, sp, -N (allocate)
		if strings.Contains(line, "addi sp, sp, -") {
			stackAdjustments++
		}
		// addi sp, sp, N (deallocate)
		if stackAllocPattern.MatchString(line) {
			stackAdjustments--
		}
		// sub sp, sp, reg (allocate)
		if strings.Contains(line, "sub sp, sp,") {
			stackAdjustments++
		}
		// add sp, sp, reg (deallocate)
		if strings.Contains(line, "add sp, sp,") && !strings.Contains(line, ", sp,") {
			stackAdjustments--
		}

		// Check balance at function exit
		if strings.Contains(line, "ret") {
			if stackAdjustments > 1 {
				v.addWarn(i+1, fmt.Sprintf("potential stack imbalance: adjustments=%d", stackAdjustments), line)
			}
			if stackAdjustments < 0 {
				v.addError(i+1, "stack underflow detected", line)
			}
			inFunction = false
		}
	}
}

// validateInstructionValidity checks for invalid instruction combinations
func (v *Validator) validateInstructionValidity(lines []string) {
	for i, line := range lines {
		line = strings.TrimSpace(line)

		// Check for invalid register destinations
		if isInstructionWithDestination(line) {
			parts := strings.Fields(line)
			if len(parts) >= 2 {
				dest := strings.TrimRight(parts[1], ",")
				// Can't write to zero register (except it's allowed but has no effect)
				if dest == "zero" || dest == "x0" {
					v.addWarn(i+1, "writing to zero register has no effect", line)
				}
			}
		}

		// Check for mv with same source and destination
		if strings.HasPrefix(line, "mv") {
			parts := strings.Fields(line)
			if len(parts) == 3 {
				dest := strings.TrimRight(parts[1], ",")
				src := parts[2]
				if dest == src {
					v.addWarn(i+1, "redundant move: source equals destination", line)
				}
			}
		}

		// Check for improper immediate sizes
		if strings.Contains(line, "addi") || strings.Contains(line, "li") {
			re := regexp.MustCompile(`[-+]?[0-9]+`)
			matches := re.FindAllString(line, -1)
			for _, match := range matches {
				var val int
				fmt.Sscanf(match, "%d", &val)
				// RISC-V immediate is 12-bit signed for I-type
				if (strings.Contains(line, "addi") || strings.Contains(line, "ld") || strings.Contains(line, "sd")) && (val < -2048 || val > 2047) {
					v.addWarn(i+1, fmt.Sprintf("immediate %d may be out of range for I-type instruction", val), line)
				}
			}
		}

		// Check for division by zero potential
		if strings.Contains(line, "div") || strings.Contains(line, "rem") {
			parts := strings.Fields(line)
			if len(parts) >= 4 {
				divisor := parts[3]
				if divisor == "zero" || divisor == "x0" {
					v.addError(i+1, "division by zero", line)
				}
			}
		}
	}
}

// validateMemoryAddressing checks memory addressing mode correctness
func (v *Validator) validateMemoryAddressing(lines []string) {
	// RISC-V addressing: offset(base) where offset is 12-bit signed immediate
	validAddrPattern := regexp.MustCompile(`-?[0-9]+\((zero|ra|sp|s[0-9]+|a[0-7]|t[0-6]|x[0-9]+|fp)\)`)

	for i, line := range lines {
		// Find all memory operands
		if strings.Contains(line, "(") && (strings.Contains(line, "ld") || strings.Contains(line, "sd") || strings.Contains(line, "lw") || strings.Contains(line, "sw")) {
			// Extract memory operand
			start := strings.Index(line, "(")
			if start > 0 {
				// Find the offset start
				offsetStart := start - 1
				for offsetStart > 0 && (line[offsetStart] >= '0' && line[offsetStart] <= '9' || line[offsetStart] == '-') {
					offsetStart--
				}
				memOp := strings.TrimSpace(line[offsetStart+1 : start+strings.Index(line[start:], ")")+1])

				if !validAddrPattern.MatchString(memOp) {
					v.addError(i+1, fmt.Sprintf("invalid memory addressing mode: %s", memOp), line)
				}
			}
		}
	}
}

// detectRedundantMoves identifies and warns about redundant move instructions
func (v *Validator) detectRedundantMoves(lines []string) {
	for i, line := range lines {
		line = strings.TrimSpace(line)

		if !strings.HasPrefix(line, "mv") {
			continue
		}

		parts := strings.Fields(line)
		if len(parts) != 3 {
			continue
		}

		dest := strings.TrimRight(parts[1], ",")
		src := parts[2]

		// Check for mv reg, reg (same register)
		if dest == src {
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
			if strings.HasPrefix(nextLine, "mv") || strings.HasPrefix(nextLine, "li") {
				nextParts := strings.Fields(nextLine)
				if len(nextParts) >= 2 {
					nextDest := strings.TrimRight(nextParts[1], ",")
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
		// Arithmetic
		"add", "addi", "sub", "mul", "div", "rem",
		"mulh", "mulhu", "mulhsu", "divu", "remu",
		// Logical
		"and", "andi", "or", "ori", "xor", "xori",
		"sll", "slli", "srl", "srli", "sra", "srai",
		// Loads and stores
		"ld", "lw", "lh", "lb", "lwu", "lhu", "lbu",
		"sd", "sw", "sh", "sb",
		// Branches
		"beq", "bne", "blt", "bge", "bltu", "bgeu",
		"beqz", "bnez", "blez", "bgez", "bltz", "bgtz",
		// Jumps
		"jal", "jalr", "j", "jr", "ret", "call",
		// Comparisons
		"slt", "slti", "sltu", "sltiu",
		"seqz", "snez", "sltz", "sgtz",
		// Pseudoinstructions
		"mv", "li", "la", "neg", "not",
		"nop",
		// Set instructions
		"csrr", "csrw", "csrs", "csrc",
		// Atomic (RV64A extension)
		"lr.d", "sc.d", "amoswap.d", "amoadd.d",
		"amoxor.d", "amoand.d", "amoor.d",
		"amomin.d", "amomax.d", "amominu.d", "amomaxu.d",
	}

	line = strings.TrimSpace(line)
	for _, inst := range validInsts {
		if strings.HasPrefix(line, inst) {
			return true
		}
	}

	// Check for directives
	if strings.HasPrefix(line, ".") {
		return true
	}

	return false
}

func isCalleeSaved(reg string) bool {
	calleeSaved := []string{
		"s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11",
		"x8", "x9", "x18", "x19", "x20", "x21", "x22", "x23", "x24", "x25", "x26", "x27",
	}
	for _, r := range calleeSaved {
		if r == reg {
			return true
		}
	}
	return reg == "sp" || reg == "fp"
}

func isInstructionWithDestination(line string) bool {
	destInsts := []string{"add", "addi", "sub", "mul", "div", "and", "or", "xor", "sll", "srl", "sra", "slt", "sltu", "ld", "lw", "mv", "li"}
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
	report.WriteString("=== RISC-V Assembly Validation Report ===\n\n")

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

	logger.Info("RISC-V assembly validation passed", "instructions", instCount, "warnings", len(validator.warns))

	return true, report.String()
}
