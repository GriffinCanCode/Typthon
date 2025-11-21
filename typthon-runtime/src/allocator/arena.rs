//! Arena management - OS memory acquisition
//!
//! Design: Lazy allocation of large blocks (64KB-4MB) for minimal syscalls.
//! Future: Thread-local arenas for zero contention.

use std::alloc::{alloc, dealloc, Layout};
use crate::logging::{debug, warn, trace};

/// Arena size strategy - balances memory overhead vs syscall frequency
const DEFAULT_ARENA_SIZE: usize = 64 * 1024; // 64KB
const MAX_ARENA_SIZE: usize = 4 * 1024 * 1024; // 4MB

/// Arena metadata - tracks OS-allocated memory regions
pub struct Arena {
    start: *mut u8,
    layout: Layout,
}

impl Arena {
    /// Allocate new arena from OS
    ///
    /// Uses standard allocator for portability.
    /// Future: Direct mmap/VirtualAlloc for zero overhead.
    pub fn new(size: usize) -> Option<Self> {
        let layout = Layout::from_size_align(size, 8).ok()?;

        trace!(size_bytes = size, "Requesting arena from OS");

        let start = unsafe { alloc(layout) };
        if start.is_null() {
            warn!(size_bytes = size, "Failed to allocate arena from OS");
            return None;
        }

        debug!(
            address = ?start,
            size_bytes = size,
            "Arena allocated successfully"
        );

        Some(Self { start, layout })
    }

    /// Get arena bounds for bump allocator
    #[inline]
    pub fn bounds(&self) -> (*mut u8, *mut u8) {
        unsafe {
            (self.start, self.start.add(self.layout.size()))
        }
    }

    /// Arena size
    #[inline]
    pub fn size(&self) -> usize {
        self.layout.size()
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        trace!(
            address = ?self.start,
            size_bytes = self.layout.size(),
            "Deallocating arena"
        );
        unsafe {
            dealloc(self.start, self.layout);
        }
    }
}

/// Arena pool - manages collection of arenas
///
/// Future: Per-thread pools for lock-free allocation
pub struct ArenaPool {
    arenas: Vec<Arena>,
    current_size: usize,
}

impl ArenaPool {
    pub fn new() -> Self {
        Self {
            arenas: Vec::new(),
            current_size: DEFAULT_ARENA_SIZE,
        }
    }

    /// Allocate new arena, growing size adaptively
    pub fn grow(&mut self) -> Option<&Arena> {
        self.grow_with_min(0)
    }

    /// Allocate new arena with minimum size requirement
    pub fn grow_with_min(&mut self, min_size: usize) -> Option<&Arena> {
        let size = self.current_size.max(min_size);

        debug!(
            min_required = min_size,
            allocated_size = size,
            arena_count = self.arenas.len(),
            "Growing arena pool"
        );

        let arena = Arena::new(size)?;

        // Grow arena size for next allocation (capped at MAX_ARENA_SIZE)
        let old_size = self.current_size;
        self.current_size = (self.current_size * 2).min(MAX_ARENA_SIZE);

        if old_size != self.current_size {
            trace!(
                old_size = old_size,
                new_size = self.current_size,
                "Arena size strategy updated"
            );
        }

        self.arenas.push(arena);
        self.arenas.last()
    }

    /// Total allocated memory across all arenas
    pub fn total_allocated(&self) -> usize {
        self.arenas.iter().map(|a| a.size()).sum()
    }
}

impl Drop for ArenaPool {
    fn drop(&mut self) {
        // Arenas automatically deallocate via Drop
    }
}

impl Default for ArenaPool {
    fn default() -> Self {
        Self::new()
    }
}

