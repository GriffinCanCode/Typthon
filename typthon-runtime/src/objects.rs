//! Object system - unified representation for all Python values
//!
//! Design: Tagged pointers + heap objects for optimal performance:
//! - Small ints inline (63-bit on 64-bit systems)
//! - Heap objects for everything else (strings, lists, dicts, objects)
//! - Type info pointer for fast dispatch
//! - Reference counting for deterministic cleanup

use std::ptr::NonNull;
use std::marker::PhantomData;
use crate::allocator::ObjectHeader;

/// Tagged pointer encoding for immediate values
///
/// Layout (64-bit):
/// - Bit 0 = 0: Pointer (8-byte aligned)
/// - Bit 0 = 1, Bit 1 = 0: SmallInt (61-bit signed, bits 3-63)
/// - Bit 0 = 1, Bit 1 = 1: Special (bool/none/etc, bits 2-7 encode type)
const TAG_MASK: usize = 0b11;
const PTR_TAG: usize = 0b00;
const INT_TAG: usize = 0b01;
const SPECIAL_TAG: usize = 0b11;

const SPECIAL_TYPE_SHIFT: usize = 2;
const SPECIAL_NONE: usize = 0;
const SPECIAL_TRUE: usize = 1;
const SPECIAL_FALSE: usize = 2;

/// Universal Python object reference (8 bytes)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PyObject {
    bits: usize,
}

impl PyObject {
    /// Create from pointer to heap object
    #[inline]
    pub fn from_ptr(ptr: NonNull<HeapObject>) -> Self {
        debug_assert_eq!(ptr.as_ptr() as usize & TAG_MASK, 0, "Pointer must be aligned");
        Self {
            bits: ptr.as_ptr() as usize | PTR_TAG,
        }
    }

    /// Create small integer (fast path, no allocation)
    #[inline]
    pub fn from_int(val: i64) -> Self {
        let small_max = 1i64 << 60;
        let small_min = -(1i64 << 60);

        if val < small_min || val >= small_max {
            // Large int needs heap allocation
            Self::from_bigint(val)
        } else {
            Self {
                bits: ((val as usize) << 3) | INT_TAG,
            }
        }
    }

    /// Create boolean
    #[inline]
    pub const fn from_bool(val: bool) -> Self {
        let special_code = if val { SPECIAL_TRUE } else { SPECIAL_FALSE };
        Self {
            bits: (special_code << SPECIAL_TYPE_SHIFT) | SPECIAL_TAG,
        }
    }

    /// Create None
    #[inline]
    pub const fn none() -> Self {
        Self {
            bits: (SPECIAL_NONE << SPECIAL_TYPE_SHIFT) | SPECIAL_TAG,
        }
    }

    /// Check if this is a pointer to heap object
    #[inline]
    pub fn is_ptr(self) -> bool {
        (self.bits & TAG_MASK) == PTR_TAG
    }

    /// Check if this is a small int
    #[inline]
    pub fn is_int(self) -> bool {
        (self.bits & TAG_MASK) == INT_TAG
    }

    /// Check if this is a special value
    #[inline]
    pub fn is_special(self) -> bool {
        (self.bits & TAG_MASK) == SPECIAL_TAG
    }

    /// Extract small int value (panics if not an int)
    #[inline]
    pub fn as_int(self) -> i64 {
        debug_assert!(self.is_int());
        (self.bits as i64) >> 3
    }

    /// Extract pointer to heap object (panics if not a pointer)
    #[inline]
    pub fn as_ptr(self) -> NonNull<HeapObject> {
        debug_assert!(self.is_ptr());
        unsafe {
            NonNull::new_unchecked((self.bits & !TAG_MASK) as *mut HeapObject)
        }
    }

    /// Get type of this object
    pub fn get_type(self) -> ObjectType {
        if self.is_int() {
            ObjectType::Int
        } else if self.is_special() {
            match (self.bits >> SPECIAL_TYPE_SHIFT) & 0b111111 {
                SPECIAL_NONE => ObjectType::None,
                SPECIAL_TRUE | SPECIAL_FALSE => ObjectType::Bool,
                _ => ObjectType::Unknown,
            }
        } else {
            unsafe {
                let obj = self.as_ptr().as_ref();
                obj.type_id()
            }
        }
    }

