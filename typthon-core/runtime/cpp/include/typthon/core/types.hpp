#pragma once
#include <cstdint>
#include <cstring>

// Architecture-specific SIMD headers
#if defined(__x86_64__) || defined(_M_X64) || defined(__i386__) || defined(_M_IX86)
    #include <immintrin.h>
    #define TYPTHON_X86_SIMD
#elif defined(__aarch64__) || defined(_M_ARM64) || defined(__arm__) || defined(_M_ARM)
    #include <arm_neon.h>
    #define TYPTHON_ARM_NEON
#endif

namespace typthon {

// Type ID is a 64-bit unique identifier
using TypeId = uint64_t;

// Bit vector for efficient set operations on type sets
// Uses cache-aligned memory and SIMD when available
class TypeSet {
private:
    static constexpr size_t BITS = 4096;  // Support up to 4096 unique types
    static constexpr size_t WORDS = BITS / 64;
    alignas(64) uint64_t bits_[WORDS];

public:
    TypeSet() { std::memset(bits_, 0, sizeof(bits_)); }

    inline void insert(TypeId id) {
        bits_[id / 64] |= (1ULL << (id % 64));
    }

    inline bool contains(TypeId id) const {
        return bits_[id / 64] & (1ULL << (id % 64));
    }

    inline void remove(TypeId id) {
        bits_[id / 64] &= ~(1ULL << (id % 64));
    }

    // Union: A ∪ B
    inline TypeSet operator|(const TypeSet& other) const {
        TypeSet result;
        #if defined(TYPTHON_X86_SIMD) && defined(__AVX2__)
        // x86 AVX2 path
        for (size_t i = 0; i < WORDS; i += 4) {
            __m256i a = _mm256_load_si256((__m256i*)&bits_[i]);
            __m256i b = _mm256_load_si256((__m256i*)&other.bits_[i]);
            _mm256_store_si256((__m256i*)&result.bits_[i], _mm256_or_si256(a, b));
        }
        #elif defined(TYPTHON_ARM_NEON)
        // ARM NEON path
        for (size_t i = 0; i < WORDS; i += 2) {
            uint64x2_t a = vld1q_u64(&bits_[i]);
            uint64x2_t b = vld1q_u64(&other.bits_[i]);
            vst1q_u64(&result.bits_[i], vorrq_u64(a, b));
        }
        #else
        // Scalar fallback
        for (size_t i = 0; i < WORDS; ++i) {
            result.bits_[i] = bits_[i] | other.bits_[i];
        }
        #endif
        return result;
    }

    // Intersection: A ∩ B
    inline TypeSet operator&(const TypeSet& other) const {
        TypeSet result;
        #if defined(TYPTHON_X86_SIMD) && defined(__AVX2__)
        // x86 AVX2 path
        for (size_t i = 0; i < WORDS; i += 4) {
            __m256i a = _mm256_load_si256((__m256i*)&bits_[i]);
            __m256i b = _mm256_load_si256((__m256i*)&other.bits_[i]);
            _mm256_store_si256((__m256i*)&result.bits_[i], _mm256_and_si256(a, b));
        }
        #elif defined(TYPTHON_ARM_NEON)
        // ARM NEON path
        for (size_t i = 0; i < WORDS; i += 2) {
            uint64x2_t a = vld1q_u64(&bits_[i]);
            uint64x2_t b = vld1q_u64(&other.bits_[i]);
            vst1q_u64(&result.bits_[i], vandq_u64(a, b));
        }
        #else
        // Scalar fallback
        for (size_t i = 0; i < WORDS; ++i) {
            result.bits_[i] = bits_[i] & other.bits_[i];
        }
        #endif
        return result;
    }

    // Difference: A \ B
    inline TypeSet operator-(const TypeSet& other) const {
        TypeSet result;
        for (size_t i = 0; i < WORDS; ++i) {
            result.bits_[i] = bits_[i] & ~other.bits_[i];
        }
        return result;
    }

    // Subset: A ⊆ B
    inline bool is_subset_of(const TypeSet& other) const {
        for (size_t i = 0; i < WORDS; ++i) {
            if ((bits_[i] & ~other.bits_[i]) != 0) return false;
        }
        return true;
    }

    // Cardinality: |A|
    inline size_t cardinality() const {
        size_t count = 0;
        for (size_t i = 0; i < WORDS; ++i) {
            count += __builtin_popcountll(bits_[i]);
        }
        return count;
    }

    inline bool empty() const {
        for (size_t i = 0; i < WORDS; ++i) {
            if (bits_[i] != 0) return false;
        }
        return true;
    }

    // Bulk insert operation for efficient array processing
    inline void insert_many(const TypeId* ids, size_t count) {
        for (size_t i = 0; i < count; ++i) {
            insert(ids[i]);
        }
    }

    // Get raw bits pointer for zero-copy operations
    inline const uint64_t* data() const { return bits_; }
    inline uint64_t* data_mut() { return bits_; }

    // In-place union operation (modifies this set)
    inline void union_inplace(const TypeSet& other) {
        #if defined(TYPTHON_X86_SIMD) && defined(__AVX2__)
        for (size_t i = 0; i < WORDS; i += 4) {
            __m256i a = _mm256_load_si256((__m256i*)&bits_[i]);
            __m256i b = _mm256_load_si256((__m256i*)&other.bits_[i]);
            _mm256_store_si256((__m256i*)&bits_[i], _mm256_or_si256(a, b));
        }
        #elif defined(TYPTHON_ARM_NEON)
        for (size_t i = 0; i < WORDS; i += 2) {
            uint64x2_t a = vld1q_u64(&bits_[i]);
            uint64x2_t b = vld1q_u64(&other.bits_[i]);
            vst1q_u64(&bits_[i], vorrq_u64(a, b));
        }
        #else
        for (size_t i = 0; i < WORDS; ++i) {
            bits_[i] |= other.bits_[i];
        }
        #endif
    }

    // In-place intersection operation
    inline void intersect_inplace(const TypeSet& other) {
        #if defined(TYPTHON_X86_SIMD) && defined(__AVX2__)
        for (size_t i = 0; i < WORDS; i += 4) {
            __m256i a = _mm256_load_si256((__m256i*)&bits_[i]);
            __m256i b = _mm256_load_si256((__m256i*)&other.bits_[i]);
            _mm256_store_si256((__m256i*)&bits_[i], _mm256_and_si256(a, b));
        }
        #elif defined(TYPTHON_ARM_NEON)
        for (size_t i = 0; i < WORDS; i += 2) {
            uint64x2_t a = vld1q_u64(&bits_[i]);
            uint64x2_t b = vld1q_u64(&other.bits_[i]);
            vst1q_u64(&bits_[i], vandq_u64(a, b));
        }
        #else
        for (size_t i = 0; i < WORDS; ++i) {
            bits_[i] &= other.bits_[i];
        }
        #endif
    }
};

// Type lattice operations for subtyping
class TypeLattice {
public:
    // Meet: greatest lower bound (most specific common type)
    static TypeId meet(TypeId a, TypeId b);

    // Join: least upper bound (most general common type)
    static TypeId join(TypeId a, TypeId b);

    // Subtype relation: a <: b
    static bool is_subtype(TypeId a, TypeId b);
};

} // namespace typthon

