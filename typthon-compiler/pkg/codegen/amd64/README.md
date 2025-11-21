# AMD64 Code Generator

High-performance x86-64 assembly code generator for the Typthon compiler.

## Features

### ✓ Caller-Saved Register Handling
- Automatic preservation of caller-saved registers across function calls
- Push/pop sequences generated for registers holding live values
- Complies with System V ABI calling convention

### ✓ Memory Operations
- Direct addressing: `movq %src, %dest`
- Indirect addressing: `movq (%src), %dest`
- Offset addressing: `movq offset(%base), %dest`
- Pointer type detection for automatic indirect loads/stores

### ✓ Register Allocation
- Linear scan algorithm for fast compilation
- Callee-saved registers: `%rbx, %r12-r15`
- Caller-saved registers: `%rax, %rcx, %rdx, %rsi, %rdi, %r8-r11`
- Automatic spilling to stack when under register pressure

### ✓ Stack Management
- Dynamic stack frame allocation
- Spill slot tracking with offset management
- Stack balance verification

### ✓ Supported Operations

**Arithmetic:**
- Addition (`add`)
- Subtraction (`sub`)
- Multiplication (`imul`)
- Division (`idiv` with proper `cqto` setup)

**Comparisons:**
- Equal (`==`)
- Not equal (`!=`)
- Less than (`<`)
- Less than or equal (`<=`)
- Greater than (`>`)
- Greater than or equal (`>=`)

**Boolean Logic:**
- AND
- OR
- XOR

**Control Flow:**
- Unconditional branches
- Conditional branches
- Function calls with full ABI compliance
- Returns

## Assembly Validation

The code generator includes a comprehensive assembly validator that checks:

1. **Syntax correctness** - Valid instruction formats and operands
2. **Register usage** - Only valid x86-64 registers used
3. **Calling convention** - System V ABI compliance (callee-saved registers)
4. **Caller-saved preservation** - Live caller-saved registers preserved across calls
5. **Stack balance** - Push/pop matching, frame setup/teardown
6. **Memory addressing** - Valid addressing modes and scale factors
7. **Instruction validity** - No invalid operand combinations
8. **Redundant move elimination** - Detects and warns about redundant move instructions

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
# From the amd64 directory
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

## System V ABI Compliance

### Register Usage

**Argument Registers (in order):**
1. `%rdi`
2. `%rsi`
3. `%rdx`
4. `%rcx`
5. `%r8`
6. `%r9`
7+ on stack

**Return Value:** `%rax`

**Caller-Saved (must preserve across calls):**
`%rax, %rcx, %rdx, %rsi, %rdi, %r8, %r9, %r10, %r11`

**Callee-Saved (function must preserve):**
`%rbx, %r12, %r13, %r14, %r15, %rbp`

**Stack Pointer:** `%rsp` (16-byte aligned before `call`)

### Function Prologue/Epilogue

```asm
# Prologue
pushq %rbp
movq %rsp, %rbp
# ... allocate stack space if needed ...

# Epilogue
popq %rbp
retq
```

## Performance

- **Fast Compilation:** Linear scan register allocation
- **Efficient Code:** Direct assembly generation (no LLVM overhead)
- **Minimal Overhead:** Caller-saved register preservation only when necessary

## Examples

### Simple Addition

```python
def add(a: int, b: int) -> int:
    return a + b
```

Generates:

```asm
	.text
	.globl _add
_add:
	pushq %rbp
	movq %rsp, %rbp
	movq %rdi, %rbx
	addq %rsi, %rbx
	movq %rbx, %rax
	popq %rbp
	retq
```

### Function Call with Register Preservation

```python
def compute(x: int, y: int) -> int:
    temp = x + y
    result = helper(temp)
    return result
```

Generates (with caller-saved register handling):

```asm
	.text
	.globl _compute
_compute:
	pushq %rbp
	movq %rsp, %rbp
	movq %rdi, %rbx
	addq %rsi, %rbx
	pushq %rbx          # Save caller-saved if needed
	movq %rbx, %rdi     # First argument
	callq _helper
	popq %rbx           # Restore
	movq %rax, %r12
	movq %r12, %rax
	popq %rbp
	retq
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
│   ├── generateCall()          - Function calls with register save/restore
│   ├── generateLoad()          - Memory loads (multiple addressing modes)
│   └── generateStore()         - Memory stores (multiple addressing modes)
├── generateTerm()              - Terminators (branches, returns)
└── Register Allocation
    ├── allocReg()              - Allocate register for value
    ├── valueReg()              - Get register/immediate for value
    ├── spillToStack()          - Spill to stack under pressure
    └── loadFromStack()         - Reload from spill slot

Validator
├── Validate()                  - Full validation
├── validateSyntax()            - Instruction format check
├── validateRegisters()         - Register validity check
├── validateCallingConvention() - ABI compliance check
├── validateStackBalance()      - Stack push/pop balance
├── validateInstructionValidity() - Invalid operand combos
└── validateMemoryAddressing()  - Addressing mode correctness
```

## Future Enhancements

- [ ] ARM64 backend (similar structure)
- [ ] RISC-V backend
- [ ] Advanced register allocation (graph coloring)
- [ ] Peephole optimizations
- [ ] SIMD instruction support (SSE, AVX)
- [ ] Profile-guided optimizations

## Contributing

When adding new features:

1. Add unit tests in `amd64_test.go`
2. Add validation rules in `validator.go` if needed
3. Run `./test_runner.sh` to verify all tests pass
4. Ensure generated code validates successfully
5. Add benchmarks for performance-critical paths

## References

- [System V AMD64 ABI](https://refspecs.linuxbase.org/elf/x86_64-abi-0.99.pdf)
- [Intel x86-64 Manual](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
- [Go Compiler SSA Backend](https://github.com/golang/go/tree/master/src/cmd/compile/internal/ssa)

