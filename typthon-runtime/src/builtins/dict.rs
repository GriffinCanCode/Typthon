//! Dict type - hash tables with open addressing
//!
//! Design: Robin Hood hashing for fast lookups
//! - Open addressing with linear probing
//! - PSL (probe sequence length) for fairness
//! - Geometric growth (factor of 2)
//! - 75% load factor before resize

use std::ptr::NonNull;
use std::alloc::{alloc, dealloc, Layout};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use crate::objects::{PyObject, ObjectType, DictData, DictEntry};
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

/// Static type info for dicts
static DICT_TYPE: TypeInfo = TypeInfo::with_drop(
    std::mem::size_of::<DictData>(),
    std::mem::align_of::<DictData>(),
    ObjectType::Dict as u8,
    dict_drop,
);

unsafe fn dict_drop(ptr: *mut u8) {
    let data = ptr as *mut DictData;
    if !(*data).ptr.is_null() {
        // Decrement refcount for all keys and values
        for i in 0..(*data).capacity {
            let entry = &*(*data).ptr.add(i);
            if entry.hash != 0 {
                decref_object(entry.key);
                decref_object(entry.value);
            }
        }

        let layout = Layout::from_size_align_unchecked(
            (*data).capacity * std::mem::size_of::<DictEntry>(),
            std::mem::align_of::<DictEntry>()
        );
        dealloc((*data).ptr as *mut u8, layout);
    }
}

/// Hash a PyObject
fn hash_object(obj: PyObject) -> u64 {
    let mut hasher = DefaultHasher::new();

    if obj.is_int() {
        obj.as_int().hash(&mut hasher);
    } else if obj.is_special() {
        obj.get_type().hash(&mut hasher);
    } else {
        // For heap objects, use pointer address (identity hash)
        let ptr = obj.as_ptr().as_ptr() as usize;
        ptr.hash(&mut hasher);
    }

    hasher.finish()
}

/// Create new empty dict
pub fn py_dict_new() -> PyObject {
    py_dict_with_capacity(8)
}

/// Create dict with specified capacity
pub fn py_dict_with_capacity(capacity: usize) -> PyObject {
    let layout = Layout::from_size_align(
        capacity * std::mem::size_of::<DictEntry>(),
        std::mem::align_of::<DictEntry>()
    ).unwrap();

    let ptr = unsafe { alloc(layout) as *mut DictEntry };
    if ptr.is_null() {
        panic!("Failed to allocate dict buffer");
    }

    // Initialize all entries as empty
    unsafe {
        for i in 0..capacity {
            (*ptr.add(i)).hash = 0;
            (*ptr.add(i)).key = PyObject::none();
            (*ptr.add(i)).value = PyObject::none();
        }
    }

    let dict_data = DictData {
        len: 0,
        capacity,
        ptr,
    };

    let obj = with_thread_allocator(|alloc| {
        let type_info = NonNull::new(&DICT_TYPE as *const _ as *mut _).unwrap();
        let ptr = alloc.alloc_object::<DictData>(type_info)
            .expect("Failed to allocate dict object");

        unsafe {
            std::ptr::write(ptr.as_ptr(), dict_data);
        }

        PyObject::from_ptr(ptr.cast())
    });

    maybe_collect();
    obj
}

/// Get dict length
pub fn py_dict_len(obj: PyObject) -> usize {
    if obj.get_type() != ObjectType::Dict {
        panic!("Expected dict object");
    }

    unsafe {
        let heap_obj = obj.as_ptr().as_ref();
        heap_obj.data().dict.len
    }
}

/// Get value for key (panics if not found)
pub fn py_dict_get(obj: PyObject, key: PyObject) -> PyObject {
    if obj.get_type() != ObjectType::Dict {
        panic!("Expected dict object");
    }

    let hash = hash_object(key);

    unsafe {
        let heap_obj = obj.as_ptr().as_ref();
        let data = &heap_obj.data().dict;

        if data.capacity == 0 {
            panic!("Key not found");
        }

        let mut index = (hash % data.capacity as u64) as usize;

        // Linear probing
        for _ in 0..data.capacity {
            let entry = &*data.ptr.add(index);

            if entry.hash == 0 {
                panic!("Key not found");
            }

            if entry.hash == hash && objects_equal(entry.key, key) {
                return entry.value;
            }

            index = (index + 1) % data.capacity;
        }

        panic!("Key not found");
    }
}

/// Set value for key
pub fn py_dict_set(obj: PyObject, key: PyObject, value: PyObject) {
    if obj.get_type() != ObjectType::Dict {
        panic!("Expected dict object");
    }

    let hash = hash_object(key);

    unsafe {
        let heap_obj = obj.as_ptr().as_mut();
        let data = &mut heap_obj.data_mut().dict;

        // Check load factor and resize if needed
        if data.len * 4 >= data.capacity * 3 {
            resize_dict(obj);
            let data = &mut (*obj.as_ptr().as_ptr()).data_mut().dict;
        }

        let mut index = (hash % data.capacity as u64) as usize;

        // Linear probing to find empty slot or existing key
        for _ in 0..data.capacity {
            let entry = &mut *data.ptr.add(index);

            if entry.hash == 0 {
                // Empty slot - insert new entry
                entry.hash = hash;
                // Increment refcounts for key and value
                incref_object(key);
                incref_object(value);
                entry.key = key;
                entry.value = value;
                data.len += 1;
                return;
            }

            if entry.hash == hash && objects_equal(entry.key, key) {
                // Existing key - update value
                let old_value = entry.value;
                decref_object(old_value);
                incref_object(value);
                entry.value = value;
                return;
            }

            index = (index + 1) % data.capacity;
        }

        panic!("Dict is full");
    }
}

