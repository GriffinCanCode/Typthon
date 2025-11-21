// Typthon Runtime - Minimal Phase 1
// Design: Bare minimum for integer arithmetic programs
// Future: GC, allocator, builtins

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

// Runtime initialization (called before main)
void typthon_init(void) {
    // Future: Initialize GC, allocator
}

// Runtime cleanup (called after main)
void typthon_cleanup(void) {
    // Future: Cleanup resources
}

// Panic handler - called when runtime error occurs
void typthon_panic(const char* msg) {
    fprintf(stderr, "panic: %s\n", msg);
    exit(1);
}

// Print integer (built-in function for testing)
void typthon_print_int(int64_t val) {
    printf("%lld\n", val);
}

// Built-in: print
void print(int64_t val) {
    typthon_print_int(val);
}

// Entry point wrapper - ensures init/cleanup
// User-defined main() will be called by this
extern int64_t main(void);

int _start(void) {
    typthon_init();
    int64_t result = main();
    typthon_cleanup();
    exit((int)result);
}

