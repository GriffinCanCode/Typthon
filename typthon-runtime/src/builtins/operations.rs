//! Polymorphic operations - type-generic arithmetic and comparison
//!
//! Design: Fast dispatch based on object type with:
//! - Branch prediction friendly code
//! - Automatic type coercion (int → float)
//! - Python semantics for mixed-type operations
//! - Optimized fast paths for common cases

use crate::objects::{PyObject, ObjectType};
use crate::builtins::{
    py_float_new, py_float_as_f64, py_float_add, py_float_sub, py_float_mul,
    py_string_concat, py_string_eq, py_string_cmp,
};

/// Add two objects (polymorphic)
pub fn py_add(a: PyObject, b: PyObject) -> PyObject {
    // Fast path: both integers
    if a.is_int() && b.is_int() {
        return PyObject::from_int(a.as_int() + b.as_int());
    }

    let type_a = a.get_type();
    let type_b = b.get_type();

    match (type_a, type_b) {
        // Int operations
        (ObjectType::Int, ObjectType::Float) => {
            let fa = py_float_new(a.as_int() as f64);
            py_float_add(fa, b)
        }
        (ObjectType::Float, ObjectType::Int) => {
            let fb = py_float_new(b.as_int() as f64);
            py_float_add(a, fb)
        }

        // Float operations
        (ObjectType::Float, ObjectType::Float) => py_float_add(a, b),

        // String concatenation
        (ObjectType::String, ObjectType::String) => py_string_concat(a, b),

        // List concatenation
        (ObjectType::List, ObjectType::List) => list_concat(a, b),

        _ => panic!("Unsupported operand types for +: {:?} and {:?}", type_a, type_b),
    }
}

/// Subtract two objects (polymorphic)
pub fn py_sub(a: PyObject, b: PyObject) -> PyObject {
    // Fast path: both integers
    if a.is_int() && b.is_int() {
        return PyObject::from_int(a.as_int() - b.as_int());
    }

    let type_a = a.get_type();
    let type_b = b.get_type();

    match (type_a, type_b) {
        (ObjectType::Int, ObjectType::Float) => {
            let fa = py_float_new(a.as_int() as f64);
            py_float_sub(fa, b)
        }
        (ObjectType::Float, ObjectType::Int) => {
            let fb = py_float_new(b.as_int() as f64);
            py_float_sub(a, fb)
        }
        (ObjectType::Float, ObjectType::Float) => py_float_sub(a, b),

        _ => panic!("Unsupported operand types for -: {:?} and {:?}", type_a, type_b),
    }
}

/// Multiply two objects (polymorphic)
pub fn py_mul(a: PyObject, b: PyObject) -> PyObject {
    // Fast path: both integers
    if a.is_int() && b.is_int() {
        return PyObject::from_int(a.as_int() * b.as_int());
    }

    let type_a = a.get_type();
    let type_b = b.get_type();

    match (type_a, type_b) {
        (ObjectType::Int, ObjectType::Float) => {
            let fa = py_float_new(a.as_int() as f64);
            py_float_mul(fa, b)
        }
        (ObjectType::Float, ObjectType::Int) => {
            let fb = py_float_new(b.as_int() as f64);
            py_float_mul(a, fb)
        }
        (ObjectType::Float, ObjectType::Float) => py_float_mul(a, b),

        // String repetition
        (ObjectType::String, ObjectType::Int) => string_repeat(a, b.as_int()),
        (ObjectType::Int, ObjectType::String) => string_repeat(b, a.as_int()),

        // List repetition
        (ObjectType::List, ObjectType::Int) => list_repeat(a, b.as_int()),
        (ObjectType::Int, ObjectType::List) => list_repeat(b, a.as_int()),

        _ => panic!("Unsupported operand types for *: {:?} and {:?}", type_a, type_b),
    }
}

/// Divide two objects (polymorphic, always returns float)
pub fn py_div(a: PyObject, b: PyObject) -> PyObject {
    let val_a = if a.is_int() {
        a.as_int() as f64
    } else if a.get_type() == ObjectType::Float {
        py_float_as_f64(a)
    } else {
        panic!("Unsupported operand type for /: {:?}", a.get_type());
    };

    let val_b = if b.is_int() {
        b.as_int() as f64
    } else if b.get_type() == ObjectType::Float {
        py_float_as_f64(b)
    } else {
        panic!("Unsupported operand type for /: {:?}", b.get_type());
    };

    if val_b == 0.0 {
        panic!("Division by zero");
    }

    py_float_new(val_a / val_b)
}

