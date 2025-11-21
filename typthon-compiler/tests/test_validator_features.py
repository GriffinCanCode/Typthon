"""Test new validator features: caller-saved preservation and redundant moves"""

def test_caller_saved():
    """Function that requires caller-saved register preservation"""
    x = 42
    y = compute(x)  # Call may clobber caller-saved registers
    return x + y

def test_redundant_moves():
    """Function with potential redundant moves"""
    a = 10
    b = a
    c = b
    return c

def compute(n):
    return n * 2

