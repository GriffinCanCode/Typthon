//! Comprehensive tests for garbage collection system

use super::*;
use crate::allocator::{ObjectHeader, TypeInfo};
use std::sync::Arc;
use std::thread;
use core::ptr::NonNull;

/// Test helper: Create a simple object header
unsafe fn create_test_object() -> *mut ObjectHeader {
    static TYPE_INFO: TypeInfo = TypeInfo {
        size: 64,
        align: 8,
        drop: None,
    };

    let layout = std::alloc::Layout::from_size_align(64 + 16, 8).unwrap();
    let ptr = std::alloc::alloc(layout);
    let header = ptr as *mut ObjectHeader;

    (*header) = ObjectHeader::new(NonNull::from(&TYPE_INFO));
    header
}

unsafe fn free_test_object(header: *mut ObjectHeader) {
    let layout = std::alloc::Layout::from_size_align(64 + 16, 8).unwrap();
    std::alloc::dealloc(header as *mut u8, layout);
}

#[cfg(test)]
mod refcount_tests {
    use super::*;

    #[test]
    fn test_refcount_new() {
        unsafe {
            let header = create_test_object();
            let obj = (header as *mut u8).add(16) as *mut u64;

            let rc = RefCount::new(obj);
            assert_eq!(rc.count(), 1);

            drop(rc);
            free_test_object(header);
        }
    }

    #[test]
    fn test_refcount_inc_dec() {
        unsafe {
            let header = create_test_object();
            let obj = (header as *mut u8).add(16) as *mut u64;

            let rc = RefCount::new(obj);
            assert_eq!(rc.count(), 1);

            rc.inc();
            assert_eq!(rc.count(), 2);

            rc.inc();
            assert_eq!(rc.count(), 3);

            rc.dec();
            assert_eq!(rc.count(), 2);

            drop(rc); // Final dec
            free_test_object(header);
        }
    }

    #[test]
    fn test_refcount_clone() {
        unsafe {
            let header = create_test_object();
            let obj = (header as *mut u8).add(16) as *mut u64;

            let rc1 = RefCount::new(obj);
            assert_eq!(rc1.count(), 1);

            let rc2 = rc1.clone();
            assert_eq!(rc1.count(), 2);
            assert_eq!(rc2.count(), 2);

            drop(rc2);
            assert_eq!(rc1.count(), 1);

            drop(rc1);
            free_test_object(header);
        }
    }

    #[test]
    fn test_refcount_into_raw() {
        unsafe {
            let header = create_test_object();
            let obj = (header as *mut u8).add(16) as *mut u64;

            let rc = RefCount::new(obj);
            let raw = rc.into_raw();

            assert_eq!(raw, obj);
            assert_eq!((*header).refcount.load(std::sync::atomic::Ordering::Relaxed), 1); // Not decremented

            free_test_object(header);
        }
    }

