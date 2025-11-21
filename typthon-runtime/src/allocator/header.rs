//! Object metadata - layout primitives for runtime
//!
//! Design: Cache-aligned headers for optimal memory access patterns.
//! All structures are C-compatible for FFI.

use core::ptr::NonNull;
use std::sync::atomic::AtomicU32;

/// Object header (16 bytes) - prefixed before every heap object
///
/// Layout optimized for:
/// - 8-byte alignment on all architectures
/// - Single cache line access with small objects
/// - Fast refcount operations without pointer chasing
/// - Thread-safe atomic refcounting
#[repr(C, align(8))]
pub struct ObjectHeader {
    pub type_info: NonNull<TypeInfo>,
    pub refcount: AtomicU32,
    pub flags: u32,
}

impl ObjectHeader {
    /// Create header for new object
    #[inline]
    pub const fn new(type_info: NonNull<TypeInfo>) -> Self {
        Self {
            type_info,
            refcount: AtomicU32::new(1),
            flags: 0,
        }
    }

    /// Get header from object pointer (header is 16 bytes before object data)
    #[inline]
    pub unsafe fn from_object(obj: *mut u8) -> *mut Self {
        obj.sub(16) as *mut Self
    }
}

/// Type metadata - immutable per-type information
///
/// Shared across all instances of a type for minimal memory overhead.
#[repr(C)]
pub struct TypeInfo {
    pub size: usize,
    pub align: usize,
    pub drop: Option<unsafe fn(*mut u8)>,
}

impl TypeInfo {
    /// Create type info for simple types (no drop)
    #[inline]
    pub const fn simple(size: usize, align: usize) -> Self {
        Self { size, align, drop: None }
    }

    /// Create type info with custom destructor
    #[inline]
    pub const fn with_drop(size: usize, align: usize, drop: unsafe fn(*mut u8)) -> Self {
        Self { size, align, drop: Some(drop) }
    }
}

