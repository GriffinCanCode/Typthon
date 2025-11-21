//! List type - dynamic arrays with amortized O(1) append
//!
//! Design: Contiguous array with geometric growth (factor of 2)
//! - Cache-friendly sequential access
//! - Fast indexing O(1)
//! - Efficient appends (amortized O(1))
//! - Reference counted elements

use std::ptr::NonNull;
use std::alloc::{alloc, dealloc, realloc, Layout};
use crate::objects::{PyObject, ObjectType, ListData};
use crate::allocator::{with_thread_allocator, TypeInfo};
use crate::gc::maybe_collect;

/// Helper: increment refcount for object if it's a heap object
#[inline]
fn incref_object(obj: PyObject) {
    if obj.is_ptr() {
        crate::ffi::refcount::typthon_incref(obj.as_ptr().as_ptr() as *mut u8);
    }
}

/// Helper: decrement refcount for object if it's a heap object
#[inline]
fn decref_object(obj: PyObject) {
    if obj.is_ptr() {
        crate::ffi::refcount::typthon_decref(obj.as_ptr().as_ptr() as *mut u8);
    }
}

/// Static type info for lists
static LIST_TYPE: TypeInfo = TypeInfo::with_drop(
    std::mem::size_of::<ListData>(),
    std::mem::align_of::<ListData>(),
    ObjectType::List as u8,
    list_drop,
);

unsafe fn list_drop(ptr: *mut u8) {
    let data = ptr as *mut ListData;
    if !(*data).ptr.is_null() {
        // TODO: Decrement refcount for all elements
        let layout = Layout::from_size_align_unchecked(
            (*data).capacity * std::mem::size_of::<PyObject>(),
            std::mem::align_of::<PyObject>()
        );
        dealloc((*data).ptr as *mut u8, layout);
    }
}

/// Create new empty list
pub fn py_list_new() -> PyObject {
    py_list_with_capacity(8)
}

/// Create list with specified capacity
pub fn py_list_with_capacity(capacity: usize) -> PyObject {
    let layout = Layout::from_size_align(
        capacity * std::mem::size_of::<PyObject>(),
        std::mem::align_of::<PyObject>()
    ).unwrap();

    let ptr = unsafe { alloc(layout) as *mut PyObject };
    if ptr.is_null() {
        panic!("Failed to allocate list buffer");
    }

    let list_data = ListData {
        len: 0,
        capacity,
        ptr,
    };

    let obj = with_thread_allocator(|alloc| {
        let type_info = NonNull::new(&LIST_TYPE as *const _ as *mut _).unwrap();
        let ptr = alloc.alloc_object::<ListData>(type_info)
            .expect("Failed to allocate list object");

        unsafe {
            std::ptr::write(ptr.as_ptr(), list_data);
        }

        PyObject::from_ptr(ptr.cast())
    });

    maybe_collect();
    obj
}

/// Get list length
pub fn py_list_len(obj: PyObject) -> usize {
    if obj.get_type() != ObjectType::List {
        panic!("Expected list object");
    }

    unsafe {
        let heap_obj = obj.as_ptr().as_ref();
        heap_obj.data().list.len
    }
}

/// Get element at index
pub fn py_list_get(obj: PyObject, index: isize) -> PyObject {
    if obj.get_type() != ObjectType::List {
        panic!("Expected list object");
    }

    unsafe {
        let heap_obj = obj.as_ptr().as_ref();
        let data = &heap_obj.data().list;

        let idx = if index < 0 {
            (data.len as isize + index) as usize
        } else {
            index as usize
        };

        if idx >= data.len {
            panic!("List index out of range");
        }

        *data.ptr.add(idx)
    }
}

/// Set element at index
pub fn py_list_set(obj: PyObject, index: isize, value: PyObject) {
    if obj.get_type() != ObjectType::List {
        panic!("Expected list object");
    }

    unsafe {
        let heap_obj = obj.as_ptr().as_mut();
        let data = &mut heap_obj.data_mut().list;

        let idx = if index < 0 {
            (data.len as isize + index) as usize
        } else {
            index as usize
        };

        if idx >= data.len {
            panic!("List index out of range");
        }

        // TODO: Decrement refcount of old value, increment refcount of new value
        *data.ptr.add(idx) = value;
    }
}

/// Append element to list
pub fn py_list_append(obj: PyObject, value: PyObject) {
    if obj.get_type() != ObjectType::List {
        panic!("Expected list object");
    }

    unsafe {
        let heap_obj = obj.as_ptr().as_mut();
        let data = &mut heap_obj.data_mut().list;

        // Grow if needed
        if data.len >= data.capacity {
            let new_capacity = data.capacity * 2;
            let old_layout = Layout::from_size_align_unchecked(
                data.capacity * std::mem::size_of::<PyObject>(),
                std::mem::align_of::<PyObject>()
            );
            let new_layout = Layout::from_size_align_unchecked(
                new_capacity * std::mem::size_of::<PyObject>(),
                std::mem::align_of::<PyObject>()
            );

            let new_ptr = realloc(data.ptr as *mut u8, old_layout, new_layout.size());
            if new_ptr.is_null() {
                panic!("Failed to grow list");
            }

            data.ptr = new_ptr as *mut PyObject;
            data.capacity = new_capacity;
        }

        // TODO: Increment refcount of value
        *data.ptr.add(data.len) = value;
        data.len += 1;
    }
}

/// Create list from slice
pub fn py_list_from_slice(items: &[PyObject]) -> PyObject {
    let list = py_list_with_capacity(items.len().next_power_of_two());

    for &item in items {
        py_list_append(list, item);
    }

    list
}

/// C FFI exports
#[no_mangle]
pub extern "C" fn typthon_list_new() -> PyObject {
    py_list_new()
}

#[no_mangle]
pub extern "C" fn typthon_list_len(obj: PyObject) -> usize {
    py_list_len(obj)
}

#[no_mangle]
pub extern "C" fn typthon_list_get(obj: PyObject, index: isize) -> PyObject {
    py_list_get(obj, index)
}

#[no_mangle]
pub extern "C" fn typthon_list_set(obj: PyObject, index: isize, value: PyObject) {
    py_list_set(obj, index, value)
}

#[no_mangle]
pub extern "C" fn typthon_list_append(obj: PyObject, value: PyObject) {
    py_list_append(obj, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::init as init_allocator;
    use crate::gc::init as init_gc;

    #[test]
    fn test_list_creation() {
        init_allocator();
        init_gc();

        let list = py_list_new();
        assert_eq!(py_list_len(list), 0);
    }

    #[test]
    fn test_list_append() {
        init_allocator();
        init_gc();

        let list = py_list_new();
        py_list_append(list, PyObject::from_int(42));
        py_list_append(list, PyObject::from_int(100));

        assert_eq!(py_list_len(list), 2);
        assert_eq!(py_list_get(list, 0).as_int(), 42);
        assert_eq!(py_list_get(list, 1).as_int(), 100);
    }

    #[test]
    fn test_list_negative_indexing() {
        init_allocator();
        init_gc();

        let list = py_list_new();
        py_list_append(list, PyObject::from_int(1));
        py_list_append(list, PyObject::from_int(2));
        py_list_append(list, PyObject::from_int(3));

        assert_eq!(py_list_get(list, -1).as_int(), 3);
        assert_eq!(py_list_get(list, -2).as_int(), 2);
    }
}

