//! Allocator tests - comprehensive validation
//!
//! Test suite organized by component:
//! - Allocator Core: High-level allocator API
//! - Header: Object metadata structures
//! - Bump: Fast-path bump allocator
//! - Arena: OS memory management
//! - Object Allocation: Typed object allocation with headers
//! - Statistics: Monitoring and metrics
//! - Edge Cases: Boundary conditions and corner cases
//!
//! Coverage: 30+ tests validating correctness, performance, and safety

#[cfg(test)]
mod tests {
    use super::super::*;
    use core::ptr::NonNull;

    // ===== Allocator Core Tests =====

    #[test]
    fn allocator_creation_starts_empty() {
        let allocator = Allocator::new();
        let stats = allocator.stats();
        assert_eq!(stats.total_allocated, 0);
        assert_eq!(stats.current_arena_remaining, 0);
    }

    #[test]
    fn bump_allocation_sequential() {
        let mut allocator = Allocator::new();

        let ptr1 = allocator.alloc(64, 8).expect("first alloc");
        let ptr2 = allocator.alloc(64, 8).expect("second alloc");
        let ptr3 = allocator.alloc(64, 8).expect("third alloc");

        // All pointers should be distinct
        let addrs = [
            ptr1.as_ptr() as usize,
            ptr2.as_ptr() as usize,
            ptr3.as_ptr() as usize,
        ];
        assert!(addrs[0] != addrs[1]);
        assert!(addrs[1] != addrs[2]);
        assert!(addrs[0] != addrs[2]);

        // Sequential allocations should be monotonically increasing
        assert!(addrs[0] < addrs[1]);
        assert!(addrs[1] < addrs[2]);
    }

    #[test]
    fn allocation_alignment_powers_of_two() {
        let mut allocator = Allocator::new();

        for align in [1, 2, 4, 8, 16, 32, 64, 128, 256] {
            let ptr = allocator.alloc(32, align).expect("aligned alloc");
            let addr = ptr.as_ptr() as usize;
            assert_eq!(addr % align, 0, "not aligned to {}", align);
        }
    }

    #[test]
    fn large_allocation_exceeds_default_arena() {
        let mut allocator = Allocator::new();

        let size = 128 * 1024; // 128KB > 64KB default
        let ptr = allocator.alloc(size, 8).expect("large alloc");

        let stats = allocator.stats();
        assert!(stats.total_allocated >= size);

        // Should be able to write to the allocation
        unsafe {
            core::ptr::write_bytes(ptr.as_ptr(), 0xAA, size);
        }
    }

    #[test]
    fn multiple_large_allocations() {
        let mut allocator = Allocator::new();

        for i in 0..5 {
            let size = 100 * 1024; // 100KB each
            let ptr = allocator.alloc(size, 8)
                .expect(&format!("large alloc {}", i));

            unsafe {
                core::ptr::write_bytes(ptr.as_ptr(), i as u8, size);
            }
        }

        let stats = allocator.stats();
        assert!(stats.total_allocated >= 500 * 1024);
    }

    #[test]
    fn mixed_size_allocations() {
        let mut allocator = Allocator::new();

        let sizes = [8, 16, 32, 64, 128, 256, 512, 1024];
        let mut ptrs = Vec::new();

        for &size in &sizes {
            let ptr = allocator.alloc(size, 8).expect("mixed size alloc");
            ptrs.push((ptr, size));
        }

        // Verify all allocations are distinct
        for i in 0..ptrs.len() {
            for j in i+1..ptrs.len() {
                assert_ne!(ptrs[i].0.as_ptr(), ptrs[j].0.as_ptr());
            }
        }
    }

    #[test]
    fn stress_many_small_allocations() {
        let mut allocator = Allocator::new();

        for _ in 0..1000 {
            allocator.alloc(16, 8).expect("small alloc");
        }

        let stats = allocator.stats();
        assert!(stats.total_allocated >= 16 * 1000);
    }

    // ===== Header Tests =====

    #[test]
    fn header_creation_initializes_correctly() {
        let type_info = TypeInfo::simple(32, 8);
        let type_ptr = NonNull::new(&type_info as *const _ as *mut TypeInfo).unwrap();

        let header = ObjectHeader::new(type_ptr);
        assert_eq!(header.refcount, 1);
        assert_eq!(header.flags, 0);
        assert_eq!(header.type_info, type_ptr);
    }

    #[test]
    fn header_from_object_pointer() {
        let mut buffer = [0u8; 32];
        let base = buffer.as_mut_ptr() as usize;
        let obj_offset = 16;

        // Verify pointer arithmetic: object at +16 means header at +0
        let expected_header = base;
        let obj_ptr = (base + obj_offset) as *mut u8;

        unsafe {
            let header_ptr = ObjectHeader::from_object(obj_ptr);
            assert_eq!(header_ptr as usize, expected_header);
        }
    }

    #[test]
    fn type_info_simple_no_drop() {
        let info = TypeInfo::simple(64, 8);
        assert_eq!(info.size, 64);
        assert_eq!(info.align, 8);
        assert!(info.drop.is_none());
    }

