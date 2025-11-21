//! Memory allocator - fast bump allocation with arenas
//!
//! Design: Bump pointer allocation for fast alloc, arena deallocation for bulk free.
//! Thread-local arenas for zero contention.

use core::ptr::NonNull;

/// Initialize allocator
pub fn init() {
    // TODO: Initialize global arena
}

/// Allocator using bump allocation
pub struct Allocator {
    current: *mut u8,
    end: *mut u8,
}

impl Allocator {
    pub fn new() -> Self {
        Self {
            current: core::ptr::null_mut(),
            end: core::ptr::null_mut(),
        }
    }

    /// Allocate memory (fast path: bump pointer)
    pub fn alloc(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let ptr = align_up(self.current as usize, align);
        let new_current = ptr + size;

        if new_current <= self.end as usize {
            self.current = new_current as *mut u8;
            NonNull::new(ptr as *mut u8)
        } else {
            // Slow path: allocate new arena
            self.alloc_slow(size, align)
        }
    }

    fn alloc_slow(&mut self, _size: usize, _align: usize) -> Option<NonNull<u8>> {
        // TODO: Allocate new arena from OS
        None
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// Object header (16 bytes)
#[repr(C)]
pub struct ObjectHeader {
    pub type_info: *const TypeInfo,
    pub refcount: u32,
    pub flags: u32,
}

/// Type information
#[repr(C)]
pub struct TypeInfo {
    pub size: usize,
    pub align: usize,
    pub drop: Option<unsafe fn(*mut u8)>,
}

