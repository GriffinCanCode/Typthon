use crate::core::types::{Type, TypeParam, TypeKind, TypeCondition, DependentConstraint};
use std::collections::HashMap;

/// Advanced type analysis for recursive, conditional, and higher-kinded types
pub struct AdvancedTypeAnalyzer {
    /// Recursive type definitions
    recursive_types: HashMap<String, Type>,

    /// Higher-kinded type constructors
    type_constructors: HashMap<String, Vec<TypeParam>>,

    /// Cache for unfolded recursive types (for occurs check)
    unfold_cache: HashMap<String, Type>,
}

impl AdvancedTypeAnalyzer {
    pub fn new() -> Self {
        Self {
            recursive_types: HashMap::new(),
            type_constructors: HashMap::new(),
            unfold_cache: HashMap::new(),
        }
    }

    /// Register a recursive type definition
    pub fn define_recursive(&mut self, name: String, body: Type) -> Type {
        // Check for immediate cycles
        if let Type::Recursive(inner_name, _) = &body {
            if inner_name == &name {
                return Type::Never; // Infinite type
            }
        }

        let ty = Type::Recursive(name.clone(), Box::new(body.clone()));
        self.recursive_types.insert(name, ty.clone());
        ty
    }

    /// Unfold a recursive type one level
    pub fn unfold(&mut self, ty: &Type) -> Type {
        match ty {
            Type::Recursive(name, body) => {
                // Check cache
                if let Some(cached) = self.unfold_cache.get(name) {
                    return cached.clone();
                }

                // Substitute self-references with the recursive type
                let unfolded = self.substitute_recursive(body, name, ty);
                self.unfold_cache.insert(name.clone(), unfolded.clone());
                unfolded
            }
            other => other.clone(),
        }
    }

    fn substitute_recursive(&self, ty: &Type, rec_name: &str, rec_ty: &Type) -> Type {
        match ty {
            Type::Class(name) if name == rec_name => rec_ty.clone(),
            Type::List(inner) => Type::List(Box::new(self.substitute_recursive(inner, rec_name, rec_ty))),
            Type::Tuple(elems) => Type::Tuple(
                elems.iter().map(|e| self.substitute_recursive(e, rec_name, rec_ty)).collect()
            ),
            Type::Dict(k, v) => Type::Dict(
                Box::new(self.substitute_recursive(k, rec_name, rec_ty)),
                Box::new(self.substitute_recursive(v, rec_name, rec_ty)),
            ),
            Type::Union(types) => Type::Union(
                types.iter().map(|t| self.substitute_recursive(t, rec_name, rec_ty)).collect()
            ),
            other => other.clone(),
        }
    }

    /// Check if a recursive type is well-formed (productive/guarded)
    pub fn is_productive(&self, ty: &Type) -> bool {
        match ty {
            Type::Recursive(name, body) => {
                // A recursive type is productive if the recursion is guarded by a constructor
                self.has_guard(body, name)
            }
            _ => true,
        }
    }

    fn has_guard(&self, ty: &Type, rec_name: &str) -> bool {
        match ty {
            Type::Class(name) if name == rec_name => false, // Unguarded recursion
            Type::List(_) | Type::Tuple(_) | Type::Dict(_, _) => true, // Guarded by constructor
            Type::Union(types) => types.iter().any(|t| self.has_guard(t, rec_name)),
            _ => true,
        }
    }

    /// Define a higher-kinded type constructor
    pub fn define_type_constructor(&mut self, name: String, params: Vec<TypeParam>) {
        self.type_constructors.insert(name, params);
    }

    /// Apply a type constructor to type arguments
    pub fn apply_constructor(&self, name: &str, args: &[Type]) -> Result<Type, String> {
        let params = self.type_constructors.get(name)
            .ok_or_else(|| format!("Unknown type constructor: {}", name))?;

        if args.len() != params.len() {
            return Err(format!(
                "Type constructor {} expects {} arguments, got {}",
                name, params.len(), args.len()
            ));
        }

        // Check kind compatibility
        for (param, arg) in params.iter().zip(args.iter()) {
            if !self.check_kind_compatibility(&param.kind, arg) {
                return Err(format!(
                    "Kind mismatch: parameter {} has kind {:?}, but argument has wrong kind",
                    param.name, param.kind
                ));
            }
        }

        Ok(Type::HigherKinded(name.to_string(), params.clone()))
    }

    fn check_kind_compatibility(&self, kind: &TypeKind, ty: &Type) -> bool {
        match kind {
            TypeKind::Star => {
                // Concrete type
                !matches!(ty, Type::HigherKinded(_, _))
            }
            TypeKind::Arrow(_, _) => {
                // Type constructor
                matches!(ty, Type::HigherKinded(_, _) | Type::Generic(_, _))
            }
        }
    }

    /// Evaluate a conditional type
    pub fn eval_conditional(&self, condition: &TypeCondition, then_type: &Type, else_type: &Type) -> Type {
        if self.eval_condition(condition) {
            then_type.clone()
        } else {
            else_type.clone()
        }
    }

