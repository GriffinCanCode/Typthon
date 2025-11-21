class Counter:
    def __init__(self, start: int) -> int:
        self.value = start
        return 0

    def increment(self) -> int:
        self.value = self.value + 1
        return self.value

    def get(self) -> int:
        return self.value

def main() -> int:
    return 42

