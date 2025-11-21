//! Reference counting - C API for refcount operations
//!
//! Hot path operations with minimal overhead, inlined by compiler.

use crate::allocator::ObjectHeader;

/// Increment reference count (hot path, always inlined)
///
/// # Safety
/// - Null-safe (no-op for null pointers)
/// - Object must be valid heap object
/// - Overflow checked in debug builds
#[no_mangle]
#[inline(always)]
pub extern "C" fn typthon_incref(obj: *mut u8) {
    if obj.is_null() {
        return;
    }

    unsafe {
        let header = &mut *ObjectHeader::from_object(obj);
        header.refcount += 1;

        debug_assert!(header.refcount != 0, "refcount overflow");
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
#[inline(always)]
pub extern "C" fn typthon_decref(obj: *mut u8) {
    if obj.is_null() {
        return;
    }

    unsafe {
        let header = &mut *ObjectHeader::from_object(obj);

        debug_assert!(header.refcount > 0, "refcount underflow");

        header.refcount -= 1;

        if header.refcount == 0 {
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
        header.refcount
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

    // TODO: Return memory to allocator
    // For now, use system allocator
    let layout = std::alloc::Layout::from_size_align_unchecked(
        header.type_info.as_ref().size + core::mem::size_of::<ObjectHeader>(),
        8,
    );
    std::alloc::dealloc(ObjectHeader::from_object(obj) as *mut u8, layout);
}

/// Increment refcount and return same pointer (for chaining)
///
/// # Safety
/// - Returns null for null input
/// - Otherwise same as `typthon_incref`
#[no_mangle]
#[inline(always)]
pub extern "C" fn typthon_incref_ret(obj: *mut u8) -> *mut u8 {
    typthon_incref(obj);
    obj
}