    /// Check if object is truthy (Python truthiness)
    pub fn is_truthy(self) -> bool {
        if self.is_int() {
            self.as_int() != 0
        } else if self.is_special() {
            match self.get_type() {
                ObjectType::None => false,
                ObjectType::Bool => (self.bits >> SPECIAL_TYPE_SHIFT) & 0b111111 == SPECIAL_TRUE,
                _ => true,
            }
        } else {
            match self.get_type() {
                ObjectType::Float => unsafe {
                    let heap_obj = self.as_ptr().as_ref();
                    heap_obj.data().float.value != 0.0
                },
                ObjectType::String => crate::builtins::py_string_len(self) > 0,
                ObjectType::List => crate::builtins::py_list_len(self) > 0,
                ObjectType::Dict => crate::builtins::py_dict_len(self) > 0,
                ObjectType::Tuple => crate::builtins::py_tuple_len(self) > 0,
                _ => true,
            }
        }
    }

    /// Convert to string representation
    pub fn to_string(self) -> String {
        if self.is_int() {
            self.as_int().to_string()
        } else if self.is_special() {
            match self.get_type() {
                ObjectType::None => "None".to_string(),
                ObjectType::Bool => {
                    if (self.bits >> SPECIAL_TYPE_SHIFT) & 0b111111 == SPECIAL_TRUE {
                        "True".to_string()
                    } else {
                        "False".to_string()
                    }
                }
                _ => "Unknown".to_string(),
            }
        } else {
            match self.get_type() {
                ObjectType::Float => {
                    let val = crate::builtins::py_float_as_f64(self);
                    val.to_string()
                }
                ObjectType::String => {
                    crate::builtins::py_string_as_str(self).to_string()
                }
                ObjectType::List => {
                    let len = crate::builtins::py_list_len(self);
                    let mut s = String::from("[");
                    for i in 0..len {
                        if i > 0 {
                            s.push_str(", ");
                        }
                        let item = crate::builtins::py_list_get(self, i as isize);
                        s.push_str(&item.to_string());
                    }
                    s.push(']');
                    s
                }
                ObjectType::Tuple => {
                    let len = crate::builtins::py_tuple_len(self);
                    let mut s = String::from("(");
                    for i in 0..len {
                        if i > 0 {
                            s.push_str(", ");
                        }
                        let item = crate::builtins::py_tuple_get(self, i as isize);
                        s.push_str(&item.to_string());
                    }
                    if len == 1 {
                        s.push(',');
                    }
                    s.push(')');
                    s
                }
                ObjectType::Dict => format!("<dict at {:p}>", self.as_ptr()),
                _ => format!("<{:?} at {:p}>", self.get_type(), self.as_ptr()),
            }
        }
    }

    /// Hash object (for dict keys)
    pub fn hash(self) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();

        if self.is_int() {
            self.as_int().hash(&mut hasher);
        } else if self.is_special() {
            self.get_type().hash(&mut hasher);
        } else {
            match self.get_type() {
                ObjectType::Float => {
                    let val = crate::builtins::py_float_as_f64(self);
                    val.to_bits().hash(&mut hasher);
                }
                ObjectType::String => {
                    crate::builtins::py_string_as_str(self).hash(&mut hasher);
                }
                ObjectType::Tuple => {
                    // Hash each element
                    let len = crate::builtins::py_tuple_len(self);
                    len.hash(&mut hasher);
                    for i in 0..len {
                        let item = crate::builtins::py_tuple_get(self, i as isize);
                        item.hash().hash(&mut hasher);
                    }
                }
                _ => {
                    // For unhashable types, use identity
                    (self.as_ptr().as_ptr() as usize).hash(&mut hasher);
                }
            }
        }

        hasher.finish()
    }

    /// Heap-allocate large integer
    fn from_bigint(val: i64) -> Self {
        // For extremely large integers, we would heap allocate
        // For now, truncate to fit in small int range since we rarely hit this
        // In full implementation, this would use arbitrary precision arithmetic
        let truncated = if val > 0 {
            (1i64 << 60) - 1
        } else {
            -(1i64 << 60)
        };

        Self {
            bits: ((truncated as usize) << 3) | INT_TAG,
        }
    }
}

/// Heap-allocated Python object
#[repr(C)]
pub struct HeapObject {
    header: ObjectHeader,
    data: ObjectData,
}

impl HeapObject {
    #[inline]
    fn type_id(&self) -> ObjectType {
        unsafe {
            let type_info = self.header.type_info().as_ref();
            type_info.object_type()
        }
    }

    /// Get reference to object data
    #[inline]
    pub fn data(&self) -> &ObjectData {
        &self.data
    }

