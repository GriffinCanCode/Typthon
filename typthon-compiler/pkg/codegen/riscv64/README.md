# RISC-V 64-bit Code Generator

High-performance RV64I assembly code generator for the Typthon compiler, designed for the elegant simplicity of RISC-V's load-store architecture.

## Features

### ✓ Load-Store Architecture Optimization
- Efficient handling of RISC-V's strict load-store model
- Smart immediate value handling with `li` pseudo-instruction
- Optimized memory access patterns with proper offset handling

### ✓ Memory Operations
- Direct addressing: `ld/sd dest, offset(base)`
- Immediate loading: `li dest, immediate`
- Large immediate support (>12-bit) via multi-instruction sequences
- Proper handling of memory-to-memory moves

### ✓ Register Allocation
- Linear scan algorithm for fast compilation
- Callee-saved registers: `s0-s11` (12 registers)
- Caller-saved registers: `a0-a7, t0-t6` (15 registers)
- Automatic spilling to stack when under register pressure

### ✓ Stack Management
- Dynamic stack frame allocation with 16-byte alignment
- Frame pointer (`s0`/`fp`) management
- Return address (`ra`) preservation
- Spill slot tracking with offset management

### ✓ Supported Operations

**Arithmetic:**
- Addition (`add`)
- Subtraction (`sub`)
- Multiplication (`mul`)
- Division (`div`)

**Comparisons:**
- Equal (`==`) - implemented via `xor` + `sltiu`
- Not equal (`!=`) - implemented via `xor` + `sltu`
- Less than (`<`) - direct `slt` instruction
- Less than or equal (`<=`) - `slt` + `xori`
- Greater than (`>`) - `slt` with swapped operands
- Greater than or equal (`>=`) - `slt` + `xori`

**Boolean Logic:**
- AND
- OR
- XOR

**Control Flow:**
- Unconditional jumps (`j`)
- Conditional branches (`bnez`, `beqz`)
- Function calls with full ABI compliance
- Returns (`ret`)

## Assembly Validation

The code generator includes a comprehensive assembly validator that checks:

1. **Syntax correctness** - Valid RISC-V instruction formats
2. **Register usage** - Only valid RV64I registers used
3. **Calling convention** - RISC-V ABI compliance
4. **Stack balance** - Proper frame setup/teardown
5. **Memory addressing** - Valid offset(base) addressing modes
6. **Instruction validity** - No invalid operand combinations
7. **Immediate ranges** - 12-bit immediate size validation

### Using the Validator

```go
// Generate with validation
assembly, err := generator.GenerateWithValidation(program)
if err != nil {
    // Handle validation failure
}

// Or validate separately
err := ValidateProgram(assembly)

// Quick validation (syntax and registers only)
if !QuickValidate(assembly) {
    // Handle error
}

// Detailed report
passed, report := ValidateAndReport(assembly)
fmt.Println(report)
```

## Testing

### Run All Tests

```bash
# From the riscv64 directory
bash test_runner.sh
```

### Run Specific Test Categories

```bash
# Unit tests only
go test -run TestArithmetic
go test -run TestMemory
go test -run TestRegister

# Validator tests
go test -run TestValidator

# Benchmarks
go test -bench=.
```

### Individual Tests

```bash
# Arithmetic operations
go test -run TestArithmeticOperations -v

# Function calls with register preservation
go test -run TestFunctionCall -v

# Memory operations
go test -run TestMemoryOperations -v

# Validator tests
go test -run TestValidatorWithGeneratedCode -v
```

## RISC-V ABI Compliance

### Register Usage

**Argument Registers (in order):**
1. `a0`
2. `a1`
3. `a2`
4. `a3`
5. `a4`
6. `a5`
7. `a6`
8. `a7`
9+ on stack

**Return Value:** `a0`

**Caller-Saved (volatile across calls):**
`a0-a7, t0-t6`

**Callee-Saved (must be preserved):**
`s0-s11`

**Special Registers:**
- `zero` (x0) - Always zero
- `ra` (x1) - Return address
- `sp` (x2) - Stack pointer
- `s0`/`fp` (x8) - Frame pointer

**Stack Pointer:** `sp` (16-byte aligned before function calls)

### Function Prologue/Epilogue

```asm
# Prologue
addi sp, sp, -frameSize
sd ra, frameSize-8(sp)
sd s0, frameSize-16(sp)
addi s0, sp, frameSize

# Epilogue
ld ra, frameSize-8(sp)
ld s0, frameSize-16(sp)
addi sp, sp, frameSize
ret
```

## Performance

- **Fast Compilation:** Linear scan register allocation
- **Efficient Code:** Direct assembly generation (no LLVM overhead)
- **Minimal Overhead:** Smart register usage with spilling only when necessary
- **RISC Philosophy:** Simple, regular instruction encoding

## Examples

### Simple Addition

```python
def add(a: int, b: int) -> int:
    return a + b
```

Generates:

```asm
	.text
	.align 2
	.globl add
add:
	addi sp, sp, -16
	sd ra, 8(sp)
	sd s0, 0(sp)
	addi s0, sp, 16
	add s1, a0, a1
	mv a0, s1
	ld ra, 8(sp)
	ld s0, 0(sp)
	addi sp, sp, 16
	ret
```

### Comparison Operations

```python
def compare(x: int, y: int) -> int:
    return x < y
```

