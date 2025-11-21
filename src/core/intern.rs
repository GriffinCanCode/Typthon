//! Type interning system for efficient SIMD operations
//!
//! Maps Rust Type enums to compact u64 TypeIds for C++ SIMD operations.
//! Uses bidirectional hash maps for O(1) lookups in both directions.

use crate::core::types::{Type, TypeId};
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use once_cell::sync::Lazy;

/// Global type interner for efficient Type ↔ TypeId mapping
static INTERNER: Lazy<TypeInterner> = Lazy::new(TypeInterner::new);

/// Thread-safe type interning system
pub struct TypeInterner {
    /// Type → TypeId mapping
    type_to_id: DashMap<Type, TypeId>,
    /// TypeId → Type mapping
    id_to_type: DashMap<TypeId, Type>,
    /// Next available TypeId
    next_id: AtomicU64,
}

impl TypeInterner {
    fn new() -> Self {
        let interner = Self {
            type_to_id: DashMap::with_capacity(4096),
            id_to_type: DashMap::with_capacity(4096),
            next_id: AtomicU64::new(0),
        };

        // Pre-intern common primitive types for efficiency
        interner.intern(Type::Any);
        interner.intern(Type::Never);
        interner.intern(Type::None);
        interner.intern(Type::Bool);
        interner.intern(Type::Int);
        interner.intern(Type::Float);
        interner.intern(Type::Str);
        interner.intern(Type::Bytes);

        interner
    }

    /// Intern a type and return its TypeId
    pub fn intern(&self, ty: Type) -> TypeId {
        // Fast path: check if already interned
        if let Some(id) = self.type_to_id.get(&ty) {
            return *id;
        }

        // Slow path: allocate new ID
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.type_to_id.insert(ty.clone(), id);
        self.id_to_type.insert(id, ty);
        id
    }

    /// Intern multiple types efficiently
    pub fn intern_many(&self, types: &[Type]) -> Vec<TypeId> {
        types.iter().map(|ty| self.intern(ty.clone())).collect()
    }

    /// Get TypeId for a type (returns None if not interned)
    pub fn get_id(&self, ty: &Type) -> Option<TypeId> {
        self.type_to_id.get(ty).map(|id| *id)
    }

    /// Get Type for a TypeId (returns None if invalid)
    pub fn get_type(&self, id: TypeId) -> Option<Type> {
        self.id_to_type.get(&id).map(|ty| ty.clone())
    }

    /// Check if a type is interned
    pub fn contains(&self, ty: &Type) -> bool {
        self.type_to_id.contains_key(ty)
    }

    /// Get the current number of interned types
    pub fn len(&self) -> usize {
        self.type_to_id.len()
    }

    /// Clear the interner (useful for testing)
    #[cfg(test)]
    pub fn clear(&self) {
        self.type_to_id.clear();
        self.id_to_type.clear();
        self.next_id.store(0, Ordering::SeqCst);
    }
}

/// Global functions for convenient access
pub fn intern(ty: Type) -> TypeId {
    INTERNER.intern(ty)
}

pub fn intern_many(types: &[Type]) -> Vec<TypeId> {
    INTERNER.intern_many(types)
}

pub fn get_type(id: TypeId) -> Option<Type> {
    INTERNER.get_type(id)
}

pub fn get_id(ty: &Type) -> Option<TypeId> {
    INTERNER.get_id(ty)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern_primitives() {
        let id1 = intern(Type::Int);
        let id2 = intern(Type::Int);
        assert_eq!(id1, id2, "Same type should get same ID");

        let ty = get_type(id1).unwrap();
        assert_eq!(ty, Type::Int);
    }

    #[test]
    fn test_intern_many() {
        let types = vec![Type::Int, Type::Str, Type::Bool];
        let ids = intern_many(&types);
        assert_eq!(ids.len(), 3);

        for (ty, id) in types.iter().zip(ids.iter()) {
            assert_eq!(get_type(*id), Some(ty.clone()));
        }
    }

    #[test]
    fn test_complex_types() {
        let list_int = Type::List(Box::new(Type::Int));
        let id = intern(list_int.clone());
        assert_eq!(get_type(id), Some(list_int));
    }
}

