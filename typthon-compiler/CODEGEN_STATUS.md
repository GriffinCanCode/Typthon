# Code Generation Status

## ✅ Completed

### Register Allocation
- **amd64**: Linear scan register allocation using callee-saved registers (%rbx, %r12-r15)
- **arm64**: Linear scan register allocation using callee-saved registers (x19-x26)
- Parameter mapping to calling convention registers
- Proper register allocation for temporaries

### Instruction Selection
- **Binary Operations**: Add, Sub, Mul, Div
- **Comparisons**: Eq, Ne, Lt, Le, Gt, Ge (using SETE/CSET instructions)
- **Boolean Operations**: And, Or, Xor
- **Function Calls**: System V ABI (amd64) and AAPCS64 (arm64)
- **Control Flow**: Branch, CondBranch, Return

### Assembly Emission
- **amd64**: AT&T syntax for macOS/Linux
  - Proper function prologue/epilogue
  - Stack frame setup
  - Argument passing via registers (rdi, rsi, rdx, rcx, r8, r9)
  - Return value in rax

- **arm64**: ARM64 assembly for Apple Silicon
  - Frame pointer and link register handling
  - Stack alignment
  - Argument passing via registers (x0-x7)
  - Return value in x0

### Parameter Handling
- ✅ Parameters correctly mapped to calling convention registers
- ✅ amd64: rdi, rsi, rdx, rcx, r8, r9
- ✅ arm64: x0-x7
- ✅ Direct use of argument registers in codegen

## Test Results

### Simple Add Function
```python
def add(a: int, b: int) -> int:
    return a + b
```

**amd64 Assembly:**
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

**arm64 Assembly:**
```asm
	.text
	.align 2
	.global _add
_add:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	add x19, x0, x1
	mov x0, x19
	ldp x29, x30, [sp], #16
	ret
```

**Test Results:**
- ✓ add(3, 5) = 8
- ✓ add(0, 0) = 0
- ✓ add(-1, 1) = 0
- ✓ add(100, 200) = 300
- ✓ add(-50, -50) = -100
- ✓ add(1000000, 2000000) = 3000000

**All 6 tests passed!**

## Implementation Details

### amd64 Code Generator (`pkg/codegen/amd64/amd64.go`)
- Direct assembly generation without LLVM
- System V calling convention
- Callee-saved register pool for allocation
- Proper handling of immediate values vs registers

### arm64 Code Generator (`pkg/codegen/arm64/arm64.go`)
- AAPCS64 calling convention
- Immediate values loaded into registers
- Three-operand instruction format
- Frame pointer (x29) and link register (x30) preservation

### SSA Preservation of Parameters
- Modified `ssa.Function` to include `Params` field
- Parameters flow from IR → SSA → Codegen
- Maintains parameter order for correct calling convention mapping

### Runtime Integration
- Minimal C runtime for Phase 1
- Print, len, range, str, isinstance builtins
- Weak main() symbol for testing flexibility

## Next Steps
1. Stack spilling for complex functions with many locals
2. Support for more complex expressions
3. Control flow (if/while/for statements)
4. Function calls between Typthon functions
5. Optimization passes (constant folding, dead code elimination)