Generates (using RISC-V's elegant comparison strategy):

```asm
	.text
	.align 2
	.globl compare
compare:
	addi sp, sp, -16
	sd ra, 8(sp)
	sd s0, 0(sp)
	addi s0, sp, 16
	slt s1, a0, a1  # Set s1 = 1 if a0 < a1, else 0
	mv a0, s1
	ld ra, 8(sp)
	ld s0, 0(sp)
	addi sp, sp, 16
	ret
```

### Function Call with Register Preservation

```python
def compute(x: int, y: int) -> int:
    temp = x + y
    result = helper(temp)
    return result
```

Generates:

```asm
	.text
	.align 2
	.globl compute
compute:
	addi sp, sp, -32
	sd ra, 24(sp)
	sd s0, 16(sp)
	sd s1, 8(sp)
	addi s0, sp, 32
	add s1, a0, a1
	mv a0, s1
	call helper
	mv s2, a0
	mv a0, s2
	ld s1, 8(sp)
	ld ra, 24(sp)
	ld s0, 16(sp)
	addi sp, sp, 32
	ret
```

## Architecture

```
Generator
├── Generate()                  - Main entry point
├── GenerateWithValidation()    - Generate + validate
├── generateFunction()          - Per-function code gen
├── generateBlock()             - Per-basic-block gen
├── generateInst()              - Per-instruction gen
│   ├── generateBinOp()         - Arithmetic/logic ops
│   ├── generateCall()          - Function calls with ABI
│   ├── generateLoad()          - Memory loads
│   └── generateStore()         - Memory stores
├── generateTerm()              - Terminators (branches, returns)
├── ensureInRegister()          - Load values to registers
└── Register Allocation
    ├── Linear scan algorithm
    ├── Spill handling
    └── Callee-saved preservation

Validator
├── Validate()                  - Full validation
├── validateSyntax()            - Instruction format check
├── validateRegisters()         - Register validity check
├── validateCallingConvention() - ABI compliance check
├── validateStackBalance()      - Stack frame balance
├── validateInstructionValidity() - Invalid operand combos
├── validateMemoryAddressing()  - Address mode correctness
└── detectRedundantMoves()      - Optimization opportunities
```

## RISC-V Advantages

### Elegant Design Principles

1. **Load-Store Architecture**
   - Clear separation: compute vs memory access
   - All arithmetic on registers only
   - Predictable performance model

2. **Regular Instruction Encoding**
   - Few instruction formats
   - Consistent operand ordering
   - Easy to decode and validate

3. **Simple Comparison Model**
   - Single `slt` instruction + combinations
   - No complex flag register
   - Composable comparison operations

4. **Scalable Register File**
   - 31 general-purpose registers
   - 12 callee-saved (excellent for optimizations)
   - Clear register roles

5. **Immediate Handling**
   - 12-bit immediate for most instructions
   - Pseudo-instructions for larger values
   - Consistent sign extension

## Implementation Highlights

### Sophisticated Comparison Implementation

RISC-V doesn't have dedicated comparison instructions like x86. Instead, it uses an elegant composition strategy:

- **Equal**: `xor + sltiu` (xor produces 0 if equal, sltiu tests for 0)
- **Not Equal**: `xor + sltu` (xor produces non-zero if different)
- **Less Than**: Direct `slt`
- **Less/Greater Equal**: `slt + xori` (invert the result)
- **Greater Than**: `slt` with swapped operands

### Smart Immediate Handling

```go
// Small immediate (<12 bits): Direct instruction
addi a0, a1, 100

// Large immediate (>12 bits): li pseudo-instruction
li t0, 5000
add a0, a1, t0
```

### Memory Access Optimization

- Checks if offset fits in 12-bit signed immediate (-2048 to 2047)
- Falls back to multi-instruction sequence for large offsets
- Efficient memory-to-memory moves via temporary registers

## Testing Strategy

### Comprehensive Coverage

- **Unit Tests**: Every instruction type, edge cases
- **Integration Tests**: Complex functions, multiple blocks
- **Validator Tests**: All validation rules
- **Benchmarks**: Performance regression detection

### Test Categories

1. Arithmetic operations (add, sub, mul, div)
2. Comparison operations (all 6 types)
3. Boolean operations (and, or, xor)
4. Function calls (argument passing, register preservation)
5. Memory operations (load, store, spilling)
6. Control flow (branches, jumps)
7. Register allocation (pressure, spilling)
8. ABI compliance (calling convention)

## Future Enhancements

- [ ] RV64M extension (multiply/divide instructions)
- [ ] RV64A extension (atomic operations)
- [ ] RV64F/D extensions (floating point)
- [ ] Compressed instruction set (RV64C)
- [ ] Advanced peephole optimizations
- [ ] Instruction scheduling
- [ ] Profile-guided optimizations

## Contributing

When adding new features:

1. Add unit tests in `riscv64_test.go`
2. Add validation rules in `validator.go` if needed
3. Run `./test_runner.sh` to verify all tests pass
4. Ensure generated code validates successfully
5. Add benchmarks for performance-critical paths
6. Update this README with new capabilities

## References

- [RISC-V Instruction Set Manual](https://riscv.org/technical/specifications/)
- [RISC-V Calling Convention](https://github.com/riscv-non-isa/riscv-elf-psabi-doc)
- [RISC-V Assembly Programmer's Manual](https://github.com/riscv-non-isa/riscv-asm-manual)
- [The RISC-V Reader](http://www.riscvbook.com/)

## Design Philosophy

> "Simplicity is the ultimate sophistication." - Leonardo da Vinci

The RISC-V code generator embodies this philosophy:

- **Elegant instruction set** → Simple, regular code generation
- **Load-store architecture** → Clear separation of concerns
- **Rich register file** → Better optimization opportunities
- **Composable operations** → Flexible implementation strategies

The sophistication lies not in complexity, but in the thoughtful composition of simple, powerful primitives.

