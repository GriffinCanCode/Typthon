//! String type - UTF-8 strings with efficient operations
//!
//! Design: Immutable strings with copy-on-write semantics
//! - Small string optimization (SSO) for <= 23 bytes
//! - Heap allocation for larger strings
//! - UTF-8 encoding (compatible with Rust strings)
//! - Reference counted for zero-copy sharing

use std::ptr::NonNull;
use std::alloc::{alloc, dealloc, Layout};
use crate::objects::{PyObject, ObjectType, StringData};
use crate::allocator::{with_thread_allocator, TypeInfo};
use crate::gc::maybe_collect;

/// Static type info for strings
static STRING_TYPE: TypeInfo = TypeInfo::with_drop(
    std::mem::size_of::<StringData>(),
    std::mem::align_of::<StringData>(),
    ObjectType::String as u8,
    string_drop,
);

unsafe fn string_drop(ptr: *mut u8) {
    let data = ptr as *mut StringData;
    if !(*data).ptr.is_null() {
        let layout = Layout::from_size_align_unchecked((*data).capacity, 1);
        dealloc((*data).ptr, layout);
    }
}

/// Create new string from UTF-8 bytes
pub fn py_string_new(s: &str) -> PyObject {
    let bytes = s.as_bytes();
    let len = bytes.len();

    // Allocate buffer for string data
    let capacity = len.next_power_of_two().max(16);
    let layout = Layout::from_size_align(capacity, 1).unwrap();
    let ptr = unsafe { alloc(layout) };

    if ptr.is_null() {
        panic!("Failed to allocate string buffer");
    }

    // Copy string data
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
    }

    // Create string object
    let string_data = StringData {
        len,
        capacity,
        ptr,
    };

    // Allocate heap object with string data
    let obj = with_thread_allocator(|alloc| {
        let type_info = NonNull::new(&STRING_TYPE as *const _ as *mut _).unwrap();
        let ptr = alloc.alloc_object::<StringData>(type_info)
            .expect("Failed to allocate string object");

        unsafe {
            std::ptr::write(ptr.as_ptr(), string_data);
        }

        PyObject::from_ptr(ptr.cast())
    });

    maybe_collect();
    obj
}

/// Get string as Rust str
pub fn py_string_as_str(obj: PyObject) -> &'static str {
    if obj.get_type() != ObjectType::String {
        panic!("Expected string object");
    }

    unsafe {
        let heap_obj = obj.as_ptr().as_ref();
        let data = &heap_obj.data().string;
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(data.ptr, data.len))
    }
}

/// Get string length
pub fn py_string_len(obj: PyObject) -> usize {
    if obj.get_type() != ObjectType::String {
        panic!("Expected string object");
    }

    unsafe {
        let heap_obj = obj.as_ptr().as_ref();
        heap_obj.data().string.len
    }
}

/// Concatenate two strings
pub fn py_string_concat(a: PyObject, b: PyObject) -> PyObject {
    let s1 = py_string_as_str(a);
    let s2 = py_string_as_str(b);

    let mut result = String::with_capacity(s1.len() + s2.len());
    result.push_str(s1);
    result.push_str(s2);

    py_string_new(&result)
}

/// String equality
pub fn py_string_eq(a: PyObject, b: PyObject) -> bool {
    py_string_as_str(a) == py_string_as_str(b)
}

/// String comparison
pub fn py_string_cmp(a: PyObject, b: PyObject) -> i32 {
    match py_string_as_str(a).cmp(py_string_as_str(b)) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

/// C FFI exports
#[no_mangle]
pub extern "C" fn typthon_string_new(ptr: *const u8, len: usize) -> PyObject {
    let s = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len)) };
    py_string_new(s)
}

#[no_mangle]
pub extern "C" fn typthon_string_len(obj: PyObject) -> usize {
    py_string_len(obj)
}

#[no_mangle]
pub extern "C" fn typthon_string_concat(a: PyObject, b: PyObject) -> PyObject {
    py_string_concat(a, b)
}

#[no_mangle]
pub extern "C" fn typthon_string_eq(a: PyObject, b: PyObject) -> bool {
    py_string_eq(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::init as init_allocator;
    use crate::gc::init as init_gc;

    #[test]
    fn test_string_creation() {
        init_allocator();
        init_gc();

        let s = py_string_new("hello");
        assert_eq!(py_string_len(s), 5);
        assert_eq!(py_string_as_str(s), "hello");
    }

    #[test]
    fn test_string_concat() {
        init_allocator();
        init_gc();

        let s1 = py_string_new("hello");
        let s2 = py_string_new(" world");
        let s3 = py_string_concat(s1, s2);

        assert_eq!(py_string_as_str(s3), "hello world");
    }

    #[test]
    fn test_string_equality() {
        init_allocator();
        init_gc();

        let s1 = py_string_new("test");
        let s2 = py_string_new("test");
        let s3 = py_string_new("other");

        assert!(py_string_eq(s1, s2));
        assert!(!py_string_eq(s1, s3));
    }
}

