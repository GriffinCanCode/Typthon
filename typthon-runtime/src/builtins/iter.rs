//! Iterator support - range and sequence iteration
//!
//! Design: Zero-cost abstractions using Rust's iterator protocol.
//! FFI-compatible layout with idiomatic Rust traits.

/// Range iterator - Python's range() equivalent
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    current: i64,
    end: i64,
    step: i64,
}

impl Range {
    /// Create new range
    #[inline]
    pub const fn new(start: i64, end: i64, step: i64) -> Self {
        Self { current: start, end, step }
    }

    /// Check if range is empty
    #[inline]
    pub const fn is_empty(&self) -> bool {
        (self.step > 0 && self.current >= self.end) ||
        (self.step < 0 && self.current <= self.end)
    }

    /// Get remaining elements count
    #[inline]
    pub const fn len(&self) -> usize {
        if self.is_empty() {
            return 0;
        }

        let diff = self.end.wrapping_sub(self.current);
        let count = diff / self.step;

        if count < 0 {
            0
        } else {
            count as usize
        }
    }
}

impl Iterator for Range {
    type Item = i64;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            return None;
        }

        let val = self.current;
        self.current = self.current.wrapping_add(self.step);
        Some(val)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for Range {
    #[inline]
    fn len(&self) -> usize {
        Range::len(self)
    }
}

impl DoubleEndedIterator for Range {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            return None;
        }

        self.end = self.end.wrapping_sub(self.step);
        if self.end < self.current {
            None
        } else {
            Some(self.end)
        }
    }
}

/// Create range (C FFI export)
#[no_mangle]
pub extern "C" fn typthon_range(start: i64, end: i64, step: i64) -> Range {
    range(start, end, step)
}

/// Iterator next (C FFI export)
///
/// Returns value or i64::MIN on exhaustion
#[no_mangle]
pub extern "C" fn typthon_range_next(range: *mut Range) -> i64 {
    if range.is_null() {
        return i64::MIN;
    }

    unsafe { (*range).next().unwrap_or(i64::MIN) }
}

/// Safe Rust API - create range
#[inline]
pub const fn range(start: i64, end: i64, step: i64) -> Range {
    Range::new(start, end, step)
}

