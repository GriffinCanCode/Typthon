// Typthon Runtime Header
#ifndef TYPTHON_RUNTIME_H
#define TYPTHON_RUNTIME_H

#include <stdint.h>

// Runtime functions
void typthon_init(void);
void typthon_cleanup(void);
void typthon_panic(const char* msg);

// Built-in functions
void typthon_print_int(int64_t val);
void print(int64_t val);

#endif // TYPTHON_RUNTIME_H

