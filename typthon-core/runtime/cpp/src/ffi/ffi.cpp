#include "typthon/ffi/ffi.hpp"
#include <new>

using namespace typthon;

extern "C" {

CTypeSet* typeset_new() {
    auto* set = new TypeSet();
    return reinterpret_cast<CTypeSet*>(set);
}

void typeset_free(CTypeSet* set) {
    delete reinterpret_cast<TypeSet*>(set);
}

void typeset_insert(CTypeSet* set, uint64_t id) {
    reinterpret_cast<TypeSet*>(set)->insert(id);
}

bool typeset_contains(const CTypeSet* set, uint64_t id) {
    return reinterpret_cast<const TypeSet*>(set)->contains(id);
}

CTypeSet* typeset_union(const CTypeSet* a, const CTypeSet* b) {
    auto* result = new TypeSet(
        *reinterpret_cast<const TypeSet*>(a) | *reinterpret_cast<const TypeSet*>(b)
    );
    return reinterpret_cast<CTypeSet*>(result);
}

CTypeSet* typeset_intersection(const CTypeSet* a, const CTypeSet* b) {
    auto* result = new TypeSet(
        *reinterpret_cast<const TypeSet*>(a) & *reinterpret_cast<const TypeSet*>(b)
    );
    return reinterpret_cast<CTypeSet*>(result);
}

bool typeset_is_subset(const CTypeSet* a, const CTypeSet* b) {
    return reinterpret_cast<const TypeSet*>(a)->is_subset_of(
        *reinterpret_cast<const TypeSet*>(b)
    );
}

size_t typeset_cardinality(const CTypeSet* set) {
    return reinterpret_cast<const TypeSet*>(set)->cardinality();
}

bool type_is_subtype(uint64_t a, uint64_t b) {
    return TypeLattice::is_subtype(a, b);
}

uint64_t type_meet(uint64_t a, uint64_t b) {
    return TypeLattice::meet(a, b);
}

uint64_t type_join(uint64_t a, uint64_t b) {
    return TypeLattice::join(a, b);
}

CTypeSet* typeset_from_array(const uint64_t* ids, size_t count) {
    auto* set = new TypeSet();
    set->insert_many(ids, count);
    return reinterpret_cast<CTypeSet*>(set);
}

size_t typeset_to_array(const CTypeSet* set, uint64_t* ids, size_t capacity) {
    auto* ts = reinterpret_cast<const TypeSet*>(set);
    size_t written = 0;

    // Iterate through set bits
    for (size_t i = 0; i < 4096 && written < capacity; ++i) {
        if (ts->contains(i)) {
            ids[written++] = i;
        }
    }

    return written;
}

void typeset_union_inplace(CTypeSet* set, const CTypeSet* other) {
    reinterpret_cast<TypeSet*>(set)->union_inplace(
        *reinterpret_cast<const TypeSet*>(other)
    );
}

void typeset_intersect_inplace(CTypeSet* set, const CTypeSet* other) {
    reinterpret_cast<TypeSet*>(set)->intersect_inplace(
        *reinterpret_cast<const TypeSet*>(other)
    );
}

CTypeSet* typeset_union_many(const CTypeSet** sets, size_t count) {
    if (count == 0) return typeset_new();

    auto* result = new TypeSet(*reinterpret_cast<const TypeSet*>(sets[0]));

    for (size_t i = 1; i < count; ++i) {
        result->union_inplace(*reinterpret_cast<const TypeSet*>(sets[i]));
    }

    return reinterpret_cast<CTypeSet*>(result);
}

CTypeSet* typeset_intersection_many(const CTypeSet** sets, size_t count) {
    if (count == 0) return typeset_new();

    auto* result = new TypeSet(*reinterpret_cast<const TypeSet*>(sets[0]));

    for (size_t i = 1; i < count; ++i) {
        result->intersect_inplace(*reinterpret_cast<const TypeSet*>(sets[i]));
    }

    return reinterpret_cast<CTypeSet*>(result);
}

} // extern "C"