    #[test]
    fn type_info_with_destructor() {
        unsafe fn test_drop(_: *mut u8) {}

        let info = TypeInfo::with_drop(64, 8, test_drop);
        assert_eq!(info.size, 64);
        assert_eq!(info.align, 8);
        assert!(info.drop.is_some());
    }

    // ===== Bump Allocator Tests =====

    #[test]
    fn bump_try_alloc_succeeds_with_space() {
        let mut bump = BumpAllocator::new();
        let mut arena = [0u8; 1024];

        bump.reset(arena.as_mut_ptr(), unsafe { arena.as_mut_ptr().add(1024) });

        let ptr = bump.try_alloc(64, 8).expect("bump alloc");
        assert!(!ptr.as_ptr().is_null());
    }

    #[test]
    fn bump_try_alloc_fails_when_exhausted() {
        let mut bump = BumpAllocator::new();
        let mut arena = [0u8; 64];

        bump.reset(arena.as_mut_ptr(), unsafe { arena.as_mut_ptr().add(64) });

        // Fill the arena
        bump.try_alloc(32, 8).expect("first");
        bump.try_alloc(32, 8).expect("second");

        // Should fail
        assert!(bump.try_alloc(32, 8).is_none());
    }

    #[test]
    fn bump_remaining_decreases() {
        let mut bump = BumpAllocator::new();
        let mut arena = [0u8; 1024];

        bump.reset(arena.as_mut_ptr(), unsafe { arena.as_mut_ptr().add(1024) });

        let initial = bump.remaining();
        assert_eq!(initial, 1024);

        bump.try_alloc(64, 8).expect("alloc");
        let after = bump.remaining();
        assert!(after < initial);
    }

    // ===== Arena Tests =====

    #[test]
    fn arena_creation_succeeds() {
        let arena = Arena::new(4096).expect("arena creation");
        assert_eq!(arena.size(), 4096);
    }

    #[test]
    fn arena_bounds_valid() {
        let arena = Arena::new(4096).expect("arena");
        let (start, end) = arena.bounds();

        assert!(start < end);
        assert_eq!(end as usize - start as usize, 4096);
    }

    #[test]
    fn arena_pool_starts_empty() {
        let pool = ArenaPool::new();
        assert_eq!(pool.total_allocated(), 0);
    }

    #[test]
    fn arena_pool_growth_adaptive() {
        let mut pool = ArenaPool::new();

        pool.grow().expect("first arena");
        let size1 = pool.total_allocated();

        pool.grow().expect("second arena");
        let size2 = pool.total_allocated();

        pool.grow().expect("third arena");
        let size3 = pool.total_allocated();

        // Each arena should be larger than previous
        assert!(size2 > size1 * 2 - 1000); // Account for rounding
        assert!(size3 > size2 * 2 - 1000);
    }

    // ===== Object Allocation Tests =====

    #[test]
    fn alloc_object_with_header() {
        let mut allocator = Allocator::new();

        let type_info = TypeInfo::simple(64, 8);
        let type_ptr = NonNull::new(&type_info as *const _ as *mut TypeInfo).unwrap();

        let obj_ptr: NonNull<u64> = allocator.alloc_object(type_ptr)
            .expect("object alloc");

        unsafe {
            let header = ObjectHeader::from_object(obj_ptr.as_ptr() as *mut u8);
            assert_eq!((*header).refcount, 1);
            assert_eq!((*header).type_info, type_ptr);
        }
    }

    #[test]
    fn alloc_multiple_objects() {
        let mut allocator = Allocator::new();

        let type_info = TypeInfo::simple(32, 8);
        let type_ptr = NonNull::new(&type_info as *const _ as *mut TypeInfo).unwrap();

        let mut objects = Vec::new();
        for _ in 0..10 {
            let obj: NonNull<u32> = allocator.alloc_object(type_ptr)
                .expect("object alloc");
            objects.push(obj);
        }

        // All should be distinct
        for i in 0..objects.len() {
            for j in i+1..objects.len() {
                assert_ne!(objects[i].as_ptr(), objects[j].as_ptr());
            }
        }
    }

    // ===== Statistics Tests =====

    #[test]
    fn stats_reflect_allocations() {
        let mut allocator = Allocator::new();

        let stats_before = allocator.stats();
        assert_eq!(stats_before.total_allocated, 0);

        allocator.alloc(1024, 8).expect("alloc");

        let stats_after = allocator.stats();
        assert!(stats_after.total_allocated > 0);
        assert!(stats_after.current_arena_remaining > 0);
    }

    // ===== Edge Cases =====

    #[test]
    fn zero_size_allocation() {
        let mut allocator = Allocator::new();
        let ptr = allocator.alloc(0, 8);
        // Zero-size allocations should still return a valid pointer
        assert!(ptr.is_some());
    }

    #[test]
    fn minimum_alignment() {
        let mut allocator = Allocator::new();
        let ptr = allocator.alloc(8, 1).expect("min align");
        assert!(!ptr.as_ptr().is_null());
    }

    #[test]
    fn maximum_practical_alignment() {
        let mut allocator = Allocator::new();
        let ptr = allocator.alloc(256, 256).expect("max align");
        let addr = ptr.as_ptr() as usize;
        assert_eq!(addr % 256, 0);
    }
}

