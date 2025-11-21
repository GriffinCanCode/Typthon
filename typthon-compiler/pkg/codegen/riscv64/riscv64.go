// Package riscv64 implements RISC-V 64-bit code generation.
//
// Design: Direct assembly generation for RISC-V servers and embedded systems.
// RISC-V RV64I base instruction set with standard calling convention.
// Optimized for the elegant simplicity of RISC-V's load-store architecture.
package riscv64

import (
	"fmt"
	"io"
	"strings"

	"github.com/GriffinCanCode/typthon-compiler/pkg/codegen/regalloc"
	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
	"github.com/GriffinCanCode/typthon-compiler/pkg/ssa"
)

// Generator generates RISC-V 64-bit assembly
type Generator struct {
	w         io.Writer
	alloc     *regalloc.Allocator
	paramMap  map[*ir.Param]int
	stackSize int
	phiMoves  map[*ssa.Block][]phiMove
}

type phiMove struct {
	src  ir.Value
	dest ir.Value
}

func NewGenerator(w io.Writer) *Generator {
	return &Generator{
		w:        w,
		paramMap: make(map[*ir.Param]int),
		phiMoves: make(map[*ssa.Block][]phiMove),
	}
}

// Generate emits assembly for an SSA program
func (g *Generator) Generate(prog *ssa.Program) error {
	logger.Debug("Generating riscv64 assembly", "functions", len(prog.Functions))

	// Emit assembly header
	fmt.Fprintf(g.w, "\t.text\n")
	fmt.Fprintf(g.w, "\t.align 2\n")

	for _, fn := range prog.Functions {
		logger.Debug("Generating function assembly", "arch", "riscv64", "name", fn.Name)
		if err := g.generateFunction(fn); err != nil {
			logger.Error("Failed to generate function", "arch", "riscv64", "name", fn.Name, "error", err)
			return err
		}
	}

	logger.Info("riscv64 code generation complete", "functions", len(prog.Functions))
	return nil
}

// GenerateWithValidation generates and validates assembly
func (g *Generator) GenerateWithValidation(prog *ssa.Program) (string, error) {
	// Generate to a buffer first
	var buf strings.Builder
	g.w = &buf

	if err := g.Generate(prog); err != nil {
		return "", fmt.Errorf("generation failed: %w", err)
	}

	assembly := buf.String()

	// Validate the generated assembly
	if err := ValidateProgram(assembly); err != nil {
		logger.Error("Assembly validation failed", "error", err)
		return assembly, fmt.Errorf("validation failed: %w", err)
	}

	logger.Info("Assembly generated and validated successfully")
	return assembly, nil
}

