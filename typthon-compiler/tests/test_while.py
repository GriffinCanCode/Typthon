def sum_to_n(n: int) -> int:
    result = 0
    i = 0
    while i < n:
        result = result + i
        i = i + 1
    return result

def main() -> int:
    return sum_to_n(10)

