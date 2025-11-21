// Test wrapper for add function
#include <stdio.h>
#include <stdint.h>

// Declare the add function from compiled Python
extern int64_t add(int64_t a, int64_t b);

// Override main to test the add function
int main(void) {
    int64_t result = add(3, 5);
    printf("add(3, 5) = %lld\n", result);

    if (result == 8) {
        printf("Test passed!\n");
        return 0;
    } else {
        printf("Test failed! Expected 8, got %lld\n", result);
        return 1;
    }
}