// generateFunction emits assembly for a single function
func (g *Generator) generateFunction(fn *ssa.Function) error {
	g.paramMap = make(map[*ir.Param]int)
	g.phiMoves = make(map[*ssa.Block][]phiMove)

	instCount := 0
	for _, block := range fn.Blocks {
		instCount += len(block.Insts)
	}
	logger.LogCodeGen("riscv64", fn.Name, instCount)

	// Map parameters to their indices
	if err := g.mapParameters(fn); err != nil {
		return err
	}

	// Perform register allocation
	cfg := &regalloc.Config{
		Available:   []string{"s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11"},
		Reserved:    []string{"a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", "zero", "ra", "sp", "s0"},
		CalleeSaved: SavedRegs,
		CallerSaved: append(ArgRegs, TempRegs...),
	}
	g.alloc = regalloc.NewAllocator(fn, cfg)
	if err := g.alloc.Allocate(); err != nil {
		return fmt.Errorf("register allocation failed: %w", err)
	}

	// Compute stack frame size (spills + stack args)
	g.stackSize = g.alloc.GetStackSize()
	frameSize := g.stackSize + 16 // 16 bytes for ra + s0
	if frameSize > 0 {
		// Align to 16 bytes (required by RISC-V ABI)
		frameSize = (frameSize + 15) & ^15
	}

	// Resolve phi nodes by inserting moves in predecessor blocks
	g.resolvePhi(fn)

	// Prologue
	fmt.Fprintf(g.w, "\t.globl %s\n", fn.Name)
	fmt.Fprintf(g.w, "%s:\n", fn.Name)

	if frameSize > 0 {
		// Save frame pointer and return address
		if frameSize <= 2047 {
			fmt.Fprintf(g.w, "\taddi sp, sp, -%d\n", frameSize)
			fmt.Fprintf(g.w, "\tsd ra, %d(sp)\n", frameSize-8)
			fmt.Fprintf(g.w, "\tsd s0, %d(sp)\n", frameSize-16)
		} else {
			// Large immediate - use li + add
			fmt.Fprintf(g.w, "\tli t0, %d\n", frameSize)
			fmt.Fprintf(g.w, "\tsub sp, sp, t0\n")
			fmt.Fprintf(g.w, "\tli t0, %d\n", frameSize-8)
			fmt.Fprintf(g.w, "\tadd t0, sp, t0\n")
			fmt.Fprintf(g.w, "\tsd ra, 0(t0)\n")
			fmt.Fprintf(g.w, "\tli t0, %d\n", frameSize-16)
			fmt.Fprintf(g.w, "\tadd t0, sp, t0\n")
			fmt.Fprintf(g.w, "\tsd s0, 0(t0)\n")
		}
		fmt.Fprintf(g.w, "\taddi s0, sp, %d\n", frameSize)
	}

	// Save callee-saved registers that we use
	usedCalleeSaved := g.getUsedCalleeSaved()
	offset := 16
	for _, reg := range usedCalleeSaved {
		if offset <= 2047 {
			fmt.Fprintf(g.w, "\tsd %s, %d(sp)\n", reg, offset)
		} else {
			fmt.Fprintf(g.w, "\tli t0, %d\n", offset)
			fmt.Fprintf(g.w, "\tadd t0, sp, t0\n")
			fmt.Fprintf(g.w, "\tsd %s, 0(t0)\n", reg)
		}
		offset += 8
	}

	// Move parameters from arg regs to allocated locations
	g.saveParameters(fn)

	// Generate blocks
	for _, block := range fn.Blocks {
		if err := g.generateBlock(block); err != nil {
			return err
		}
	}

	return nil
}

// mapParameters builds the parameter index map
func (g *Generator) mapParameters(fn *ssa.Function) error {
	for i, param := range fn.Params {
		g.paramMap[param] = i
	}
	return nil
}

// resolvePhi resolves phi nodes by inserting moves in predecessor blocks
func (g *Generator) resolvePhi(fn *ssa.Function) {
	for _, block := range fn.Blocks {
		if len(block.Phis) == 0 {
			continue
		}

		// For each phi, insert moves in predecessor blocks
		for _, phi := range block.Phis {
			for _, phiVal := range phi.Values {
				pred := phiVal.Block
				if g.phiMoves[pred] == nil {
					g.phiMoves[pred] = make([]phiMove, 0)
				}
				g.phiMoves[pred] = append(g.phiMoves[pred], phiMove{
					src:  phiVal.Value,
					dest: phi.Dest,
				})
			}
		}
	}
}

// saveParameters moves parameters from arg registers to allocated locations
func (g *Generator) saveParameters(fn *ssa.Function) {
	for i, param := range fn.Params {
		if i < len(ArgRegs) {
			// Parameter in register
			if reg, ok := g.alloc.GetRegister(param); ok {
				if reg != ArgRegs[i] {
					fmt.Fprintf(g.w, "\tmv %s, %s\n", reg, ArgRegs[i])
				}
			} else if slot, ok := g.alloc.GetSpillSlot(param); ok {
				// Spilled parameter
				if slot <= 2047 {
					fmt.Fprintf(g.w, "\tsd %s, %d(sp)\n", ArgRegs[i], slot)
				} else {
					fmt.Fprintf(g.w, "\tli t0, %d\n", slot)
					fmt.Fprintf(g.w, "\tadd t0, sp, t0\n")
					fmt.Fprintf(g.w, "\tsd %s, 0(t0)\n", ArgRegs[i])
				}
			}
		} else {
			// Parameter on stack (from caller)
			stackOffset := g.stackSize + 16 + (i-len(ArgRegs))*8
			if reg, ok := g.alloc.GetRegister(param); ok {
				if stackOffset <= 2047 {
					fmt.Fprintf(g.w, "\tld %s, %d(s0)\n", reg, stackOffset)
				} else {
					fmt.Fprintf(g.w, "\tli t0, %d\n", stackOffset)
					fmt.Fprintf(g.w, "\tadd t0, s0, t0\n")
					fmt.Fprintf(g.w, "\tld %s, 0(t0)\n", reg)
				}
			} else if slot, ok := g.alloc.GetSpillSlot(param); ok {
				// Load from caller stack and store to our spill area
				if stackOffset <= 2047 && slot <= 2047 {
					fmt.Fprintf(g.w, "\tld t1, %d(s0)\n", stackOffset)
					fmt.Fprintf(g.w, "\tsd t1, %d(sp)\n", slot)
				} else {
					fmt.Fprintf(g.w, "\tli t0, %d\n", stackOffset)
					fmt.Fprintf(g.w, "\tadd t0, s0, t0\n")
					fmt.Fprintf(g.w, "\tld t1, 0(t0)\n")
					fmt.Fprintf(g.w, "\tli t0, %d\n", slot)
					fmt.Fprintf(g.w, "\tadd t0, sp, t0\n")
					fmt.Fprintf(g.w, "\tsd t1, 0(t0)\n")
				}
			}
		}
	}
}

