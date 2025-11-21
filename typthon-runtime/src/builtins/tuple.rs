//! Tuple type - immutable fixed-size arrays
//!
//! Design: Immutable heterogeneous collections with:
//! - Inline elements (flexible array member)
//! - Fast indexing O(1)
//! - Hashable (for dict keys)
//! - Reference counted elements

use std::ptr::NonNull;
use crate::objects::{PyObject, ObjectType, TupleData};
use crate::allocator::{with_thread_allocator, TypeInfo};
use crate::gc::maybe_collect;

/// Helper: increment refcount for object if it's a heap object
#[inline]
fn incref_object(obj: PyObject) {
    if obj.is_ptr() {
        crate::ffi::typthon_incref(obj.as_ptr().as_ptr() as *mut u8);
    }
}

/// Helper: decrement refcount for object if it's a heap object
#[inline]
fn decref_object(obj: PyObject) {
    if obj.is_ptr() {
        crate::ffi::typthon_decref(obj.as_ptr().as_ptr() as *mut u8);
    }
}

/// Static type info for tuples
static TUPLE_TYPE: TypeInfo = TypeInfo::with_drop(
    std::mem::size_of::<TupleData>(),
    std::mem::align_of::<TupleData>(),
    ObjectType::Tuple as u8,
    tuple_drop,
);

unsafe fn tuple_drop(ptr: *mut u8) {
    let data = ptr as *mut TupleData;
    let len = (*data).len;

    // Decrement refcount for all elements
    let elements_ptr = (*data).elements.as_ptr() as *mut PyObject;
    for i in 0..len {
        let obj = *elements_ptr.add(i);
        decref_object(obj);
    }
}

/// Create new tuple from slice of objects
pub fn py_tuple_new(items: &[PyObject]) -> PyObject {
    let len = items.len();

    // Allocate tuple with inline elements
    let total_size = std::mem::size_of::<TupleData>() + len * std::mem::size_of::<PyObject>();

    let obj = with_thread_allocator(|alloc| {
        let type_info = NonNull::new(&TUPLE_TYPE as *const _ as *mut _).unwrap();
        let ptr = alloc.alloc(total_size, std::mem::align_of::<TupleData>())
            .expect("Failed to allocate tuple");

        unsafe {
            // Initialize header
            let header_ptr = ptr.as_ptr() as *mut crate::allocator::ObjectHeader;
            header_ptr.write(crate::allocator::ObjectHeader::new(type_info));

            // Initialize tuple data
            let data_ptr = header_ptr.add(1) as *mut TupleData;
            (*data_ptr).len = len;

            // Copy elements and increment refcounts
            let elements_ptr = (*data_ptr).elements.as_ptr() as *mut PyObject;
            for i in 0..len {
                let item = items[i];
                incref_object(item);
                *elements_ptr.add(i) = item;
            }

            PyObject::from_ptr(NonNull::new_unchecked(data_ptr).cast())
        }
    });

    maybe_collect();
    obj
}

/// Get tuple length
pub fn py_tuple_len(obj: PyObject) -> usize {
    if obj.get_type() != ObjectType::Tuple {
        panic!("Expected tuple object");
    }

    unsafe {
        let heap_obj = obj.as_ptr().as_ref();
        heap_obj.data().tuple.len
    }
}

/// Get element at index
pub fn py_tuple_get(obj: PyObject, index: isize) -> PyObject {
    if obj.get_type() != ObjectType::Tuple {
        panic!("Expected tuple object");
    }

    unsafe {
        let heap_obj = obj.as_ptr().as_ref();
        let data = &heap_obj.data().tuple;

        let idx = if index < 0 {
            (data.len as isize + index) as usize
        } else {
            index as usize
        };

        if idx >= data.len {
            panic!("Tuple index out of range");
        }

        let elements_ptr = data.elements.as_ptr() as *const PyObject;
        *elements_ptr.add(idx)
    }
}

/// Tuple equality
pub fn py_tuple_eq(a: PyObject, b: PyObject) -> bool {
    if a.get_type() != ObjectType::Tuple || b.get_type() != ObjectType::Tuple {
        return false;
    }

    let len_a = py_tuple_len(a);
    let len_b = py_tuple_len(b);

    if len_a != len_b {
        return false;
    }

    for i in 0..len_a {
        let elem_a = py_tuple_get(a, i as isize);
        let elem_b = py_tuple_get(b, i as isize);

        // Use operations module for equality
        if !crate::builtins::operations::py_eq(elem_a, elem_b) {
            return false;
        }
    }

    true
}

/// C FFI exports
#[no_mangle]
pub extern "C" fn typthon_tuple_new(items: *const PyObject, len: usize) -> PyObject {
    let slice = unsafe { std::slice::from_raw_parts(items, len) };
    py_tuple_new(slice)
}

#[no_mangle]
pub extern "C" fn typthon_tuple_len(obj: PyObject) -> usize {
    py_tuple_len(obj)
}

#[no_mangle]
pub extern "C" fn typthon_tuple_get(obj: PyObject, index: isize) -> PyObject {
    py_tuple_get(obj, index)
}

#[no_mangle]
pub extern "C" fn typthon_tuple_eq(a: PyObject, b: PyObject) -> bool {
    py_tuple_eq(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::init as init_allocator;
    use crate::gc::init as init_gc;

    #[test]
    fn test_tuple_creation() {
        init_allocator();
        init_gc();

        let items = vec![PyObject::from_int(1), PyObject::from_int(2), PyObject::from_int(3)];
        let tuple = py_tuple_new(&items);

        assert_eq!(py_tuple_len(tuple), 3);
        assert_eq!(py_tuple_get(tuple, 0).as_int(), 1);
        assert_eq!(py_tuple_get(tuple, 1).as_int(), 2);
        assert_eq!(py_tuple_get(tuple, 2).as_int(), 3);
    }

    #[test]
    fn test_tuple_negative_indexing() {
        init_allocator();
        init_gc();

        let items = vec![PyObject::from_int(10), PyObject::from_int(20), PyObject::from_int(30)];
        let tuple = py_tuple_new(&items);

        assert_eq!(py_tuple_get(tuple, -1).as_int(), 30);
        assert_eq!(py_tuple_get(tuple, -2).as_int(), 20);
        assert_eq!(py_tuple_get(tuple, -3).as_int(), 10);
    }
}