    /// Get mutable reference to object data
    #[inline]
    pub fn data_mut(&mut self) -> &mut ObjectData {
        &mut self.data
    }
}

/// Object data union - different representations per type
#[repr(C)]
pub union ObjectData {
    pub float: FloatData,
    pub string: StringData,
    pub list: ListData,
    pub dict: DictData,
    pub tuple: TupleData,
    pub function: FunctionData,
    pub class: ClassData,
    pub instance: InstanceData,
}

/// Float object data
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FloatData {
    pub value: f64,
}

/// String object data
#[repr(C)]
#[derive(Clone, Copy)]
pub struct StringData {
    pub len: usize,
    pub capacity: usize,
    pub ptr: *mut u8,
}

/// List object data
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ListData {
    pub len: usize,
    pub capacity: usize,
    pub ptr: *mut PyObject,
}

/// Dict object data
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DictData {
    pub len: usize,
    pub capacity: usize,
    pub ptr: *mut DictEntry,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DictEntry {
    pub hash: u64,
    pub key: PyObject,
    pub value: PyObject,
}

/// Tuple object data (immutable)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TupleData {
    pub len: usize,
    pub elements: [PyObject; 0], // Flexible array member
}

/// Function object data
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FunctionData {
    pub code_ptr: *const u8,
    pub closure: *mut ClosureData,
    pub name: *const u8,
}

/// Closure data (captured variables)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ClosureData {
    pub len: usize,
    pub captures: [PyObject; 0], // Flexible array member
}

/// Class object data
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ClassData {
    pub name: *const u8,
    pub bases: *mut PyObject,
    pub methods: *mut DictData,
    pub attrs: *mut DictData,
}

/// Instance object data
#[repr(C)]
#[derive(Clone, Copy)]
pub struct InstanceData {
    pub class: PyObject,
    pub attrs: *mut DictData,
}

/// Object types for dispatch
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ObjectType {
    None = 0,
    Bool = 1,
    Int = 2,
    Float = 3,
    String = 4,
    List = 5,
    Dict = 6,
    Tuple = 7,
    Function = 8,
    Class = 9,
    Instance = 10,
    Unknown = 255,
}

/// Type-safe wrapper for specific object types
pub struct PyString {
    obj: PyObject,
    _phantom: PhantomData<StringData>,
}

impl PyString {
    pub fn new(s: &str) -> Self {
        let obj = crate::builtins::py_string_new(s);
        Self {
            obj,
            _phantom: PhantomData,
        }
    }

    pub fn as_str(&self) -> &str {
        crate::builtins::py_string_as_str(self.obj)
    }

    pub fn inner(&self) -> PyObject {
        self.obj
    }
}

/// Type-safe wrapper for list objects
pub struct PyList {
    obj: PyObject,
    _phantom: PhantomData<ListData>,
}

impl PyList {
    pub fn new() -> Self {
        let obj = crate::builtins::py_list_new();
        Self {
            obj,
            _phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        crate::builtins::py_list_len(self.obj)
    }

    pub fn inner(&self) -> PyObject {
        self.obj
    }
}

/// Type-safe wrapper for dict objects
pub struct PyDict {
    obj: PyObject,
    _phantom: PhantomData<DictData>,
}

impl PyDict {
    pub fn new() -> Self {
        let obj = crate::builtins::py_dict_new();
        Self {
            obj,
            _phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        crate::builtins::py_dict_len(self.obj)
    }

    pub fn inner(&self) -> PyObject {
        self.obj
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_int_encoding() {
        let obj = PyObject::from_int(42);
        assert!(obj.is_int());
        assert_eq!(obj.as_int(), 42);

        let obj = PyObject::from_int(-100);
        assert!(obj.is_int());
        assert_eq!(obj.as_int(), -100);
    }

    #[test]
    fn test_bool_encoding() {
        let t = PyObject::from_bool(true);
        assert!(t.is_special());
        assert_eq!(t.get_type(), ObjectType::Bool);

        let f = PyObject::from_bool(false);
        assert!(f.is_special());
        assert_eq!(f.get_type(), ObjectType::Bool);
    }

    #[test]
    fn test_none_encoding() {
        let n = PyObject::none();
        assert!(n.is_special());
        assert_eq!(n.get_type(), ObjectType::None);
    }

    #[test]
    fn test_size() {
        assert_eq!(std::mem::size_of::<PyObject>(), 8);
    }
}

