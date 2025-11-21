"""
Test program to verify compiler optimizations:
1. Peephole optimizations (pattern matching)
2. RISC-V code generation
3. ARM64 immediate form optimizations
"""

def test_peephole_arithmetic(a: int) -> int:
    """Tests peephole optimizations for arithmetic"""
    # Should optimize away add-by-zero
    x = a + 0

    # Should optimize away subtract-by-zero
    y = x - 0

    # Should optimize multiply-by-one
    z = y * 1

    # Should optimize divide-by-one
    result = z / 1

    return result

def test_peephole_multiply(n: int) -> int:
    """Tests multiply optimizations"""
    # Should convert multiply-by-2 to add
    doubled = n * 2

    # Should eliminate multiply-by-zero
    zero = n * 0

    # Should eliminate multiply-by-one
    same = n * 1

    return doubled + zero + same

def test_peephole_logical(val: int) -> int:
    """Tests logical operation optimizations"""
    # Should optimize away and-with-zero
    a = val & 0

    # Should optimize away or-with-zero
    b = val | 0

    # Should optimize away xor-with-zero
    c = val ^ 0

    return a + b + c

def test_arm64_immediates(x: int) -> int:
    """Tests ARM64 immediate form usage"""
    # These should use immediate forms on ARM64 (values < 4096)
    a = x + 10
    b = a - 5
    c = b + 100
    d = c - 1

    # Comparison with immediate
    if d > 42:
        return d + 7

    return d - 3

def test_riscv_operations(a: int, b: int) -> int:
    """Tests RISC-V code generation"""
    # Basic arithmetic
    sum_val = a + b
    diff = a - b
    prod = a * b

    # Comparisons
    if sum_val > diff:
        result = prod + 1
    else:
        result = prod - 1

    # Logical operations
    and_val = a & b
    or_val = a | b
    xor_val = a ^ b

    return result + and_val + or_val + xor_val

def test_complex_optimization(x: int, y: int) -> int:
    """Tests combination of optimizations"""
    # Multiple peephole opportunities
    a = x + 0  # Should optimize
    b = y * 1  # Should optimize
    c = a * 2  # Should convert to add
    d = b - 0  # Should optimize

    # ARM64 immediate forms (if on ARM64)
    e = c + 50
    f = d + 100

    # RISC-V style comparison
    if e > f:
        return e - f
    else:
        return f - e

def test_constant_folding(n: int) -> int:
    """Tests that constant operations are folded"""
    # These constants should be folded by constant folding pass
    const1 = 10 + 20
    const2 = 100 - 30
    const3 = 5 * 6

    # Then used with immediates
    result = n + const1 + const2 + const3
    return result

