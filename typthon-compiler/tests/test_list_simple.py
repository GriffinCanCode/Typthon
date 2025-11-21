def sum_list(nums: list) -> int:
    total = 0
    i = 0
    while i < 5:
        total = total + i
        i = i + 1
    return total

def main() -> int:
    return sum_list([])

