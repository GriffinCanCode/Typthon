// Typthon Runtime - Minimal Phase 1
// Design: Bare minimum for integer arithmetic programs
// Future: GC, allocator, builtins

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>

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

// Print string
void typthon_print_str(const char* str) {
    printf("%s\n", str);
}

// Built-in: print (supports both int and strings)
void print(int64_t val) {
    typthon_print_int(val);
}

// Built-in: len (returns length of collection)
int64_t len(void* obj) {
    if (obj == NULL) {
        typthon_panic("len() called on NULL object");
    }

    // TODO: Implement proper type dispatch
    // For Phase 1, we don't support collections yet
    typthon_panic("len() not yet implemented");
    return 0;
}

// Built-in: range (returns iterator end value for simple loops)
int64_t range(int64_t end) {
    return end;
}

// Built-in: str (convert int to string - returns allocated pointer)
char* str(int64_t val) {
    static char buf[32];
    snprintf(buf, sizeof(buf), "%lld", val);
    return buf;
}

// Built-in: isinstance (simple type check)
int64_t isinstance(void* obj, int64_t type_id) {
    if (obj == NULL) {
        return 0;
    }

    // Check if obj is a small int (tagged pointer check)
    // If bit 0-1 == 01, it's a small int
    uintptr_t bits = (uintptr_t)obj;
    if ((bits & 0x3) == 0x1) {
        return type_id == 2; // 2 = Int type
    }

    // Check if it's a special value (None, bool)
    if ((bits & 0x3) == 0x3) {
        uint8_t special_type = (bits >> 2) & 0x3F;
        if (special_type == 0) return type_id == 0; // None
        if (special_type <= 2) return type_id == 1; // Bool
        return 0;
    }

    // Heap object - check type tag from header
    uint8_t obj_type = *(uint8_t*)((uintptr_t)obj - 16);
    return obj_type == (uint8_t)type_id;
}

// Main entry point - can be overridden by test wrappers
// If no main is provided, we just initialize and cleanup
__attribute__((weak)) int64_t main(void) {
    return 0;
}

