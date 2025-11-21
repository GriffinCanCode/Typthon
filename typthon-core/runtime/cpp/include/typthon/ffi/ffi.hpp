#pragma once
#include "../core/types.hpp"

extern "C" {

// C ABI for Rust FFI
struct CTypeSet {
    void* inner;
};

CTypeSet* typeset_new();
void typeset_free(CTypeSet* set);
void typeset_insert(CTypeSet* set, uint64_t id);
bool typeset_contains(const CTypeSet* set, uint64_t id);
CTypeSet* typeset_union(const CTypeSet* a, const CTypeSet* b);
CTypeSet* typeset_intersection(const CTypeSet* a, const CTypeSet* b);
bool typeset_is_subset(const CTypeSet* a, const CTypeSet* b);
size_t typeset_cardinality(const CTypeSet* set);

bool type_is_subtype(uint64_t a, uint64_t b);
uint64_t type_meet(uint64_t a, uint64_t b);
uint64_t type_join(uint64_t a, uint64_t b);

// Bulk operations for efficient array processing
CTypeSet* typeset_from_array(const uint64_t* ids, size_t count);
size_t typeset_to_array(const CTypeSet* set, uint64_t* ids, size_t capacity);
void typeset_union_inplace(CTypeSet* set, const CTypeSet* other);
void typeset_intersect_inplace(CTypeSet* set, const CTypeSet* other);

// Multi-way SIMD operations
CTypeSet* typeset_union_many(const CTypeSet** sets, size_t count);
CTypeSet* typeset_intersection_many(const CTypeSet** sets, size_t count);

} // extern "C"

