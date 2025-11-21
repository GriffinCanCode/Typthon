// Comprehensive test for add function
#include <stdio.h>
#include <stdint.h>

extern int64_t add(int64_t a, int64_t b);

int main(void) {
    struct {
        int64_t a;
        int64_t b;
        int64_t expected;
    } tests[] = {
        {3, 5, 8},
        {0, 0, 0},
        {-1, 1, 0},
        {100, 200, 300},
        {-50, -50, -100},
        {1000000, 2000000, 3000000}
    };

    int passed = 0;
    int failed = 0;

    for (int i = 0; i < sizeof(tests)/sizeof(tests[0]); i++) {
        int64_t result = add(tests[i].a, tests[i].b);
        if (result == tests[i].expected) {
            printf("✓ add(%lld, %lld) = %lld\n", tests[i].a, tests[i].b, result);
            passed++;
        } else {
            printf("✗ add(%lld, %lld) = %lld (expected %lld)\n",
                   tests[i].a, tests[i].b, result, tests[i].expected);
            failed++;
        }
    }

    printf("\nResults: %d passed, %d failed\n", passed, failed);
    return failed;
}