    fn eval_condition(&self, condition: &TypeCondition) -> bool {
        match condition {
            TypeCondition::Extends(sub, sup) => sub.is_subtype(sup),
            TypeCondition::Equal(a, b) => a == b,
            TypeCondition::HasProperty(ty, prop) => {
                // Check if type has property (simplified)
                match ty {
                    Type::Class(name) => {
                        // Would need class definition lookup
                        false
                    }
                    _ => false,
                }
            }
            TypeCondition::Custom(_) => false,
        }
    }

    /// Create a dependent length type for collections
    pub fn dependent_length(elem_type: Type, length: usize) -> Type {
        Type::Dependent(
            Box::new(Type::List(Box::new(elem_type))),
            DependentConstraint::Length(length),
        )
    }

    /// Create a dependent range type
    pub fn dependent_range(elem_type: Type, min: usize, max: usize) -> Type {
        Type::Dependent(
            Box::new(Type::List(Box::new(elem_type))),
            DependentConstraint::LengthRange(min, max),
        )
    }
}

impl Default for AdvancedTypeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Common recursive type patterns
pub mod recursive {
    use super::*;

    /// JSON type: recursive union of primitives and structures
    pub fn json_type() -> Type {
        // JSON = Null | Bool | Int | Float | Str | List[JSON] | Dict[Str, JSON]
        let json_name = "JSON".to_string();

        let primitives = vec![
            Type::None,
            Type::Bool,
            Type::Int,
            Type::Float,
            Type::Str,
        ];

        let recursive_list = Type::List(Box::new(Type::Class(json_name.clone())));
        let recursive_dict = Type::Dict(
            Box::new(Type::Str),
            Box::new(Type::Class(json_name.clone())),
        );

        let mut all_types = primitives;
        all_types.push(recursive_list);
        all_types.push(recursive_dict);

        Type::Recursive(json_name, Box::new(Type::Union(all_types)))
    }

    /// Linked list type: List[T] = Nil | Cons(T, List[T])
    pub fn linked_list(elem_type: Type) -> Type {
        let list_name = "List".to_string();

        let nil = Type::None;
        let cons = Type::Tuple(vec![
            elem_type,
            Type::Class(list_name.clone()),
        ]);

        Type::Recursive(list_name, Box::new(Type::Union(vec![nil, cons])))
    }

    /// Binary tree type: Tree[T] = Leaf(T) | Node(Tree[T], T, Tree[T])
    pub fn binary_tree(elem_type: Type) -> Type {
        let tree_name = "Tree".to_string();

        let leaf = Type::Tuple(vec![elem_type.clone()]);
        let node = Type::Tuple(vec![
            Type::Class(tree_name.clone()),
            elem_type,
            Type::Class(tree_name.clone()),
        ]);

        Type::Recursive(tree_name, Box::new(Type::Union(vec![leaf, node])))
    }
}

/// Common higher-kinded type patterns
pub mod higher_kinded {
    use super::*;

    /// Functor type class: F[_] with map operation
    pub fn functor() -> Vec<TypeParam> {
        vec![TypeParam {
            name: "F".to_string(),
            kind: TypeKind::Arrow(
                Box::new(TypeKind::Star),
                Box::new(TypeKind::Star),
            ),
        }]
    }

    /// Monad type class: M[_] with flatMap operation
    pub fn monad() -> Vec<TypeParam> {
        vec![TypeParam {
            name: "M".to_string(),
            kind: TypeKind::Arrow(
                Box::new(TypeKind::Star),
                Box::new(TypeKind::Star),
            ),
        }]
    }

    /// Applicative type class: F[_] with pure and apply operations
    pub fn applicative() -> Vec<TypeParam> {
        vec![TypeParam {
            name: "F".to_string(),
            kind: TypeKind::Arrow(
                Box::new(TypeKind::Star),
                Box::new(TypeKind::Star),
            ),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recursive_list() {
        let mut analyzer = AdvancedTypeAnalyzer::new();
        let list_ty = recursive::linked_list(Type::Int);

        assert!(analyzer.is_productive(&list_ty));
    }

    #[test]
    fn test_json_type() {
        let json_ty = recursive::json_type();

        match json_ty {
            Type::Recursive(name, _) => assert_eq!(name, "JSON"),
            _ => panic!("Expected recursive type"),
        }
    }

    #[test]
    fn test_dependent_length() {
        let ty = AdvancedTypeAnalyzer::dependent_length(Type::Int, 5);

        match ty {
            Type::Dependent(_, constraint) => {
                assert_eq!(constraint, DependentConstraint::Length(5));
            }
            _ => panic!("Expected dependent type"),
        }
    }

    #[test]
    fn test_type_constructor() {
        let mut analyzer = AdvancedTypeAnalyzer::new();
        analyzer.define_type_constructor("Functor".to_string(), higher_kinded::functor());

        assert!(analyzer.type_constructors.contains_key("Functor"));
    }
}

