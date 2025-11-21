//! Type marshaling - Python ↔ C conversions
//!
//! Design: Zero-copy when possible, minimal overhead for scalar types

use super::types::{FfiType, FfiValue};
use crate::allocator::ObjectHeader;
use core::ptr;

/// Marshal Python object to C value
///
/// Performance: Inline for scalar types, minimal branching
pub fn to_c(obj: *const u8, target_type: FfiType) -> FfiValue {
    if obj.is_null() {
        return FfiValue::null();
    }

    unsafe {
        let header = ObjectHeader::from_object(obj as *mut u8);
        let _type_info = (*header).type_info.as_ref();

        // Fast path: scalar types with matching sizes
        match target_type {
            FfiType::I8 => FfiValue { i8: *(obj as *const i8) },
            FfiType::I16 => FfiValue { i16: *(obj as *const i16) },
            FfiType::I32 => FfiValue { i32: *(obj as *const i32) },
            FfiType::I64 => FfiValue { i64: *(obj as *const i64) },

            FfiType::U8 => FfiValue { u8: *(obj as *const u8) },
            FfiType::U16 => FfiValue { u16: *(obj as *const u16) },
            FfiType::U32 => FfiValue { u32: *(obj as *const u32) },
            FfiType::U64 => FfiValue { u64: *(obj as *const u64) },

            FfiType::F32 => FfiValue { f32: *(obj as *const f32) },
            FfiType::F64 => FfiValue { f64: *(obj as *const f64) },

            FfiType::Bool => FfiValue { boolean: *(obj as *const bool) },
            FfiType::Pointer | FfiType::String => {
                FfiValue::from_ptr(obj as *const core::ffi::c_void)
            }

            FfiType::Void => FfiValue::void(),
        }
    }
}

/// Marshal C value to Python object
///
/// Performance: Inline allocation, single-pass conversion
pub fn from_c(value: FfiValue, source_type: FfiType) -> *const u8 {
    // TODO: Integrate with allocator to create proper Python objects
    // For now, return raw pointer representation

    unsafe {
        match source_type {
            FfiType::Void => ptr::null(),
            FfiType::Pointer | FfiType::String => value.ptr as *const u8,

            // Scalar types need boxing into Python objects
            // This requires allocator integration
            _ => {
                // Temporary: Return null for scalar types
                // Production: use allocator::Allocator::alloc_object()
                ptr::null()
            }
        }
    }
}

/// Marshal array of Python objects to C values
///
/// Performance: Vectorized for cache efficiency
pub fn marshal_args(
    args: &[*const u8],
    types: &[FfiType],
    out: &mut [FfiValue],
) {
    debug_assert_eq!(args.len(), types.len());
    debug_assert_eq!(args.len(), out.len());

    for i in 0..args.len() {
        out[i] = to_c(args[i], types[i]);
    }
}

/// Check if types are compatible for zero-copy marshaling
#[inline]
pub const fn can_zero_copy(py_size: usize, py_align: usize, ffi_type: FfiType) -> bool {
    py_size == ffi_type.size() && py_align >= ffi_type.align()
}

/// Get Python type code for FFI type (for error messages)
pub const fn python_type_name(ffi_type: FfiType) -> &'static str {
    match ffi_type {
        FfiType::Void => "None",
        FfiType::Bool => "bool",
        FfiType::I8 | FfiType::I16 | FfiType::I32 | FfiType::I64 => "int",
        FfiType::U8 | FfiType::U16 | FfiType::U32 | FfiType::U64 => "int",
        FfiType::F32 | FfiType::F64 => "float",
        FfiType::Pointer => "int",
        FfiType::String => "str",
    }
}