    #[test]
    fn test_refcount_concurrent_inc() {
        // Test thread-safe atomic refcount operations
        unsafe {
            let header = create_test_object();
            let obj = (header as *mut u8).add(16) as *mut u64;
            let rc = Arc::new(RefCount::new(obj));

            let handles: Vec<_> = (0..10)
                .map(|_| {
                    let rc_clone = Arc::clone(&rc);
                    thread::spawn(move || {
                        for _ in 0..100 {
                            rc_clone.inc();
                            thread::yield_now();
                            rc_clone.dec();
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }

            // All inc/dec pairs should balance out
            assert_eq!(rc.count(), 1);

            drop(rc);
            free_test_object(header);
        }
    }
}

#[cfg(test)]
mod roots_tests {
    use super::*;

    #[test]
    fn test_register_unregister_root() {
        init();

        unsafe {
            let header = create_test_object();
            let obj = (header as *mut u8).add(16);

            register_root(obj);
            let roots = roots::get_roots();
            assert!(roots.contains(&header));

            unregister_root(obj);
            let roots = roots::get_roots();
            assert!(!roots.contains(&header));

            free_test_object(header);
        }
    }

    #[test]
    fn test_root_guard() {
        init();

        unsafe {
            let header = create_test_object();
            let obj = (header as *mut u8).add(16);

            {
                let _guard = RootGuard::new(obj);
                let roots = roots::get_roots();
                assert!(roots.contains(&header));
            }

            // Guard dropped, root should be unregistered
            let roots = roots::get_roots();
            assert!(!roots.contains(&header));

            free_test_object(header);
        }
    }

    #[test]
    fn test_null_root_handling() {
        init();

        // Should not panic
        register_root(core::ptr::null_mut());
        unregister_root(core::ptr::null_mut());
    }

    #[test]
    fn test_roots_idempotent() {
        init();

        unsafe {
            let header = create_test_object();
            let obj = (header as *mut u8).add(16);

            // Multiple registrations
            register_root(obj);
            register_root(obj);
            register_root(obj);

            let roots = roots::get_roots();
            let count = roots.iter().filter(|&&r| r == header).count();

            // Should only appear once (set semantics)
            assert_eq!(count, 1);

            // Multiple unregistrations (should not panic)
            unregister_root(obj);
            unregister_root(obj);
            unregister_root(obj);

            free_test_object(header);
        }
    }
}

#[cfg(test)]
mod cycles_tests {
    use super::*;

    #[test]
    fn test_register_potential_cycle() {
        init();

        unsafe {
            let header = create_test_object();
            register_potential_cycle(header);

            // Should not crash
            collect_cycles();

            free_test_object(header);
        }
    }

    #[test]
    fn test_collect_empty() {
        init();

        // Should handle empty candidate set gracefully
        collect_cycles();
    }

    #[test]
    fn test_cycle_collection_stats() {
        init();

        let initial_stats = stats();

        unsafe {
            let headers: Vec<_> = (0..10)
                .map(|_| create_test_object())
                .collect();

            for &header in &headers {
                register_potential_cycle(header);
            }

            collect_cycles();

            let new_stats = stats();
            assert!(new_stats.collections_run >= initial_stats.collections_run);

            for header in headers {
                free_test_object(header);
            }
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_gc_lifecycle() {
        init();

        unsafe {
            // Create objects
            let obj1 = create_test_object();
            let obj2 = create_test_object();
            let obj3 = create_test_object();

            // Register as roots
            register_root((obj1 as *mut u8).add(16));
            register_root((obj2 as *mut u8).add(16));

            // Mark obj3 as potential cycle (not rooted)
            register_potential_cycle(obj3);

            // Collect
            force_collect();

            // Clean up
            unregister_root((obj1 as *mut u8).add(16));
            unregister_root((obj2 as *mut u8).add(16));

            free_test_object(obj1);
            free_test_object(obj2);
            free_test_object(obj3);
        }
    }

    #[test]
    fn test_maybe_collect_threshold() {
        init();

        let initial = stats().collections_run;

        // Trigger threshold
        for _ in 0..1000 {
            maybe_collect();
        }

        let final_count = stats().collections_run;
        assert!(final_count >= initial);
    }

    #[test]
    fn test_refcount_with_roots() {
        init();

        unsafe {
            let header = create_test_object();
            let obj = (header as *mut u8).add(16) as *mut u64;

            // Create RefCount
            let rc = RefCount::new(obj);

            // Register as root
            let _guard = RootGuard::new(obj as *mut u8);

            // Mark as potential cycle
            rc.mark_potential_cycle();

            // Collect
            collect_cycles();

            // Should still be alive (rooted)
            assert_eq!(rc.count(), 1);

            drop(rc);
            free_test_object(header);
        }
    }

    #[test]
    fn test_stress_refcount() {
        init();

        unsafe {
            let header = create_test_object();
            let obj = (header as *mut u8).add(16) as *mut u64;
            let rc = Arc::new(RefCount::new(obj));

            let handles: Vec<_> = (0..16)
                .map(|_| {
                    let rc = Arc::clone(&rc);
                    thread::spawn(move || {
                        for _ in 0..1000 {
                            let _clone = rc.clone();
                            thread::yield_now();
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }

            assert_eq!(rc.count(), 1);

            drop(rc);
            free_test_object(header);
        }
    }

    #[test]
    fn test_gc_stats_monotonic() {
        init();

        let stats1 = stats();

        force_collect();
        let stats2 = stats();
        assert!(stats2.collections_run >= stats1.collections_run);

        force_collect();
        let stats3 = stats();
        assert!(stats3.collections_run >= stats2.collections_run);
    }

    #[test]
    fn test_refcount_never_negative() {
        unsafe {
            let header = create_test_object();
            let obj = (header as *mut u8).add(16) as *mut u64;

            let rc = RefCount::new(obj);
            for _ in 0..100 {
                rc.inc();
            }

            for _ in 0..100 {
                rc.dec();
            }

            assert_eq!(rc.count(), 1);

            drop(rc);
            free_test_object(header);
        }
    }
}
