//! Arena management - OS memory acquisition
//!
//! Design: Lazy allocation of large blocks (64KB-4MB) for minimal syscalls.
//! Future: Thread-local arenas for zero contention.

use std::alloc::{alloc, dealloc, Layout};

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

        let start = unsafe { alloc(layout) };
        if start.is_null() {
            return None;
        }

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
        let arena = Arena::new(self.current_size)?;

        // Grow arena size for next allocation (capped at MAX_ARENA_SIZE)
        self.current_size = (self.current_size * 2).min(MAX_ARENA_SIZE);

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

