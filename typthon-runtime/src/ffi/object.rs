//! Object lifecycle - C API for object creation and destruction
//!
//! Provides safe wrappers for object allocation and deallocation with
//! proper refcount initialization and cleanup.

use crate::objects::PyObject;
use crate::allocator::{TypeInfo, ObjectHeader};
use crate::logging::{trace, debug};
use core::ptr::NonNull;

/// Create new heap object with given type info
///
/// # Safety
/// - type_info must point to valid TypeInfo
/// - Returns null on allocation failure
#[no_mangle]
pub extern "C" fn typthon_object_new(type_info: *const TypeInfo, size: usize) -> *mut u8 {
    if type_info.is_null() {
        return std::ptr::null_mut();
    }

    trace!(event = "object_new", size_bytes = size);

    crate::allocator::with_thread_allocator(|alloc| {
        let type_info_nn = unsafe { NonNull::new_unchecked(type_info as *mut TypeInfo) };

        alloc.alloc(size + core::mem::size_of::<ObjectHeader>(), 8)
            .map(|ptr| {
                unsafe {
                    // Initialize header
                    let header_ptr = ptr.as_ptr() as *mut ObjectHeader;
                    header_ptr.write(ObjectHeader::new(type_info_nn));

                    // Return pointer to data (after header)
                    let data_ptr = header_ptr.add(1) as *mut u8;
                    debug!(address = ?data_ptr, "Object allocated");
                    data_ptr
                }
            })
            .unwrap_or(std::ptr::null_mut())
    })
}

/// Destroy object (called by refcount when it hits zero)
///
/// # Safety
/// - obj must be valid heap object
/// - Should only be called by refcount system
#[no_mangle]
pub extern "C" fn typthon_object_destroy(obj: *mut u8) {
    if obj.is_null() {
        return;
    }

    trace!(event = "object_destroy", address = ?obj);

    unsafe {
        let header = &*ObjectHeader::from_object(obj);

        // Call type-specific destructor if present
        if let Some(drop_fn) = header.type_info.as_ref().drop {
            debug!(address = ?obj, "Calling destructor");
            drop_fn(obj);
        }
    }
}

/// Get object type ID
#[no_mangle]
pub extern "C" fn typthon_object_type(obj: PyObject) -> u8 {
    obj.get_type() as u8
}

/// Check if object is truthy
#[no_mangle]
pub extern "C" fn typthon_object_is_truthy(obj: PyObject) -> bool {
    obj.is_truthy()
}

/// Get string representation of object
#[no_mangle]
pub extern "C" fn typthon_object_to_string(obj: PyObject) -> PyObject {
    let s = obj.to_string();
    crate::builtins::py_string_new(&s)
}

/// Hash object
#[no_mangle]
pub extern "C" fn typthon_object_hash(obj: PyObject) -> u64 {
    obj.hash()
}

/// Create PyObject from C int
#[no_mangle]
pub extern "C" fn typthon_int_from_i64(value: i64) -> PyObject {
    PyObject::from_int(value)
}

/// Extract int value from PyObject
#[no_mangle]
pub extern "C" fn typthon_int_to_i64(obj: PyObject) -> i64 {
    if obj.is_int() {
        obj.as_int()
    } else {
        panic!("Expected int object");
    }
}

/// Create PyObject from C bool
#[no_mangle]
pub extern "C" fn typthon_bool_from_bool(value: bool) -> PyObject {
    PyObject::from_bool(value)
}

/// Get None singleton
#[no_mangle]
pub extern "C" fn typthon_none() -> PyObject {
    PyObject::none()
}

/// Check if object is None
#[no_mangle]
pub extern "C" fn typthon_is_none(obj: PyObject) -> bool {
    obj.get_type() == crate::objects::ObjectType::None
}
