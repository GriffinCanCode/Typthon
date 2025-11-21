//! GC root tracking - identify starting points for cycle detection
//!
//! Roots include:
//! - Stack variables (call frames)
//! - Global variables
//! - Thread-local storage

use crate::allocator::ObjectHeader;
use dashmap::DashSet;
use once_cell::sync::Lazy;

/// Global roots registry (lock-free concurrent set)
static ROOTS: Lazy<RootSet> = Lazy::new(RootSet::new);

/// Root set - collection of GC roots using lock-free concurrent set
struct RootSet {
    roots: DashSet<*mut ObjectHeader>,
}

// Safety: Raw pointers are only dereferenced under GC synchronization
unsafe impl Send for RootSet {}
unsafe impl Sync for RootSet {}

impl RootSet {
    fn new() -> Self {
        Self {
            roots: DashSet::with_capacity(128),
        }
    }

    #[inline]
    fn add(&self, obj: *mut ObjectHeader) {
        self.roots.insert(obj);
    }

    #[inline]
    fn remove(&self, obj: *mut ObjectHeader) {
        self.roots.remove(&obj);
    }

    fn clear(&self) {
        self.roots.clear();
    }

    fn get_all(&self) -> Vec<*mut ObjectHeader> {
        self.roots.iter().map(|entry| *entry.key()).collect()
    }
}

/// Initialize root tracking (idempotent)
pub(super) fn init_roots() {
    Lazy::force(&ROOTS);
}

/// Register object as GC root (lock-free)
///
/// Should be called for:
/// - Module globals
/// - Thread-local variables
/// - Stack frame variables (automated by compiler)
#[inline]
pub fn register_root(obj: *mut u8) {
    if obj.is_null() {
        return;
    }

    unsafe {
        let header = ObjectHeader::from_object(obj);
        ROOTS.add(header);
    }
}

/// Unregister GC root (lock-free)
///
/// Called when:
/// - Variable goes out of scope
/// - Module unloaded
/// - Thread terminates
#[inline]
pub fn unregister_root(obj: *mut u8) {
    if obj.is_null() {
        return;
    }

    unsafe {
        let header = ObjectHeader::from_object(obj);
        ROOTS.remove(header);
    }
}

/// Get all current roots (for cycle collector)
pub(super) fn get_roots() -> Vec<*mut ObjectHeader> {
    ROOTS.get_all()
}

/// Clear all roots (cleanup)
pub(super) fn clear_roots() {
    ROOTS.clear();
}

/// RAII guard for automatic root registration/unregistration
///
/// Usage:
/// ```ignore
/// let obj = allocate_object();
/// let _guard = RootGuard::new(obj);
/// // obj is now a GC root
/// // automatically unregistered when _guard drops
/// ```
pub struct RootGuard {
    obj: *mut u8,
}

impl RootGuard {
    #[inline]
    pub fn new(obj: *mut u8) -> Self {
        register_root(obj);
        Self { obj }
    }
}

impl Drop for RootGuard {
    #[inline]
    fn drop(&mut self) {
        unregister_root(self.obj);
    }
}

// Safety: RootGuard manages registration/unregistration correctly
unsafe impl Send for RootGuard {}
unsafe impl Sync for RootGuard {}
