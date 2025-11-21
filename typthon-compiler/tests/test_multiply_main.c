#include <stdio.h>
#include <stdint.h>

extern int64_t multiply(int64_t x, int64_t y);

int main(void) {
    int64_t result = multiply(7, 8);
    printf("multiply(7, 8) = %lld\n", result);
    return (result == 56) ? 0 : 1;
}

