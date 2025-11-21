//! Type definitions for FFI interoperability
//!
//! Defines type representations compatible with C ABIs.

/// FFI-compatible type descriptor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FfiType {
    Void,
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Pointer,
    String,
}

impl FfiType {
    /// Get size of type in bytes
    #[inline]
    pub const fn size(self) -> usize {
        match self {
            Self::Void => 0,
            Self::Bool | Self::I8 | Self::U8 => 1,
            Self::I16 | Self::U16 => 2,
            Self::I32 | Self::U32 | Self::F32 => 4,
            Self::I64 | Self::U64 | Self::F64 | Self::Pointer | Self::String => 8,
        }
    }

    /// Get alignment requirement
    #[inline]
    pub const fn align(self) -> usize {
        self.size()
    }

    /// Check if type is integral
    #[inline]
    pub const fn is_integral(self) -> bool {
        matches!(self, Self::I8 | Self::I16 | Self::I32 | Self::I64 |
                      Self::U8 | Self::U16 | Self::U32 | Self::U64)
    }

    /// Check if type is floating point
    #[inline]
    pub const fn is_float(self) -> bool {
        matches!(self, Self::F32 | Self::F64)
    }
}

/// FFI value container (untagged union)
#[repr(C)]
pub union FfiValue {
    pub void: (),
    pub boolean: bool,
    pub i8: i8,
    pub i16: i16,
    pub i32: i32,
    pub i64: i64,
    pub u8: u8,
    pub u16: u16,
    pub u32: u32,
    pub u64: u64,
    pub f32: f32,
    pub f64: f64,
    pub ptr: *const core::ffi::c_void,
}

impl FfiValue {
    /// Create void value
    #[inline]
    pub const fn void() -> Self {
        Self { void: () }
    }

    /// Create null pointer
    #[inline]
    pub const fn null() -> Self {
        Self { ptr: core::ptr::null() }
    }

    /// Create from pointer
    #[inline]
    pub const fn from_ptr(ptr: *const core::ffi::c_void) -> Self {
        Self { ptr }
    }
}

impl Default for FfiValue {
    #[inline]
    fn default() -> Self {
        Self::void()
    }
}

// Manual implementations for Copy, Clone, and Debug since union doesn't auto-derive
impl Copy for FfiValue {}
impl Clone for FfiValue {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl core::fmt::Debug for FfiValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FfiValue {{ ... }}")
    }
}

/// Type-safe FFI argument
#[derive(Debug, Clone, Copy)]
pub struct TypedArg {
    pub ty: FfiType,
    pub value: FfiValue,
}

impl TypedArg {
    /// Create typed argument
    #[inline]
    pub const fn new(ty: FfiType, value: FfiValue) -> Self {
        Self { ty, value }
    }
}
