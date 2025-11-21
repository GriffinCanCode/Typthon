//! Reference counting smart pointer
//!
//! Optimized for minimal overhead and cache efficiency.
//! Hot path operations are always inlined.
//! Thread-safe atomic refcounting for concurrent access.

use crate::allocator::ObjectHeader;
use crate::logging::trace;
use core::ptr::NonNull;
use core::ops::{Deref, DerefMut};
use core::marker::PhantomData;
use std::sync::atomic::Ordering;

/// Reference counted pointer with inline operations
///
/// Design: Minimal overhead smart pointer with:
/// - Zero-cost cloning (just inc refcount)
/// - Deterministic destruction
/// - Cycle detection integration
pub struct RefCount<T> {
    ptr: NonNull<T>,
    _marker: PhantomData<T>,
}

impl<T> RefCount<T> {
    /// Create new reference counted pointer from raw allocation
    #[inline]
    pub fn new(ptr: *mut T) -> Self {
        debug_assert!(!ptr.is_null());

        unsafe {
            header(ptr).refcount.store(1, Ordering::Relaxed);
        }

        trace!(event = "refcount_new", address = ?ptr, count = 1);

        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            _marker: PhantomData,
        }
    }

    /// Increment reference count (hot path, always inlined)
    #[inline(always)]
    pub fn inc(&self) {
        unsafe {
            let h = header(self.ptr.as_ptr());
            let old = h.refcount.fetch_add(1, Ordering::Relaxed);

            // Overflow detection in debug builds
            debug_assert!(old < u32::MAX, "refcount overflow");
        }
    }

    /// Decrement reference count (hot path, always inlined)
    #[inline(always)]
    pub fn dec(&self) {
        unsafe {
            let h = header(self.ptr.as_ptr());
            let old = h.refcount.fetch_sub(1, Ordering::Release);

            debug_assert!(old > 0, "refcount underflow");

            if old == 1 {
                // Synchronize with all previous decrements
                std::sync::atomic::fence(Ordering::Acquire);
                self.destroy();
            }
        }
    }

    /// Get current reference count (for debugging)
    #[inline]
    pub fn count(&self) -> u32 {
        unsafe { header(self.ptr.as_ptr()).refcount.load(Ordering::Relaxed) }
    }

    /// Mark as potential cycle candidate
    #[inline]
    pub fn mark_potential_cycle(&self) {
        unsafe {
            let header_ptr = ObjectHeader::from_object(self.ptr.as_ptr() as *mut u8);
            super::cycles::register_potential_cycle(header_ptr);
        }
    }

    /// Destroy object and free memory (cold path)
    #[cold]
    unsafe fn destroy(&self) {
        let h = header(self.ptr.as_ptr());

        trace!(event = "refcount_destroy", address = ?self.ptr.as_ptr(), count = 0);

        // Call type-specific destructor if present
        if let Some(drop_fn) = h.type_info.as_ref().drop {
            drop_fn(self.ptr.as_ptr() as *mut u8);
        }

        // TODO: Return memory to allocator
        // Currently memory is never freed (arena-based collection)
    }

    /// Get raw pointer
    #[inline]
    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Convert to raw pointer, consuming self without decrementing refcount
    #[inline]
    pub fn into_raw(self) -> *mut T {
        let ptr = self.ptr.as_ptr();
        core::mem::forget(self);
        ptr
    }

    /// Create from raw pointer without incrementing refcount
    #[inline]
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr),
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for RefCount<T> {
    #[inline]
    fn clone(&self) -> Self {
        self.inc();
        Self {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T> Drop for RefCount<T> {
    #[inline]
    fn drop(&mut self) {
        self.dec();
    }
}

impl<T> Deref for RefCount<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for RefCount<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

// RefCount is thread-safe with atomic refcounting
unsafe impl<T: Send> Send for RefCount<T> {}
unsafe impl<T: Sync> Sync for RefCount<T> {}

/// Get immutable reference to object header
#[inline(always)]
unsafe fn header<T>(ptr: *mut T) -> &'static ObjectHeader {
    &*(ObjectHeader::from_object(ptr as *mut u8))
}
