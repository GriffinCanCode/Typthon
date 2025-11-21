//! Bump pointer allocation - O(1) fast path
//!
//! Design: Branch-prediction optimized for sequential allocations.
//! Target: <10ns per allocation on modern hardware.

use core::ptr::NonNull;

/// Bump allocator state - minimal overhead
pub struct BumpAllocator {
    current: *mut u8,
    end: *mut u8,
}

impl BumpAllocator {
    /// Create empty allocator (requires arena initialization)
    #[inline]
    pub const fn new() -> Self {
        Self {
            current: core::ptr::null_mut(),
            end: core::ptr::null_mut(),
        }
    }

    /// Fast path: bump pointer allocation
    ///
    /// Returns None if arena exhausted (caller handles slow path).
    /// Inlined for zero-cost abstraction.
    #[inline(always)]
    pub fn try_alloc(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        debug_assert!(align.is_power_of_two(), "alignment must be power of 2");

        let ptr = align_up(self.current as usize, align);
        let new_current = ptr.checked_add(size)?;

        if new_current <= self.end as usize {
            self.current = new_current as *mut u8;
            NonNull::new(ptr as *mut u8)
        } else {
            None
        }
    }

    /// Reset to new arena bounds
    #[inline]
    pub fn reset(&mut self, start: *mut u8, end: *mut u8) {
        debug_assert!(start <= end, "invalid arena bounds");
        self.current = start;
        self.end = end;
    }

    /// Remaining capacity in current arena
    #[inline]
    pub fn remaining(&self) -> usize {
        (self.end as usize).saturating_sub(self.current as usize)
    }
}

/// Align address upward to next multiple of alignment
///
/// Uses bit manipulation for branch-free execution:
/// - Add (align - 1) to round up
/// - Mask with !(align - 1) to align down
#[inline(always)]
const fn align_up(addr: usize, align: usize) -> usize {
    (addr.wrapping_add(align).wrapping_sub(1)) & !align.wrapping_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 8), 0);
        assert_eq!(align_up(1, 8), 8);
        assert_eq!(align_up(8, 8), 8);
        assert_eq!(align_up(9, 8), 16);
    }
}