// getUsedCalleeSaved returns callee-saved registers that were allocated
func (g *Generator) getUsedCalleeSaved() []string {
	used := make(map[string]bool)
	calleeSaved := map[string]bool{
		"s1": true, "s2": true, "s3": true, "s4": true,
		"s5": true, "s6": true, "s7": true, "s8": true,
		"s9": true, "s10": true, "s11": true,
	}

	// Check all intervals for callee-saved regs
	for _, block := range g.alloc.GetFunction().Blocks {
		for _, inst := range block.Insts {
			if def := getDef(inst); def != nil {
				if reg, ok := g.alloc.GetRegister(def); ok {
					if calleeSaved[reg] {
						used[reg] = true
					}
				}
			}
		}
	}

	result := make([]string, 0, len(used))
	// Return in order
	for _, reg := range []string{"s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11"} {
		if used[reg] {
			result = append(result, reg)
		}
	}
	return result
}

// generateBlock emits assembly for a basic block
func (g *Generator) generateBlock(block *ssa.Block) error {
	// Emit label (skip for entry block)
	if block.Label != "entry_0" {
		fmt.Fprintf(g.w, ".L%s:\n", block.Label)
	}

	// Emit instructions
	for _, inst := range block.Insts {
		if err := g.generateInst(inst); err != nil {
			return err
		}
	}

	// Emit phi resolution moves before terminator
	if moves, ok := g.phiMoves[block]; ok {
		for _, move := range moves {
			srcLoc := g.getValueLocation(move.src)
			destLoc := g.getValueLocation(move.dest)
			if srcLoc != destLoc {
				// Handle memory-to-memory moves with temp register
				if strings.Contains(srcLoc, "(") && strings.Contains(destLoc, "(") {
					fmt.Fprintf(g.w, "\tld t2, %s\n", srcLoc)
					fmt.Fprintf(g.w, "\tsd t2, %s\n", destLoc)
				} else if strings.Contains(srcLoc, "(") {
					fmt.Fprintf(g.w, "\tld %s, %s\n", destLoc, srcLoc)
				} else if strings.Contains(destLoc, "(") {
					fmt.Fprintf(g.w, "\tsd %s, %s\n", srcLoc, destLoc)
				} else {
					fmt.Fprintf(g.w, "\tmv %s, %s\n", destLoc, srcLoc)
				}
			}
		}
	}

	// Emit terminator
	return g.generateTerm(block.Term)
}

// generateInst emits assembly for an instruction
func (g *Generator) generateInst(inst ir.Inst) error {
	switch i := inst.(type) {
	case *ir.BinOp:
		return g.generateBinOp(i)
	case *ir.Call:
		return g.generateCall(i)
	case *ir.Load:
		return g.generateLoad(i)
	case *ir.Store:
		return g.generateStore(i)
	default:
		return fmt.Errorf("unsupported instruction: %T", inst)
	}
}

