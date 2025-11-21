//! Reference counting - C API for refcount operations
//!
//! Hot path operations with minimal overhead, inlined by compiler.
//! Thread-safe atomic operations.

use crate::allocator::ObjectHeader;
use std::sync::atomic::Ordering;

/// Increment reference count (hot path, always inlined)
///
/// # Safety
/// - Null-safe (no-op for null pointers)
/// - Object must be valid heap object
/// - Overflow checked in debug builds
#[no_mangle]
pub extern "C" fn typthon_incref(obj: *mut u8) {
    if obj.is_null() {
        return;
    }

    unsafe {
        let header = &*ObjectHeader::from_object(obj);
        let old = header.refcount.fetch_add(1, Ordering::Relaxed);

        debug_assert!(old < u32::MAX, "refcount overflow");
    }
}

/// Decrement reference count, destroy if reaches zero (hot path)
///
/// # Safety
/// - Null-safe (no-op for null pointers)
/// - Object must be valid heap object
/// - Underflow checked in debug builds
/// - Calls `typthon_object_destroy` when refcount hits zero
#[no_mangle]
pub extern "C" fn typthon_decref(obj: *mut u8) {
    if obj.is_null() {
        return;
    }

    unsafe {
        let header = &*ObjectHeader::from_object(obj);
        let old = header.refcount.fetch_sub(1, Ordering::Release);

        debug_assert!(old > 0, "refcount underflow");

        if old == 1 {
            // Synchronize with all previous decrements
            std::sync::atomic::fence(Ordering::Acquire);
            // Cold path: destroy object
            destroy_object(obj);
        }
    }
}

/// Get current reference count (for debugging/testing)
///
/// # Safety
/// - Returns 0 for null pointers
/// - Object must be valid heap object
#[no_mangle]
pub extern "C" fn typthon_refcount(obj: *const u8) -> u32 {
    if obj.is_null() {
        return 0;
    }

    unsafe {
        let header = &*ObjectHeader::from_object(obj as *mut u8);
        header.refcount.load(Ordering::Relaxed)
    }
}

/// Destroy object (cold path, separated for better code generation)
#[cold]
#[inline(never)]
unsafe fn destroy_object(obj: *mut u8) {
    let header = &*ObjectHeader::from_object(obj);

    // Call type-specific destructor if present
    if let Some(drop_fn) = header.type_info.as_ref().drop {
        drop_fn(obj);
    }

    // Note: In production, the allocator uses arena-based allocation
    // where memory is freed in bulk. For FFI objects that may be allocated
    // via system allocator (during bootstrapping or testing), we use dealloc.
    // Arena-allocated objects will be reclaimed during arena sweeps.
    let layout = std::alloc::Layout::from_size_align_unchecked(
        header.type_info.as_ref().size + core::mem::size_of::<ObjectHeader>(),
        8,
    );
    // Only dealloc if this was system-allocated (not arena-allocated)
    // For now we always dealloc; in production we'd check allocation source
    std::alloc::dealloc(ObjectHeader::from_object(obj) as *mut u8, layout);
}

/// Increment refcount and return same pointer (for chaining)
///
/// # Safety
/// - Returns null for null input
/// - Otherwise same as `typthon_incref`
#[no_mangle]
pub extern "C" fn typthon_incref_ret(obj: *mut u8) -> *mut u8 {
    typthon_incref(obj);
    obj
}

