//! Float type - IEEE 754 double-precision floats
//!
//! Design: Heap-allocated 64-bit floats with:
//! - Standard IEEE 754 semantics
//! - Fast arithmetic operations
//! - Proper NaN/Infinity handling
//! - Hash support for dict keys

use std::ptr::NonNull;
use crate::objects::{PyObject, ObjectType};
use crate::allocator::{with_thread_allocator, TypeInfo, ObjectHeader};
use crate::gc::maybe_collect;

/// Float data stored on heap
#[repr(C)]
pub struct FloatData {
    value: f64,
}

/// Static type info for floats
static FLOAT_TYPE: TypeInfo = TypeInfo::simple(
    std::mem::size_of::<FloatData>(),
    std::mem::align_of::<FloatData>(),
    ObjectType::Float as u8,
);

/// Create new float from f64
pub fn py_float_new(value: f64) -> PyObject {
    let float_data = FloatData { value };

    let obj = with_thread_allocator(|alloc| {
        let type_info = NonNull::new(&FLOAT_TYPE as *const _ as *mut _).unwrap();
        let ptr = alloc.alloc_object::<FloatData>(type_info)
            .expect("Failed to allocate float object");

        unsafe {
            std::ptr::write(ptr.as_ptr(), float_data);
        }

        PyObject::from_ptr(ptr.cast())
    });

    maybe_collect();
    obj
}

/// Get float value as f64
pub fn py_float_as_f64(obj: PyObject) -> f64 {
    if obj.get_type() != ObjectType::Float {
        panic!("Expected float object");
    }

    unsafe {
        let header = ObjectHeader::from_object(obj.as_ptr().as_ptr() as *mut u8);
        let data_ptr = header.add(1) as *const FloatData;
        (*data_ptr).value
    }
}

/// Float addition
pub fn py_float_add(a: PyObject, b: PyObject) -> PyObject {
    let val_a = py_float_as_f64(a);
    let val_b = py_float_as_f64(b);
    py_float_new(val_a + val_b)
}

/// Float subtraction
pub fn py_float_sub(a: PyObject, b: PyObject) -> PyObject {
    let val_a = py_float_as_f64(a);
    let val_b = py_float_as_f64(b);
    py_float_new(val_a - val_b)
}

/// Float multiplication
pub fn py_float_mul(a: PyObject, b: PyObject) -> PyObject {
    let val_a = py_float_as_f64(a);
    let val_b = py_float_as_f64(b);
    py_float_new(val_a * val_b)
}

/// Float division
pub fn py_float_div(a: PyObject, b: PyObject) -> PyObject {
    let val_a = py_float_as_f64(a);
    let val_b = py_float_as_f64(b);
    py_float_new(val_a / val_b)
}

/// Float negation
pub fn py_float_neg(a: PyObject) -> PyObject {
    let val = py_float_as_f64(a);
    py_float_new(-val)
}

/// Float equality
pub fn py_float_eq(a: PyObject, b: PyObject) -> bool {
    let val_a = py_float_as_f64(a);
    let val_b = py_float_as_f64(b);
    val_a == val_b
}

/// Float comparison
pub fn py_float_cmp(a: PyObject, b: PyObject) -> i32 {
    let val_a = py_float_as_f64(a);
    let val_b = py_float_as_f64(b);

    if val_a < val_b {
        -1
    } else if val_a > val_b {
        1
    } else if val_a == val_b {
        0
    } else {
        // NaN handling
        if val_a.is_nan() && val_b.is_nan() {
            0
        } else if val_a.is_nan() {
            1
        } else {
            -1
        }
    }
}

/// Convert int to float
pub fn py_int_to_float(obj: PyObject) -> PyObject {
    if obj.is_int() {
        py_float_new(obj.as_int() as f64)
    } else {
        panic!("Expected int object");
    }
}

/// Convert float to int (truncates)
pub fn py_float_to_int(obj: PyObject) -> PyObject {
    let val = py_float_as_f64(obj);
    PyObject::from_int(val as i64)
}

/// C FFI exports
#[no_mangle]
pub extern "C" fn typthon_float_new(value: f64) -> PyObject {
    py_float_new(value)
}

#[no_mangle]
pub extern "C" fn typthon_float_value(obj: PyObject) -> f64 {
    py_float_as_f64(obj)
}

#[no_mangle]
pub extern "C" fn typthon_float_add(a: PyObject, b: PyObject) -> PyObject {
    py_float_add(a, b)
}

#[no_mangle]
pub extern "C" fn typthon_float_sub(a: PyObject, b: PyObject) -> PyObject {
    py_float_sub(a, b)
}

#[no_mangle]
pub extern "C" fn typthon_float_mul(a: PyObject, b: PyObject) -> PyObject {
    py_float_mul(a, b)
}

#[no_mangle]
pub extern "C" fn typthon_float_div(a: PyObject, b: PyObject) -> PyObject {
    py_float_div(a, b)
}

#[no_mangle]
pub extern "C" fn typthon_float_neg(a: PyObject) -> PyObject {
    py_float_neg(a)
}

#[no_mangle]
pub extern "C" fn typthon_float_eq(a: PyObject, b: PyObject) -> bool {
    py_float_eq(a, b)
}

#[no_mangle]
pub extern "C" fn typthon_int_to_float(obj: PyObject) -> PyObject {
    py_int_to_float(obj)
}

#[no_mangle]
pub extern "C" fn typthon_float_to_int(obj: PyObject) -> PyObject {
    py_float_to_int(obj)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::init as init_allocator;
    use crate::gc::init as init_gc;

    #[test]
    fn test_float_creation() {
        init_allocator();
        init_gc();

        let f = py_float_new(3.14);
        assert_eq!(py_float_as_f64(f), 3.14);
    }

    #[test]
    fn test_float_arithmetic() {
        init_allocator();
        init_gc();

        let a = py_float_new(10.5);
        let b = py_float_new(2.5);

        assert_eq!(py_float_as_f64(py_float_add(a, b)), 13.0);
        assert_eq!(py_float_as_f64(py_float_sub(a, b)), 8.0);
        assert_eq!(py_float_as_f64(py_float_mul(a, b)), 26.25);
        assert_eq!(py_float_as_f64(py_float_div(a, b)), 4.2);
    }

    #[test]
    fn test_float_conversion() {
        init_allocator();
        init_gc();

        let i = PyObject::from_int(42);
        let f = py_int_to_float(i);
        assert_eq!(py_float_as_f64(f), 42.0);

        let f2 = py_float_new(3.7);
        let i2 = py_float_to_int(f2);
        assert_eq!(i2.as_int(), 3);
    }
}

