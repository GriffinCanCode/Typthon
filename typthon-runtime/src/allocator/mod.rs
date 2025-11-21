//! Memory allocator - fast bump allocation with arenas
//!
//! Design: Three-layer architecture for optimal performance:
//! 1. Bump allocation (fast path, <10ns)
//! 2. Arena management (slow path, amortized cost)
//! 3. OS memory (rare, bulk acquisition)
//!
//! Thread-local arenas planned for zero contention.

mod header;
mod bump;
mod arena;

#[cfg(test)]
mod tests;

pub use header::{ObjectHeader, TypeInfo};
pub use bump::BumpAllocator;
pub use arena::{Arena, ArenaPool};

use core::ptr::NonNull;

/// Initialize allocator subsystem
///
/// TODO: Initialize thread-local allocators for zero-contention allocation
pub fn init() {
    // Global state initialization will go here
}

/// High-level allocator combining bump allocation and arena management
pub struct Allocator {
    bump: BumpAllocator,
    arenas: ArenaPool,
}

impl Allocator {
    pub fn new() -> Self {
        Self {
            bump: BumpAllocator::new(),
            arenas: ArenaPool::new(),
        }
    }

    /// Allocate memory (fast path first, falls back to arena allocation)
    pub fn alloc(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        // Fast path: try bump allocation
        if let Some(ptr) = self.bump.try_alloc(size, align) {
            return Some(ptr);
        }

        // Slow path: allocate new arena
        self.alloc_slow(size, align)
    }

    fn alloc_slow(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        // Acquire new arena from pool, large enough for this allocation
        let arena = self.arenas.grow_with_min(size + align)?;
        let (start, end) = arena.bounds();

        // Reset bump allocator to new arena
        self.bump.reset(start, end);

        // Retry allocation (guaranteed to succeed if arena large enough)
        self.bump.try_alloc(size, align)
    }

    /// Allocate typed object with header
    pub fn alloc_object<T>(&mut self, type_info: NonNull<TypeInfo>) -> Option<NonNull<T>> {
        let total_size = core::mem::size_of::<ObjectHeader>() + core::mem::size_of::<T>();
        let align = core::mem::align_of::<ObjectHeader>().max(core::mem::align_of::<T>());

        let ptr = self.alloc(total_size, align)?;

        unsafe {
            // Write header
            let header_ptr = ptr.as_ptr() as *mut ObjectHeader;
            header_ptr.write(ObjectHeader::new(type_info));

            // Return pointer to data (after header)
            let data_ptr = header_ptr.add(1) as *mut T;
            NonNull::new(data_ptr)
        }
    }

    /// Get allocator statistics
    pub fn stats(&self) -> AllocatorStats {
        AllocatorStats {
            total_allocated: self.arenas.total_allocated(),
            current_arena_remaining: self.bump.remaining(),
        }
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