/// Integer division (floor division)
pub fn py_floordiv(a: PyObject, b: PyObject) -> PyObject {
    if a.is_int() && b.is_int() {
        let denom = b.as_int();
        if denom == 0 {
            panic!("Division by zero");
        }
        return PyObject::from_int(a.as_int() / denom);
    }

    let result = py_div(a, b);
    let val = py_float_as_f64(result);
    PyObject::from_int(val.floor() as i64)
}

/// Modulo operation
pub fn py_mod(a: PyObject, b: PyObject) -> PyObject {
    if a.is_int() && b.is_int() {
        let denom = b.as_int();
        if denom == 0 {
            panic!("Modulo by zero");
        }
        return PyObject::from_int(a.as_int() % denom);
    }

    panic!("Unsupported operand types for %: {:?} and {:?}", a.get_type(), b.get_type());
}

/// Power operation
pub fn py_pow(a: PyObject, b: PyObject) -> PyObject {
    let val_a = if a.is_int() {
        a.as_int() as f64
    } else if a.get_type() == ObjectType::Float {
        py_float_as_f64(a)
    } else {
        panic!("Unsupported operand type for **: {:?}", a.get_type());
    };

    let val_b = if b.is_int() {
        b.as_int() as f64
    } else if b.get_type() == ObjectType::Float {
        py_float_as_f64(b)
    } else {
        panic!("Unsupported operand type for **: {:?}", b.get_type());
    };

    py_float_new(val_a.powf(val_b))
}

/// Equality comparison (polymorphic)
pub fn py_eq(a: PyObject, b: PyObject) -> bool {
    // Fast path: both integers
    if a.is_int() && b.is_int() {
        return a.as_int() == b.as_int();
    }

    // Special values
    if a.is_special() && b.is_special() {
        return a.get_type() == b.get_type();
    }

    let type_a = a.get_type();
    let type_b = b.get_type();

    // Different types are generally not equal (except numeric coercion)
    if type_a != type_b {
        match (type_a, type_b) {
            (ObjectType::Int, ObjectType::Float) | (ObjectType::Float, ObjectType::Int) => {
                let fa = if a.is_int() { a.as_int() as f64 } else { py_float_as_f64(a) };
                let fb = if b.is_int() { b.as_int() as f64 } else { py_float_as_f64(b) };
                return fa == fb;
            }
            _ => return false,
        }
    }

    // Same type comparisons
    match type_a {
        ObjectType::Float => {
            py_float_as_f64(a) == py_float_as_f64(b)
        }
        ObjectType::String => py_string_eq(a, b),
        ObjectType::Tuple => super::tuple::py_tuple_eq(a, b),
        ObjectType::List | ObjectType::Dict | ObjectType::Function |
        ObjectType::Class | ObjectType::Instance => {
            // Identity comparison for mutable types
            a.as_ptr() == b.as_ptr()
        }
        _ => false,
    }
}

/// Inequality comparison
pub fn py_ne(a: PyObject, b: PyObject) -> bool {
    !py_eq(a, b)
}

/// Less than comparison
pub fn py_lt(a: PyObject, b: PyObject) -> bool {
    py_cmp(a, b) < 0
}

/// Less than or equal comparison
pub fn py_le(a: PyObject, b: PyObject) -> bool {
    py_cmp(a, b) <= 0
}

/// Greater than comparison
pub fn py_gt(a: PyObject, b: PyObject) -> bool {
    py_cmp(a, b) > 0
}

/// Greater than or equal comparison
pub fn py_ge(a: PyObject, b: PyObject) -> bool {
    py_cmp(a, b) >= 0
}

/// Generic comparison (-1, 0, 1)
fn py_cmp(a: PyObject, b: PyObject) -> i32 {
    // Fast path: both integers
    if a.is_int() && b.is_int() {
        let va = a.as_int();
        let vb = b.as_int();
        return if va < vb { -1 } else if va > vb { 1 } else { 0 };
    }

    let type_a = a.get_type();
    let type_b = b.get_type();

    match (type_a, type_b) {
        (ObjectType::Int, ObjectType::Float) => {
            let fa = a.as_int() as f64;
            let fb = py_float_as_f64(b);
            if fa < fb { -1 } else if fa > fb { 1 } else { 0 }
        }
        (ObjectType::Float, ObjectType::Int) => {
            let fa = py_float_as_f64(a);
            let fb = b.as_int() as f64;
            if fa < fb { -1 } else if fa > fb { 1 } else { 0 }
        }
        (ObjectType::Float, ObjectType::Float) => {
            let fa = py_float_as_f64(a);
            let fb = py_float_as_f64(b);
            if fa < fb { -1 } else if fa > fb { 1 } else { 0 }
        }
        (ObjectType::String, ObjectType::String) => py_string_cmp(a, b),
        _ => panic!("Cannot compare {:?} and {:?}", type_a, type_b),
    }
}

