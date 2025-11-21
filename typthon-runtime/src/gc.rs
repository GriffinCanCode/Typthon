//! Garbage collector - reference counting + cycle detection
//!
//! Design: Fast reference counting for common case, mark-sweep for cycles.
//! Inline refcount in object header for cache efficiency.

use crate::allocator::ObjectHeader;

/// Initialize GC
pub fn init() {
    // TODO: Initialize GC state
}

/// Cleanup GC
pub fn cleanup() {
    // TODO: Run final collection
}

/// Reference counted pointer
pub struct RefCount<T> {
    ptr: *mut T,
}

impl<T> RefCount<T> {
    pub fn new(ptr: *mut T) -> Self {
        unsafe {
            let header = (ptr as *mut u8).sub(16) as *mut ObjectHeader;
            (*header).refcount = 1;
        }
        Self { ptr }
    }

    /// Increment reference count (inlined for speed)
    #[inline(always)]
    pub fn inc(&self) {
        unsafe {
            let header = (self.ptr as *mut u8).sub(16) as *mut ObjectHeader;
            (*header).refcount += 1;
        }
    }

    /// Decrement reference count (inlined for speed)
    #[inline(always)]
    pub fn dec(&self) {
        unsafe {
            let header = (self.ptr as *mut u8).sub(16) as *mut ObjectHeader;
            (*header).refcount -= 1;

            if (*header).refcount == 0 {
                self.drop();
            }
        }
    }

    unsafe fn drop(&self) {
        let header = (self.ptr as *mut u8).sub(16) as *mut ObjectHeader;

        // Call custom drop if present
        if let Some(drop_fn) = (*(*header).type_info).drop {
            drop_fn(self.ptr as *mut u8);
        }

        // TODO: Free memory
    }
}

/// Mark-sweep for cycle detection (rare)
pub fn collect_cycles() {
    // TODO: Implement cycle collector
    // 1. Mark reachable objects
    // 2. Sweep unreachable with refcount > 0 (cycles)
}