// generateBinOp emits assembly for binary operations
func (g *Generator) generateBinOp(binop *ir.BinOp) error {
	leftLoc := g.getValueLocation(binop.L)
	rightLoc := g.getValueLocation(binop.R)
	destLoc := g.getValueLocation(binop.Dest)

	// Load operands into registers if needed
	leftReg := g.ensureInRegister(leftLoc, "t3")
	rightReg := g.ensureInRegister(rightLoc, "t4")
	destReg := destLoc
	if strings.Contains(destLoc, "(") {
		destReg = "t5"
	}

	switch binop.Op {
	// Arithmetic
	case ir.OpAdd:
		fmt.Fprintf(g.w, "\tadd %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpSub:
		fmt.Fprintf(g.w, "\tsub %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpMul:
		fmt.Fprintf(g.w, "\tmul %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpDiv:
		fmt.Fprintf(g.w, "\tdiv %s, %s, %s\n", destReg, leftReg, rightReg)

	// Comparisons - use slt/sltu and xor tricks for RISC-V
	case ir.OpEq:
		fmt.Fprintf(g.w, "\txor %s, %s, %s\n", destReg, leftReg, rightReg)
		fmt.Fprintf(g.w, "\tsltiu %s, %s, 1\n", destReg, destReg)
	case ir.OpNe:
		fmt.Fprintf(g.w, "\txor %s, %s, %s\n", destReg, leftReg, rightReg)
		fmt.Fprintf(g.w, "\tsltu %s, zero, %s\n", destReg, destReg)
	case ir.OpLt:
		fmt.Fprintf(g.w, "\tslt %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpLe:
		// a <= b  ==  !(a > b)  ==  !(b < a)
		fmt.Fprintf(g.w, "\tslt %s, %s, %s\n", destReg, rightReg, leftReg)
		fmt.Fprintf(g.w, "\txori %s, %s, 1\n", destReg, destReg)
	case ir.OpGt:
		fmt.Fprintf(g.w, "\tslt %s, %s, %s\n", destReg, rightReg, leftReg)
	case ir.OpGe:
		// a >= b  ==  !(a < b)
		fmt.Fprintf(g.w, "\tslt %s, %s, %s\n", destReg, leftReg, rightReg)
		fmt.Fprintf(g.w, "\txori %s, %s, 1\n", destReg, destReg)

	// Boolean operations
	case ir.OpAnd:
		fmt.Fprintf(g.w, "\tand %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpOr:
		fmt.Fprintf(g.w, "\tor %s, %s, %s\n", destReg, leftReg, rightReg)
	case ir.OpXor:
		fmt.Fprintf(g.w, "\txor %s, %s, %s\n", destReg, leftReg, rightReg)

	default:
		return fmt.Errorf("unsupported operation: %v", binop.Op)
	}

	// Store result if destination is memory
	if strings.Contains(destLoc, "(") {
		offset, base := parseMemoryOperand(destLoc)
		if offset <= 2047 && offset >= -2048 {
			fmt.Fprintf(g.w, "\tsd %s, %d(%s)\n", destReg, offset, base)
		} else {
			fmt.Fprintf(g.w, "\tli t6, %d\n", offset)
			fmt.Fprintf(g.w, "\tadd t6, %s, t6\n", base)
			fmt.Fprintf(g.w, "\tsd %s, 0(t6)\n", destReg)
		}
	}

	return nil
}

// ensureInRegister loads a value into a register if it's not already
func (g *Generator) ensureInRegister(loc string, tempReg string) string {
	if strings.Contains(loc, "(") {
		// Memory location - load it
		offset, base := parseMemoryOperand(loc)
		if offset <= 2047 && offset >= -2048 {
			fmt.Fprintf(g.w, "\tld %s, %d(%s)\n", tempReg, offset, base)
		} else {
			fmt.Fprintf(g.w, "\tli %s, %d\n", tempReg, offset)
			fmt.Fprintf(g.w, "\tadd %s, %s, %s\n", tempReg, base, tempReg)
			fmt.Fprintf(g.w, "\tld %s, 0(%s)\n", tempReg, tempReg)
		}
		return tempReg
	}
	return loc
}

// parseMemoryOperand parses offset(base) into offset and base
func parseMemoryOperand(loc string) (int, string) {
	// Simple parser for offset(base) format
	if !strings.Contains(loc, "(") {
		return 0, loc
	}
	parts := strings.Split(loc, "(")
	offset := 0
	if len(parts[0]) > 0 {
		fmt.Sscanf(parts[0], "%d", &offset)
	}
	base := strings.TrimSuffix(parts[1], ")")
	return offset, base
}

// generateCall emits assembly for function calls
func (g *Generator) generateCall(call *ir.Call) error {
	// RISC-V ABI: up to 8 args in registers, rest on stack
	numStackArgs := 0
	if len(call.Args) > len(ArgRegs) {
		numStackArgs = len(call.Args) - len(ArgRegs)
		// Align to 16 bytes
		stackBytes := (numStackArgs*8 + 15) & ^15
		if stackBytes > 0 {
			if stackBytes <= 2047 {
				fmt.Fprintf(g.w, "\taddi sp, sp, -%d\n", stackBytes)
			} else {
				fmt.Fprintf(g.w, "\tli t0, %d\n", stackBytes)
				fmt.Fprintf(g.w, "\tsub sp, sp, t0\n")
			}
		}
	}

	// Store stack arguments
	for i := len(ArgRegs); i < len(call.Args); i++ {
		argLoc := g.getValueLocation(call.Args[i])
		offset := (i - len(ArgRegs)) * 8
		argReg := g.ensureInRegister(argLoc, "t0")
		if offset <= 2047 {
			fmt.Fprintf(g.w, "\tsd %s, %d(sp)\n", argReg, offset)
		} else {
			fmt.Fprintf(g.w, "\tli t1, %d\n", offset)
			fmt.Fprintf(g.w, "\tadd t1, sp, t1\n")
			fmt.Fprintf(g.w, "\tsd %s, 0(t1)\n", argReg)
		}
	}

	// Move register arguments
	for i := 0; i < len(call.Args) && i < len(ArgRegs); i++ {
		argLoc := g.getValueLocation(call.Args[i])
		if argLoc != ArgRegs[i] {
			argReg := g.ensureInRegister(argLoc, ArgRegs[i])
			if argReg != ArgRegs[i] {
				fmt.Fprintf(g.w, "\tmv %s, %s\n", ArgRegs[i], argReg)
			}
		}
	}

	// Call function
	fmt.Fprintf(g.w, "\tcall %s\n", call.Function)

	// Clean up stack arguments
	if numStackArgs > 0 {
		stackBytes := (numStackArgs*8 + 15) & ^15
		if stackBytes > 0 {
			if stackBytes <= 2047 {
				fmt.Fprintf(g.w, "\taddi sp, sp, %d\n", stackBytes)
			} else {
				fmt.Fprintf(g.w, "\tli t0, %d\n", stackBytes)
				fmt.Fprintf(g.w, "\tadd sp, sp, t0\n")
			}
		}
	}

	// Move result to destination
	destLoc := g.getValueLocation(call.Dest)
	if destLoc != "a0" {
		if strings.Contains(destLoc, "(") {
			offset, base := parseMemoryOperand(destLoc)
			if offset <= 2047 && offset >= -2048 {
				fmt.Fprintf(g.w, "\tsd a0, %d(%s)\n", offset, base)
			} else {
				fmt.Fprintf(g.w, "\tli t0, %d\n", offset)
				fmt.Fprintf(g.w, "\tadd t0, %s, t0\n", base)
				fmt.Fprintf(g.w, "\tsd a0, 0(t0)\n")
			}
		} else {
			fmt.Fprintf(g.w, "\tmv %s, a0\n", destLoc)
		}
	}

	return nil
}

// generateLoad emits assembly for load instructions
func (g *Generator) generateLoad(load *ir.Load) error {
	srcLoc := g.getValueLocation(load.Src)
	destLoc := g.getValueLocation(load.Dest)
	if srcLoc != destLoc {
		if strings.Contains(srcLoc, "(") && strings.Contains(destLoc, "(") {
			// Memory to memory - use temp
			srcReg := g.ensureInRegister(srcLoc, "t0")
			offset, base := parseMemoryOperand(destLoc)
			if offset <= 2047 && offset >= -2048 {
				fmt.Fprintf(g.w, "\tsd %s, %d(%s)\n", srcReg, offset, base)
			} else {
				fmt.Fprintf(g.w, "\tli t1, %d\n", offset)
				fmt.Fprintf(g.w, "\tadd t1, %s, t1\n", base)
				fmt.Fprintf(g.w, "\tsd %s, 0(t1)\n", srcReg)
			}
		} else if strings.Contains(srcLoc, "(") {
			offset, base := parseMemoryOperand(srcLoc)
			if offset <= 2047 && offset >= -2048 {
				fmt.Fprintf(g.w, "\tld %s, %d(%s)\n", destLoc, offset, base)
			} else {
				fmt.Fprintf(g.w, "\tli t0, %d\n", offset)
				fmt.Fprintf(g.w, "\tadd t0, %s, t0\n", base)
				fmt.Fprintf(g.w, "\tld %s, 0(t0)\n", destLoc)
			}
		} else if strings.Contains(destLoc, "(") {
			srcReg := g.ensureInRegister(srcLoc, "t0")
			offset, base := parseMemoryOperand(destLoc)
			if offset <= 2047 && offset >= -2048 {
				fmt.Fprintf(g.w, "\tsd %s, %d(%s)\n", srcReg, offset, base)
			} else {
				fmt.Fprintf(g.w, "\tli t1, %d\n", offset)
				fmt.Fprintf(g.w, "\tadd t1, %s, t1\n", base)
				fmt.Fprintf(g.w, "\tsd %s, 0(t1)\n", srcReg)
			}
		} else {
			fmt.Fprintf(g.w, "\tmv %s, %s\n", destLoc, srcLoc)
		}
	}
	return nil
}

// generateStore emits assembly for store instructions
func (g *Generator) generateStore(store *ir.Store) error {
	srcLoc := g.getValueLocation(store.Src)
	destLoc := g.getValueLocation(store.Dest)

	if strings.Contains(srcLoc, "(") && strings.Contains(destLoc, "(") {
		// Memory to memory - use temp
		srcReg := g.ensureInRegister(srcLoc, "t0")
		offset, base := parseMemoryOperand(destLoc)
		if offset <= 2047 && offset >= -2048 {
			fmt.Fprintf(g.w, "\tsd %s, %d(%s)\n", srcReg, offset, base)
		} else {
			fmt.Fprintf(g.w, "\tli t1, %d\n", offset)
			fmt.Fprintf(g.w, "\tadd t1, %s, t1\n", base)
			fmt.Fprintf(g.w, "\tsd %s, 0(t1)\n", srcReg)
		}
	} else if strings.Contains(srcLoc, "(") {
		srcReg := g.ensureInRegister(srcLoc, "t0")
		if strings.Contains(destLoc, "(") {
			offset, base := parseMemoryOperand(destLoc)
			if offset <= 2047 && offset >= -2048 {
				fmt.Fprintf(g.w, "\tsd %s, %d(%s)\n", srcReg, offset, base)
			} else {
				fmt.Fprintf(g.w, "\tli t1, %d\n", offset)
				fmt.Fprintf(g.w, "\tadd t1, %s, t1\n", base)
				fmt.Fprintf(g.w, "\tsd %s, 0(t1)\n", srcReg)
			}
		} else {
			fmt.Fprintf(g.w, "\tmv %s, %s\n", destLoc, srcReg)
		}
	} else if strings.Contains(destLoc, "(") {
		srcReg := g.ensureInRegister(srcLoc, "t0")
		offset, base := parseMemoryOperand(destLoc)
		if offset <= 2047 && offset >= -2048 {
			fmt.Fprintf(g.w, "\tsd %s, %d(%s)\n", srcReg, offset, base)
		} else {
			fmt.Fprintf(g.w, "\tli t1, %d\n", offset)
			fmt.Fprintf(g.w, "\tadd t1, %s, t1\n", base)
			fmt.Fprintf(g.w, "\tsd %s, 0(t1)\n", srcReg)
		}
	} else {
		srcReg := g.ensureInRegister(srcLoc, "t0")
		fmt.Fprintf(g.w, "\tmv %s, %s\n", destLoc, srcReg)
	}

	return nil
}

// generateTerm emits assembly for terminator instructions
func (g *Generator) generateTerm(term ir.Terminator) error {
	switch t := term.(type) {
	case *ir.Return:
		// Move return value to a0
		if t.Value != nil {
			valLoc := g.getValueLocation(t.Value)
			if valLoc != "a0" {
				valReg := g.ensureInRegister(valLoc, "a0")
				if valReg != "a0" {
					fmt.Fprintf(g.w, "\tmv a0, %s\n", valReg)
				}
			}
		}

		// Restore callee-saved registers
		usedCalleeSaved := g.getUsedCalleeSaved()
		offset := 16
		for _, reg := range usedCalleeSaved {
			if offset <= 2047 {
				fmt.Fprintf(g.w, "\tld %s, %d(sp)\n", reg, offset)
			} else {
				fmt.Fprintf(g.w, "\tli t0, %d\n", offset)
				fmt.Fprintf(g.w, "\tadd t0, sp, t0\n")
				fmt.Fprintf(g.w, "\tld %s, 0(t0)\n", reg)
			}
			offset += 8
		}

		// Epilogue
		frameSize := g.stackSize + 16
		if frameSize > 0 {
			frameSize = (frameSize + 15) & ^15
			if frameSize <= 2047 {
				fmt.Fprintf(g.w, "\tld ra, %d(sp)\n", frameSize-8)
				fmt.Fprintf(g.w, "\tld s0, %d(sp)\n", frameSize-16)
				fmt.Fprintf(g.w, "\taddi sp, sp, %d\n", frameSize)
			} else {
				fmt.Fprintf(g.w, "\tli t0, %d\n", frameSize-8)
				fmt.Fprintf(g.w, "\tadd t0, sp, t0\n")
				fmt.Fprintf(g.w, "\tld ra, 0(t0)\n")
				fmt.Fprintf(g.w, "\tli t0, %d\n", frameSize-16)
				fmt.Fprintf(g.w, "\tadd t0, sp, t0\n")
				fmt.Fprintf(g.w, "\tld s0, 0(t0)\n")
				fmt.Fprintf(g.w, "\tli t0, %d\n", frameSize)
				fmt.Fprintf(g.w, "\tadd sp, sp, t0\n")
			}
		}
		fmt.Fprintf(g.w, "\tret\n")

	case *ir.Branch:
		fmt.Fprintf(g.w, "\tj .L%s\n", t.Target)

	case *ir.CondBranch:
		condLoc := g.getValueLocation(t.Cond)
		condReg := g.ensureInRegister(condLoc, "t0")
		fmt.Fprintf(g.w, "\tandi %s, %s, 1\n", condReg, condReg)
		fmt.Fprintf(g.w, "\tbnez %s, .L%s\n", condReg, t.TrueBlock)
		fmt.Fprintf(g.w, "\tj .L%s\n", t.FalseBlock)

	default:
		return fmt.Errorf("unsupported terminator: %T", term)
	}

	return nil
}

// getValueLocation returns the register or memory location for a value
func (g *Generator) getValueLocation(val ir.Value) string {
	switch v := val.(type) {
	case *ir.Const:
		// Load immediate using li pseudo-instruction
		return fmt.Sprintf("%d", v.Val)
	case *ir.Temp, *ir.Param:
		// Check if in register
		if reg, ok := g.alloc.GetRegister(val); ok {
			return reg
		}
		// Check if spilled
		if slot, ok := g.alloc.GetSpillSlot(val); ok {
			return fmt.Sprintf("%d(sp)", slot)
		}
		// Fallback - shouldn't happen
		panic(fmt.Sprintf("no location for value: %T", val))
	default:
		panic(fmt.Sprintf("unsupported value type: %T", val))
	}
}

// RISC-V calling convention (RV64I)
var (
	// Argument registers a0-a7
	ArgRegs = []string{"a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7"}
	// Return register
	RetReg = "a0"
	// Saved registers (callee-saved)
	SavedRegs = []string{"s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11"}
	// Temporary registers (caller-saved)
	TempRegs = []string{"t0", "t1", "t2", "t3", "t4", "t5", "t6"}
	// Zero register
	Zero = "zero"
	// Return address
	RetAddr = "ra"
	// Stack pointer
	StackPointer = "sp"
	// Frame pointer
	FramePointer = "s0"
)

// Helper to get definition from instruction
func getDef(inst ir.Inst) ir.Value {
	switch i := inst.(type) {
	case *ir.BinOp:
		return i.Dest
	case *ir.Call:
		return i.Dest
	case *ir.Load:
		return i.Dest
	case *ir.Alloc:
		return i.Dest
	case *ir.AllocObject:
		return i.Dest
	case *ir.GetAttr:
		return i.Dest
	case *ir.GetItem:
		return i.Dest
	case *ir.MethodCall:
		return i.Dest
	case *ir.ClosureCall:
		return i.Dest
	case *ir.MakeClosure:
		return i.Dest
	}
	return nil
}
