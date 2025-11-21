#pragma once

/**
 * @file typthon.hpp
 * @brief Main header for the Typthon C++ type system library
 *
 * This header provides a single point of inclusion for all Typthon C++ components.
 * Include this file to access the complete type system API.
 */

// Core type system components
#include "core/types.hpp"

// Foreign Function Interface (FFI) for Rust interop
#include "ffi/ffi.hpp"

namespace typthon {
    // Main namespace containing all Typthon C++ components
    //
    // Core types:
    //   - TypeId: 64-bit unique type identifier
    //   - TypeSet: SIMD-optimized bit-vector set for type operations
    //   - TypeLattice: Type hierarchy operations (meet, join, subtyping)
    //
    // FFI bindings (extern "C"):
    //   - CTypeSet and related functions for Rust FFI
    //   - Type lattice operations with C ABI
}

