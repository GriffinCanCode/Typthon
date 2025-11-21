def test_if(x: int) -> int:
    if x > 10:
        return 1
    else:
        return 0

def main() -> int:
    return test_if(15)

