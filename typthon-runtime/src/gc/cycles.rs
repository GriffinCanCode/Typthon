//! Cycle detection via tricolor mark-sweep
//!
//! Rare operation for circular reference chains that refcounting can't handle.
//! Based on Python's generational GC and Bacon's incremental cycle collector.

use crate::allocator::ObjectHeader;
use dashmap::DashSet;
use parking_lot::Mutex;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use super::roots::get_roots;

/// Global cycle collector state (lock-free + fine-grained locking)
static COLLECTOR: Lazy<CycleCollector> = Lazy::new(CycleCollector::new);

/// Tricolor marking states for cycle detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    White,  // Unreachable (potential garbage)
    Gray,   // Reachable, children not yet scanned
    Black,  // Reachable, children scanned
}

/// Cycle collector using tricolor mark-sweep with lock-free registration
struct CycleCollector {
    /// Candidate objects (lock-free concurrent set)
    candidates: DashSet<*mut ObjectHeader>,

    /// Gray set for incremental marking (mutex-protected during collection only)
    gray_set: Mutex<VecDeque<*mut ObjectHeader>>,

    /// Collection lock (prevents concurrent collections)
    collection_lock: Mutex<()>,

    /// Statistics (lock-free)
    total_objects: AtomicUsize,
    reachable_objects: AtomicUsize,
    cycles_collected: AtomicUsize,
    collections_run: AtomicUsize,
}

// Safety: Raw pointers are only dereferenced under proper synchronization
unsafe impl Send for CycleCollector {}
unsafe impl Sync for CycleCollector {}

impl CycleCollector {
    fn new() -> Self {
        Self {
            candidates: DashSet::with_capacity(1024),
            gray_set: Mutex::new(VecDeque::with_capacity(256)),
            collection_lock: Mutex::new(()),
            total_objects: AtomicUsize::new(0),
            reachable_objects: AtomicUsize::new(0),
            cycles_collected: AtomicUsize::new(0),
            collections_run: AtomicUsize::new(0),
        }
    }

    /// Run full mark-sweep collection cycle (synchronized)
    fn collect(&self) {
        // Only one collection at a time
        let _guard = self.collection_lock.lock();

        if self.candidates.is_empty() {
            return;
        }

        self.collections_run.fetch_add(1, Ordering::Relaxed);
        self.total_objects.store(self.candidates.len(), Ordering::Relaxed);
        self.reachable_objects.store(0, Ordering::Relaxed);

        // Phase 1: Mark all as white (assume garbage)
        self.mark_white();

        // Phase 2: Mark reachable from roots as gray
        self.mark_from_roots();

        // Phase 3: Propagate gray to black (mark children)
        self.propagate_marks();

        // Phase 4: Sweep white objects (unreachable cycles)
        self.sweep();

        // Clear candidates for next cycle
        self.candidates.clear();
    }

    /// Phase 1: Initialize all candidates as white
    fn mark_white(&self) {
        self.candidates.iter().for_each(|entry| {
            let header_ptr = *entry.key();
            unsafe {
                // Use flags field bits 0-1 for color marking
                // 00 = white, 01 = gray, 10 = black
                (*header_ptr).flags &= !0b11;
            }
        });
    }

    /// Phase 2: Mark roots and objects reachable from them as gray
    fn mark_from_roots(&self) {
        let roots = get_roots();

        for root_ptr in roots {
            if self.candidates.contains(&root_ptr) {
                self.mark_gray(root_ptr);
            }
        }
    }

    /// Phase 3: Propagate gray marks to children (mark children gray, promote to black)
    fn propagate_marks(&self) {
        loop {
            let obj_ptr = {
                let mut gray = self.gray_set.lock();
                gray.pop_front()
            };

            match obj_ptr {
                None => break,
                Some(ptr) => {
                    unsafe {
                        // Mark this object as black
                        self.mark_black(ptr);

                        // Mark children as gray
                        // TODO: Traverse object graph based on type info
                        self.mark_children(ptr);
                    }

                    self.reachable_objects.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }

    /// Phase 4: Free white objects (unreachable cycles)
    fn sweep(&self) {
        let to_free: Vec<*mut ObjectHeader> = self.candidates
            .iter()
            .filter_map(|entry| {
                let header_ptr = *entry.key();
                unsafe {
                    let color = self.get_color(header_ptr);
                    let refcount = (*header_ptr).refcount.load(std::sync::atomic::Ordering::Relaxed);

                    if color == Color::White && refcount > 0 {
                        // Found a cycle: refcount > 0 but unreachable
                        Some(header_ptr)
                    } else {
                        None
                    }
                }
            })
            .collect();

        let count = to_free.len();
        self.cycles_collected.fetch_add(count, Ordering::Relaxed);

        // Free collected cycles
        for header_ptr in to_free {
            unsafe {
                self.free_cycle(header_ptr);
            }
        }
    }

    /// Mark object as gray (discovered, needs scanning)
    fn mark_gray(&self, obj: *mut ObjectHeader) {
        unsafe {
            (*obj).flags = ((*obj).flags & !0b11) | 0b01;
        }
        self.gray_set.lock().push_back(obj);
    }

    /// Mark object as black (scanned)
    fn mark_black(&self, obj: *mut ObjectHeader) {
        unsafe {
            (*obj).flags = ((*obj).flags & !0b11) | 0b10;
        }
    }

    /// Get current color of object
    fn get_color(&self, obj: *mut ObjectHeader) -> Color {
        unsafe {
            match (*obj).flags & 0b11 {
                0b00 => Color::White,
                0b01 => Color::Gray,
                0b10 => Color::Black,
                _ => Color::White,
            }
        }
    }

    /// Mark children of object as gray
    unsafe fn mark_children(&self, _obj: *mut ObjectHeader) {
        // TODO: Implement type-specific child traversal
        // Would need to walk object fields based on type_info
        // For container types (list, dict, etc.)
        // This is where we'd traverse:
        // - List items
        // - Dict keys/values
        // - Object attributes
        // - Closure captures
    }

    /// Free cycle by forcing refcount to 0
    unsafe fn free_cycle(&self, header: *mut ObjectHeader) {
        // Force refcount to 0 to trigger destruction
        (*header).refcount.store(1, std::sync::atomic::Ordering::Relaxed);

        // Call destructor if present
        if let Some(drop_fn) = (*header).type_info.as_ref().drop {
            let obj_ptr = (header as *mut u8).add(core::mem::size_of::<ObjectHeader>());
            drop_fn(obj_ptr);
        }

        // TODO: Return memory to allocator arena
    }
}

/// Initialize cycle collector (idempotent)
pub(super) fn init_collector() {
    Lazy::force(&COLLECTOR);
}

/// Run mark-sweep cycle collection
pub fn collect_cycles() {
    COLLECTOR.collect();
}

/// Register object as potential cycle candidate (lock-free)
///
/// Called when:
/// - Refcount decreases but stays > 0
/// - Object is part of a container type
/// - Circular reference suspected
#[inline]
pub fn register_potential_cycle(header: *mut ObjectHeader) {
    COLLECTOR.candidates.insert(header);
}

/// Get collector statistics (lock-free reads)
pub(super) fn collector_stats() -> super::GcStats {
    super::GcStats {
        total_objects: COLLECTOR.total_objects.load(Ordering::Relaxed),
        reachable_objects: COLLECTOR.reachable_objects.load(Ordering::Relaxed),
        cycles_collected: COLLECTOR.cycles_collected.load(Ordering::Relaxed),
        collections_run: 0, // Filled by caller
    }
}
