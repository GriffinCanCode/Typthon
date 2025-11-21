//! Length operations - collection size queries
//!
//! Design: Fast path reads length from object header without indirection.
//! Generic trait for extensibility.

/// Trait for types with computable length
pub trait HasLen {
    fn len(&self) -> usize;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Length query (C FFI export)
///
/// # Layout assumption
/// Objects store length at fixed offset in header (8 bytes after type_info).
#[no_mangle]
pub extern "C" fn typthon_len(obj: *const u8) -> usize {
    if obj.is_null() {
        return 0;
    }

    unsafe { len_raw(obj) }
}

/// Safe internal implementation
#[inline]
unsafe fn len_raw(obj: *const u8) -> usize {
    // Future: Read from object header layout
    // For now, assume length is stored at fixed offset
    let header = obj.cast::<ObjectHeader>();
    (*header).length
}

/// Safe Rust API - compute length
#[inline]
pub fn len<T: HasLen + ?Sized>(obj: &T) -> usize {
    obj.len()
}

/// Placeholder for object header (future: unify with allocator)
#[repr(C)]
struct ObjectHeader {
    type_info: *const u8,
    length: usize,
}

// Standard library implementations
impl<T> HasLen for [T] {
    #[inline]
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl HasLen for str {
    #[inline]
    fn len(&self) -> usize {
        str::len(self)
    }
}

impl<T> HasLen for &[T] {
    #[inline]
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T, const N: usize> HasLen for [T; N] {
    #[inline]
    fn len(&self) -> usize {
        N
    }
}
