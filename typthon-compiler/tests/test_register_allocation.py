"""
Test register allocation, phi nodes, spilling, and stack arguments.
"""

# Test 1: Linear scan register allocation with liveness intervals
def test_register_allocation(a: int, b: int, c: int) -> int:
    """Uses multiple temporaries to trigger proper register allocation."""
    t1 = a + b
    t2 = b + c
    t3 = t1 * t2
    t4 = t3 - a
    t5 = t4 + b
    t6 = t5 * c
    t7 = t6 - t1
    return t7

# Test 2: Phi nodes with control flow
def test_phi_nodes(x: int) -> int:
    """Tests phi node resolution at control flow merge points."""
    if x > 10:
        y = x * 2
    else:
        y = x + 5
    # y is a phi node here
    return y + 1

# Test 3: Register spilling (many live values)
def test_spilling(a: int, b: int, c: int, d: int, e: int) -> int:
    """Forces register spilling by having many live variables."""
    x1 = a + b
    x2 = c + d
    x3 = e + a
    x4 = b + c
    x5 = d + e
    x6 = a + c
    x7 = b + d
    x8 = c + e
    x9 = a + d
    x10 = b + e
    # All x1-x10 should be live here - forces spilling
    result = x1 + x2 + x3 + x4 + x5 + x6 + x7 + x8 + x9 + x10
    return result

# Test 4: Stack arguments (>6 on amd64, >8 on arm64)
def test_many_args(a: int, b: int, c: int, d: int, e: int, f: int,
                   g: int, h: int, i: int, j: int) -> int:
    """Tests stack argument passing for arguments beyond register capacity."""
    return a + b + c + d + e + f + g + h + i + j

# Test 5: Combined test - phi nodes with spilling
def test_combined(x: int, y: int, z: int) -> int:
    """Tests phi nodes with enough values to cause spilling."""
    if x > 0:
        a1 = x + 1
        a2 = x + 2
        a3 = x + 3
        a4 = x + 4
        a5 = x + 5
        result = a1 + a2 + a3 + a4 + a5
    else:
        b1 = y + 1
        b2 = y + 2
        b3 = y + 3
        b4 = y + 4
        b5 = y + 5
        result = b1 + b2 + b3 + b4 + b5
    # result is a phi node
    return result + z