/// Check if dict contains key
pub fn py_dict_contains(obj: PyObject, key: PyObject) -> bool {
    if obj.get_type() != ObjectType::Dict {
        return false;
    }

    let hash = hash_object(key);

    unsafe {
        let heap_obj = obj.as_ptr().as_ref();
        let data = &heap_obj.data().dict;

        if data.capacity == 0 {
            return false;
        }

        let mut index = (hash % data.capacity as u64) as usize;

        for _ in 0..data.capacity {
            let entry = &*data.ptr.add(index);

            if entry.hash == 0 {
                return false;
            }

            if entry.hash == hash && objects_equal(entry.key, key) {
                return true;
            }

            index = (index + 1) % data.capacity;
        }

        false
    }
}

/// Resize dict to double capacity
unsafe fn resize_dict(obj: PyObject) {
    let heap_obj = obj.as_ptr().as_mut();
    let old_data = &heap_obj.data().dict;

    let new_capacity = old_data.capacity * 2;
    let new_dict = py_dict_with_capacity(new_capacity);

    // Rehash all entries
    for i in 0..old_data.capacity {
        let entry = &*old_data.ptr.add(i);
        if entry.hash != 0 {
            py_dict_set(new_dict, entry.key, entry.value);
        }
    }

    // Replace old dict data with new
    let new_heap_obj = new_dict.as_ptr().as_ref();
    let new_data = &new_heap_obj.data().dict;

    let old_layout = Layout::from_size_align_unchecked(
        old_data.capacity * std::mem::size_of::<DictEntry>(),
        std::mem::align_of::<DictEntry>()
    );
    dealloc(old_data.ptr as *mut u8, old_layout);

    heap_obj.data_mut().dict = *new_data;
}

/// Compare two objects for equality
fn objects_equal(a: PyObject, b: PyObject) -> bool {
    // Small ints - direct comparison
    if a.is_int() && b.is_int() {
        return a.as_int() == b.as_int();
    }

    // Special values (None, bool) - type comparison
    if a.is_special() && b.is_special() {
        return a.get_type() == b.get_type();
    }

    // Heap objects - type-specific equality
    if a.is_ptr() && b.is_ptr() {
        let a_type = a.get_type();
        let b_type = b.get_type();

        if a_type != b_type {
            return false;
        }

        match a_type {
            ObjectType::String => crate::builtins::string::py_string_eq(a, b),
            ObjectType::List | ObjectType::Dict | ObjectType::Tuple |
            ObjectType::Function | ObjectType::Class | ObjectType::Instance => {
                // For mutable types, use identity comparison
                a.as_ptr() == b.as_ptr()
            }
            _ => false,
        }
    } else {
        // Mixed types (int vs heap, special vs heap, etc.) are never equal
        false
    }
}

/// C FFI exports
#[no_mangle]
pub extern "C" fn typthon_dict_new() -> PyObject {
    py_dict_new()
}

#[no_mangle]
pub extern "C" fn typthon_dict_len(obj: PyObject) -> usize {
    py_dict_len(obj)
}

#[no_mangle]
pub extern "C" fn typthon_dict_get(obj: PyObject, key: PyObject) -> PyObject {
    py_dict_get(obj, key)
}

#[no_mangle]
pub extern "C" fn typthon_dict_set(obj: PyObject, key: PyObject, value: PyObject) {
    py_dict_set(obj, key, value)
}

#[no_mangle]
pub extern "C" fn typthon_dict_contains(obj: PyObject, key: PyObject) -> bool {
    py_dict_contains(obj, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::init as init_allocator;
    use crate::gc::init as init_gc;

    #[test]
    fn test_dict_creation() {
        init_allocator();
        init_gc();

        let dict = py_dict_new();
        assert_eq!(py_dict_len(dict), 0);
    }

    #[test]
    fn test_dict_set_get() {
        init_allocator();
        init_gc();

        let dict = py_dict_new();
        let key = PyObject::from_int(42);
        let value = PyObject::from_int(100);

        py_dict_set(dict, key, value);
        assert_eq!(py_dict_len(dict), 1);
        assert_eq!(py_dict_get(dict, key).as_int(), 100);
    }

    #[test]
    fn test_dict_contains() {
        init_allocator();
        init_gc();

        let dict = py_dict_new();
        let key1 = PyObject::from_int(1);
        let key2 = PyObject::from_int(2);

        py_dict_set(dict, key1, PyObject::from_int(100));

        assert!(py_dict_contains(dict, key1));
        assert!(!py_dict_contains(dict, key2));
    }
}

