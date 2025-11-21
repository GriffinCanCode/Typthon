//! Memory allocator - fast bump allocation with arenas
//!
//! Design: Three-layer architecture for optimal performance:
//! 1. Bump allocation (fast path, <10ns)
//! 2. Arena management (slow path, amortized cost)
//! 3. OS memory (rare, bulk acquisition)
//!
//! Thread-local arenas for zero-contention allocation.

mod header;
mod bump;
mod arena;

#[cfg(test)]
mod tests;

pub use header::{ObjectHeader, TypeInfo};
pub use bump::BumpAllocator;
pub use arena::{Arena, ArenaPool};

use core::ptr::NonNull;
use core::cell::RefCell;
use crate::logging::{info, debug, trace, log_allocation};

thread_local! {
    /// Thread-local allocator for zero-contention fast path
    static TLS_ALLOCATOR: RefCell<Option<Allocator>> = RefCell::new(None);
}

/// Initialize allocator subsystem with thread-local allocators
pub fn init() {
    info!("Allocator subsystem initializing");
    TLS_ALLOCATOR.with(|alloc| {
        *alloc.borrow_mut() = Some(Allocator::new());
    });
    debug!("Allocator ready with thread-local bump allocation and arena management");
}

/// Get or initialize thread-local allocator
pub fn with_thread_allocator<F, R>(f: F) -> R
where
    F: FnOnce(&mut Allocator) -> R,
{
    TLS_ALLOCATOR.with(|alloc| {
        let mut alloc_ref = alloc.borrow_mut();
        let allocator = alloc_ref.get_or_insert_with(Allocator::new);
        f(allocator)
    })
}

/// High-level allocator combining bump allocation and arena management
pub struct Allocator {
    bump: BumpAllocator,
    arenas: ArenaPool,
}

impl Allocator {
    pub fn new() -> Self {
        trace!("Creating new allocator instance");
        Self {
            bump: BumpAllocator::new(),
            arenas: ArenaPool::new(),
        }
    }

    /// Allocate memory (fast path first, falls back to arena allocation)
    pub fn alloc(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        trace!(operation = "alloc_attempt", size_bytes = size, alignment = align);

        // Fast path: try bump allocation
        if let Some(ptr) = self.bump.try_alloc(size, align) {
            log_allocation(size, ptr.as_ptr());
            return Some(ptr);
        }

        // Slow path: allocate new arena
        debug!(size_bytes = size, alignment = align, "Fast path failed, allocating new arena");
        self.alloc_slow(size, align)
    }

    fn alloc_slow(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        // Acquire new arena from pool, large enough for this allocation
        let arena = self.arenas.grow_with_min(size + align)?;
        let (start, end) = arena.bounds();

        debug!(
            arena_start = ?start,
            arena_end = ?end,
            size_bytes = end as usize - start as usize,
            "New arena allocated from pool"
        );

        // Reset bump allocator to new arena
        self.bump.reset(start, end);

        // Retry allocation (guaranteed to succeed if arena large enough)
        let result = self.bump.try_alloc(size, align);
        if let Some(ptr) = result {
            log_allocation(size, ptr.as_ptr());
            Some(ptr)
        } else {
            None
        }
    }

    /// Allocate typed object with header
    pub fn alloc_object<T>(&mut self, type_info: NonNull<TypeInfo>) -> Option<NonNull<T>> {
        let total_size = core::mem::size_of::<ObjectHeader>() + core::mem::size_of::<T>();
        let align = core::mem::align_of::<ObjectHeader>().max(core::mem::align_of::<T>());

        trace!(
            type_name = core::any::type_name::<T>(),
            total_size = total_size,
            alignment = align,
            "Allocating typed object with header"
        );

        let ptr = self.alloc(total_size, align)?;

        unsafe {
            // Write header
            let header_ptr = ptr.as_ptr() as *mut ObjectHeader;
            header_ptr.write(ObjectHeader::new(type_info));

            // Return pointer to data (after header)
            let data_ptr = header_ptr.add(1) as *mut T;
            let result = NonNull::new(data_ptr);

            if result.is_some() {
                trace!(address = ?data_ptr, "Object allocated successfully");
            }

            result
        }
    }

    /// Get allocator statistics
    pub fn stats(&self) -> AllocatorStats {
        let stats = AllocatorStats {
            total_allocated: self.arenas.total_allocated(),
            current_arena_remaining: self.bump.remaining(),
        };

        trace!(
            total_allocated = stats.total_allocated,
            remaining = stats.current_arena_remaining,
            "Allocator statistics retrieved"
        );

        stats
    }
}

impl Default for Allocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Allocator statistics for monitoring and debugging
#[derive(Debug, Clone, Copy)]
pub struct AllocatorStats {
    pub total_allocated: usize,
    pub current_arena_remaining: usize,
}

