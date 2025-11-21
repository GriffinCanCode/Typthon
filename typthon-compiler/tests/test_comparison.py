def test_comparisons() -> int:
    a = 5
    b = 10
    if a < b:
        if a == 5:
            if b >= 10:
                return 42
    return 0

def main() -> int:
    return test_comparisons()