/// Helper: concatenate two lists
fn list_concat(a: PyObject, b: PyObject) -> PyObject {
    use crate::builtins::{py_list_new, py_list_len, py_list_get, py_list_append};

    let result = py_list_new();

    // Append all elements from first list
    for i in 0..py_list_len(a) {
        py_list_append(result, py_list_get(a, i as isize));
    }

    // Append all elements from second list
    for i in 0..py_list_len(b) {
        py_list_append(result, py_list_get(b, i as isize));
    }

    result
}

/// Helper: repeat list n times
fn list_repeat(list: PyObject, count: i64) -> PyObject {
    use crate::builtins::{py_list_new, py_list_len, py_list_get, py_list_append};

    if count <= 0 {
        return py_list_new();
    }

    let result = py_list_new();
    let len = py_list_len(list);

    for _ in 0..count {
        for i in 0..len {
            py_list_append(result, py_list_get(list, i as isize));
        }
    }

    result
}

/// Helper: repeat string n times
fn string_repeat(s: PyObject, count: i64) -> PyObject {
    use crate::builtins::{py_string_new, py_string_as_str};

    if count <= 0 {
        return py_string_new("");
    }

    let base = py_string_as_str(s);
    let repeated = base.repeat(count as usize);
    py_string_new(&repeated)
}

/// C FFI exports
#[no_mangle]
pub extern "C" fn typthon_add(a: PyObject, b: PyObject) -> PyObject {
    py_add(a, b)
}

#[no_mangle]
pub extern "C" fn typthon_sub(a: PyObject, b: PyObject) -> PyObject {
    py_sub(a, b)
}

#[no_mangle]
pub extern "C" fn typthon_mul(a: PyObject, b: PyObject) -> PyObject {
    py_mul(a, b)
}

#[no_mangle]
pub extern "C" fn typthon_div(a: PyObject, b: PyObject) -> PyObject {
    py_div(a, b)
}

#[no_mangle]
pub extern "C" fn typthon_floordiv(a: PyObject, b: PyObject) -> PyObject {
    py_floordiv(a, b)
}

#[no_mangle]
pub extern "C" fn typthon_mod(a: PyObject, b: PyObject) -> PyObject {
    py_mod(a, b)
}

#[no_mangle]
pub extern "C" fn typthon_pow(a: PyObject, b: PyObject) -> PyObject {
    py_pow(a, b)
}

#[no_mangle]
pub extern "C" fn typthon_eq(a: PyObject, b: PyObject) -> bool {
    py_eq(a, b)
}

#[no_mangle]
pub extern "C" fn typthon_ne(a: PyObject, b: PyObject) -> bool {
    py_ne(a, b)
}

#[no_mangle]
pub extern "C" fn typthon_lt(a: PyObject, b: PyObject) -> bool {
    py_lt(a, b)
}

#[no_mangle]
pub extern "C" fn typthon_le(a: PyObject, b: PyObject) -> bool {
    py_le(a, b)
}

#[no_mangle]
pub extern "C" fn typthon_gt(a: PyObject, b: PyObject) -> bool {
    py_gt(a, b)
}

#[no_mangle]
pub extern "C" fn typthon_ge(a: PyObject, b: PyObject) -> bool {
    py_ge(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::init as init_allocator;
    use crate::gc::init as init_gc;

    #[test]
    fn test_int_arithmetic() {
        init_allocator();
        init_gc();

        let a = PyObject::from_int(10);
        let b = PyObject::from_int(3);

        assert_eq!(py_add(a, b).as_int(), 13);
        assert_eq!(py_sub(a, b).as_int(), 7);
        assert_eq!(py_mul(a, b).as_int(), 30);
        assert_eq!(py_floordiv(a, b).as_int(), 3);
        assert_eq!(py_mod(a, b).as_int(), 1);
    }

    #[test]
    fn test_mixed_arithmetic() {
        init_allocator();
        init_gc();

        let i = PyObject::from_int(5);
        let f = py_float_new(2.5);

        let result = py_add(i, f);
        assert_eq!(py_float_as_f64(result), 7.5);

        let result = py_mul(i, f);
        assert_eq!(py_float_as_f64(result), 12.5);
    }

    #[test]
    fn test_comparisons() {
        init_allocator();
        init_gc();

        let a = PyObject::from_int(5);
        let b = PyObject::from_int(10);
        let c = PyObject::from_int(5);

        assert!(py_lt(a, b));
        assert!(py_le(a, b));
        assert!(py_le(a, c));
        assert!(py_eq(a, c));
        assert!(py_ne(a, b));
        assert!(py_gt(b, a));
        assert!(py_ge(b, a));
    }
}

