//! C FFI - Public C API for runtime
//!
//! Design: Stable C API for linking with generated code.

use crate::allocator::ObjectHeader;

/// Create new object
#[no_mangle]
pub extern "C" fn typthon_object_new(size: usize) -> *mut u8 {
    // TODO: Allocate object with header
    core::ptr::null_mut()
}

/// Increment refcount
#[no_mangle]
pub extern "C" fn typthon_incref(obj: *mut u8) {
    if obj.is_null() {
        return;
    }

    unsafe {
        let header = obj.sub(16) as *mut ObjectHeader;
        (*header).refcount += 1;
    }
}

/// Decrement refcount
#[no_mangle]
pub extern "C" fn typthon_decref(obj: *mut u8) {
    if obj.is_null() {
        return;
    }

    unsafe {
        let header = obj.sub(16) as *mut ObjectHeader;
        (*header).refcount -= 1;

        if (*header).refcount == 0 {
            // Free object
            // TODO: Call drop + free
        }
    }
}

