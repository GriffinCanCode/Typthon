#[repr(C)]
pub struct CTypeSet {
    inner: *mut std::ffi::c_void,
}

extern "C" {
    pub fn typeset_new() -> *mut CTypeSet;
    pub fn typeset_free(set: *mut CTypeSet);
    pub fn typeset_insert(set: *mut CTypeSet, id: u64);
    pub fn typeset_contains(set: *const CTypeSet, id: u64) -> bool;
    pub fn typeset_union(a: *const CTypeSet, b: *const CTypeSet) -> *mut CTypeSet;
    pub fn typeset_intersection(a: *const CTypeSet, b: *const CTypeSet) -> *mut CTypeSet;
    pub fn typeset_is_subset(a: *const CTypeSet, b: *const CTypeSet) -> bool;
    pub fn typeset_cardinality(set: *const CTypeSet) -> usize;

    pub fn type_is_subtype(a: u64, b: u64) -> bool;
    pub fn type_meet(a: u64, b: u64) -> u64;
    pub fn type_join(a: u64, b: u64) -> u64;

    // Bulk operations
    pub fn typeset_from_array(ids: *const u64, count: usize) -> *mut CTypeSet;
    pub fn typeset_to_array(set: *const CTypeSet, ids: *mut u64, capacity: usize) -> usize;
    pub fn typeset_union_inplace(set: *mut CTypeSet, other: *const CTypeSet);
    pub fn typeset_intersect_inplace(set: *mut CTypeSet, other: *const CTypeSet);
    pub fn typeset_union_many(sets: *const *const CTypeSet, count: usize) -> *mut CTypeSet;
    pub fn typeset_intersection_many(sets: *const *const CTypeSet, count: usize) -> *mut CTypeSet;
}

pub struct TypeSet {
    ptr: *mut CTypeSet,
}

impl TypeSet {
    pub fn new() -> Self {
        Self {
            ptr: unsafe { typeset_new() },
        }
    }

    /// Create TypeSet from array of TypeIds (bulk operation)
    pub fn from_ids(ids: &[u64]) -> Self {
        Self {
            ptr: unsafe { typeset_from_array(ids.as_ptr(), ids.len()) },
        }
    }

    /// Convert TypeSet to Vec<TypeId>
    pub fn to_ids(&self) -> Vec<u64> {
        let len = self.len();
        let mut ids = vec![0u64; len];
        let written = unsafe { typeset_to_array(self.ptr, ids.as_mut_ptr(), len) };
        ids.truncate(written);
        ids
    }

    pub fn insert(&mut self, id: u64) {
        unsafe { typeset_insert(self.ptr, id) }
    }

    pub fn contains(&self, id: u64) -> bool {
        unsafe { typeset_contains(self.ptr, id) }
    }

    pub fn union(&self, other: &TypeSet) -> TypeSet {
        TypeSet {
            ptr: unsafe { typeset_union(self.ptr, other.ptr) },
        }
    }

    /// In-place union (faster than allocating new set)
    pub fn union_inplace(&mut self, other: &TypeSet) {
        unsafe { typeset_union_inplace(self.ptr, other.ptr) }
    }

    /// Multi-way union (SIMD-optimized for 3+ sets)
    pub fn union_many(sets: &[&TypeSet]) -> TypeSet {
        if sets.is_empty() {
            return Self::new();
        }

        let ptrs: Vec<*const CTypeSet> = sets.iter().map(|s| s.ptr as *const _).collect();
        TypeSet {
            ptr: unsafe { typeset_union_many(ptrs.as_ptr(), ptrs.len()) },
        }
    }

    pub fn intersection(&self, other: &TypeSet) -> TypeSet {
        TypeSet {
            ptr: unsafe { typeset_intersection(self.ptr, other.ptr) },
        }
    }

    /// In-place intersection
    pub fn intersect_inplace(&mut self, other: &TypeSet) {
        unsafe { typeset_intersect_inplace(self.ptr, other.ptr) }
    }

    /// Multi-way intersection (SIMD-optimized for 3+ sets)
    pub fn intersection_many(sets: &[&TypeSet]) -> TypeSet {
        if sets.is_empty() {
            return Self::new();
        }

        let ptrs: Vec<*const CTypeSet> = sets.iter().map(|s| s.ptr as *const _).collect();
        TypeSet {
            ptr: unsafe { typeset_intersection_many(ptrs.as_ptr(), ptrs.len()) },
        }
    }

    pub fn is_subset_of(&self, other: &TypeSet) -> bool {
        unsafe { typeset_is_subset(self.ptr, other.ptr) }
    }

    pub fn len(&self) -> usize {
        unsafe { typeset_cardinality(self.ptr) }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Drop for TypeSet {
    fn drop(&mut self) {
        unsafe { typeset_free(self.ptr) }
    }
}

impl Default for TypeSet {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for TypeSet {}
unsafe impl Sync for TypeSet {}

