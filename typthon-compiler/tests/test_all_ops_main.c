#include <stdio.h>
#include <stdint.h>

extern int64_t add(int64_t a, int64_t b);
extern int64_t subtract(int64_t a, int64_t b);
extern int64_t multiply(int64_t a, int64_t b);
extern int64_t divide(int64_t a, int64_t b);

int main(void) {
    printf("=== Typthon Codegen Test Suite ===\n\n");

    int passed = 0;
    int total = 0;

    // Test add
    total++;
    int64_t result = add(10, 5);
    if (result == 15) {
        printf("✓ add(10, 5) = %lld\n", result);
        passed++;
    } else {
        printf("✗ add(10, 5) = %lld (expected 15)\n", result);
    }

    // Test subtract
    total++;
    result = subtract(10, 5);
    if (result == 5) {
        printf("✓ subtract(10, 5) = %lld\n", result);
        passed++;
    } else {
        printf("✗ subtract(10, 5) = %lld (expected 5)\n", result);
    }

    // Test multiply
    total++;
    result = multiply(10, 5);
    if (result == 50) {
        printf("✓ multiply(10, 5) = %lld\n", result);
        passed++;
    } else {
        printf("✗ multiply(10, 5) = %lld (expected 50)\n", result);
    }

    // Test divide
    total++;
    result = divide(50, 5);
    if (result == 10) {
        printf("✓ divide(50, 5) = %lld\n", result);
        passed++;
    } else {
        printf("✗ divide(50, 5) = %lld (expected 10)\n", result);
    }

    printf("\n=== Results: %d/%d tests passed ===\n", passed, total);
    return (passed == total) ? 0 : 1;
}

