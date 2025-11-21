//! Comprehensive test suite for interop module

use super::*;
use super::types::*;
use super::marshal::*;
use super::call::*;

// Test helpers
extern "C" fn add_i32(a: i32, b: i32) -> i32 {
    a + b
}

extern "C" fn mul_f64(a: f64, b: f64) -> f64 {
    a * b
}

extern "C" fn identity_ptr(ptr: *const core::ffi::c_void) -> *const core::ffi::c_void {
    ptr
}

extern "C" fn no_args() -> i32 {
    42
}

#[test]
fn test_ffi_type_sizes() {
    assert_eq!(FfiType::I8.size(), 1);
    assert_eq!(FfiType::I16.size(), 2);
    assert_eq!(FfiType::I32.size(), 4);
    assert_eq!(FfiType::I64.size(), 8);
    assert_eq!(FfiType::F32.size(), 4);
    assert_eq!(FfiType::F64.size(), 8);
    assert_eq!(FfiType::Pointer.size(), 8);
}

#[test]
fn test_ffi_type_alignment() {
    assert_eq!(FfiType::I8.align(), 1);
    assert_eq!(FfiType::I16.align(), 2);
    assert_eq!(FfiType::I32.align(), 4);
    assert_eq!(FfiType::I64.align(), 8);
    assert_eq!(FfiType::F64.align(), 8);
}

#[test]
fn test_ffi_value_constructors() {
    let v = FfiValue { i64: 42 };
    unsafe {
        assert_eq!(v.i64, 42);
    }

    let v = FfiValue { f64: 3.14 };
    unsafe {
        assert!((v.f64 - 3.14).abs() < 1e-10);
    }

    let ptr = 0x1234 as *const core::ffi::c_void;
    let v = FfiValue::from_ptr(ptr);
    unsafe {
        assert_eq!(v.ptr, ptr);
    }
}

#[test]
fn test_marshal_null() {
    let result = to_c(core::ptr::null(), FfiType::Pointer);
    unsafe {
        assert!(result.ptr.is_null());
    }
}

#[test]
fn test_zero_copy_check() {
    // f64 → F64 should be zero-copy
    assert!(can_zero_copy(8, 8, FfiType::F64));

    // Smaller alignment should fail
    assert!(!can_zero_copy(8, 4, FfiType::F64));

    // Wrong size should fail
    assert!(!can_zero_copy(4, 8, FfiType::F64));
}

#[test]
fn test_python_type_names() {
    assert_eq!(python_type_name(FfiType::I64), "int");
    assert_eq!(python_type_name(FfiType::F64), "float");
    assert_eq!(python_type_name(FfiType::Pointer), "int");
    assert_eq!(python_type_name(FfiType::Void), "None");
}

#[test]
fn test_ffi_call_no_args() {
    let call = FunctionCall::new(
        no_args as *const (),
        CallingConvention::default(),
        FfiType::I32,
        vec![],
    );

    unsafe {
        let result = call.call(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().i32, 42);
    }
}

#[test]
fn test_ffi_type_checks() {
    assert!(FfiType::I32.is_integral());
    assert!(FfiType::F64.is_float());
    assert!(!FfiType::Void.is_integral());
    assert!(!FfiType::Pointer.is_float());
}

#[test]
fn test_interop_stats() {
    let stats = stats();
    assert_eq!(stats.calls_made, 0);
    assert_eq!(stats.marshaling_errors, 0);
}

#[test]
fn test_typed_arg() {
    let arg = TypedArg::new(FfiType::I32, FfiValue { i32: 42 });
    assert_eq!(arg.ty, FfiType::I32);
    unsafe {
        assert_eq!(arg.value.i32, 42);
    }
}
