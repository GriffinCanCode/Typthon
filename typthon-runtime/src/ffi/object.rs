//! Object lifecycle - allocation and destruction
//!
//! C API for object creation and cleanup, integrated with allocator.

use crate::allocator::ObjectHeader;
use core::ptr;

/// Allocate new object with header (returns pointer to data, not header)
///
/// # Safety
/// - Returns null on allocation failure
/// - Returned pointer is 8-byte aligned
/// - Caller must call `typthon_object_destroy` or manage refcount to zero
#[no_mangle]
pub extern "C" fn typthon_object_new(size: usize) -> *mut u8 {
    if size == 0 || size > isize::MAX as usize {
        return ptr::null_mut();
    }

    // TODO: Integrate with allocator::Allocator
    // For now, use system allocator as placeholder
    unsafe {
        let layout = std::alloc::Layout::from_size_align_unchecked(
            size + core::mem::size_of::<ObjectHeader>(),
            8,
        );

        let ptr = std::alloc::alloc(layout);
        if ptr.is_null() {
            return ptr::null_mut();
        }

        // Initialize header with default type info
        let _header = ptr as *mut ObjectHeader;
        // TODO: Use proper type info from allocator
        // _header.write(ObjectHeader::new(type_info));

        // Return pointer to data (after header)
        ptr.add(core::mem::size_of::<ObjectHeader>())
    }
}

/// Destroy object and free memory (does not check refcount)
///
/// # Safety
/// - Object must have been allocated with `typthon_object_new`
/// - Refcount must be zero (caller's responsibility)
/// - Pointer becomes invalid after this call
#[no_mangle]
pub extern "C" fn typthon_object_destroy(obj: *mut u8) {
    if obj.is_null() {
        return;
    }

    unsafe {
        let header_ptr = ObjectHeader::from_object(obj);
        let header = &*header_ptr;

        // Call type-specific destructor if present
        if let Some(drop_fn) = header.type_info.as_ref().drop {
            drop_fn(obj);
        }

        // Free memory
        let layout = std::alloc::Layout::from_size_align_unchecked(
            header.type_info.as_ref().size + core::mem::size_of::<ObjectHeader>(),
            8,
        );
        std::alloc::dealloc(header_ptr as *mut u8, layout);
    }
}

/// Get object size (excluding header)
///
/// # Safety
/// - Returns 0 for null pointers
/// - Object must be valid
#[no_mangle]
pub extern "C" fn typthon_object_size(obj: *const u8) -> usize {
    if obj.is_null() {
        return 0;
    }

    unsafe {
        let header = &*ObjectHeader::from_object(obj as *mut u8);
        header.type_info.as_ref().size
    }
}

