use crate::compiler::types::Type;
use crate::compiler::analysis::constraints::ConstraintSolver;

/// Common protocol definitions for structural typing
pub struct ProtocolLibrary;

impl ProtocolLibrary {
    /// Sized protocol: has __len__ method
    pub fn sized() -> Vec<(String, Type)> {
        vec![
            ("__len__".to_string(), Type::Function(vec![], Box::new(Type::Int))),
        ]
    }

    /// Iterable protocol: has __iter__ method
    pub fn iterable(elem_type: Type) -> Vec<(String, Type)> {
        vec![
            ("__iter__".to_string(), Type::Function(
                vec![],
                Box::new(Type::Generic("Iterator".to_string(), vec![elem_type]))
            )),
        ]
    }

    /// Iterator protocol: has __next__ and __iter__ methods
    pub fn iterator(elem_type: Type) -> Vec<(String, Type)> {
        vec![
            ("__iter__".to_string(), Type::Function(vec![], Box::new(Type::Class("Self".to_string())))),
            ("__next__".to_string(), Type::Function(vec![], Box::new(elem_type))),
        ]
    }

    /// Callable protocol: has __call__ method
    pub fn callable(param_types: Vec<Type>, return_type: Type) -> Vec<(String, Type)> {
        vec![
            ("__call__".to_string(), Type::Function(param_types, Box::new(return_type))),
        ]
    }

    /// Context manager protocol: has __enter__ and __exit__
    pub fn context_manager(resource_type: Type) -> Vec<(String, Type)> {
        vec![
            ("__enter__".to_string(), Type::Function(vec![], Box::new(resource_type))),
            ("__exit__".to_string(), Type::Function(
                vec![Type::Any, Type::Any, Type::Any],
                Box::new(Type::None)
            )),
        ]
    }

    /// Comparable protocol: has comparison operators
    pub fn comparable() -> Vec<(String, Type)> {
        let comparison_type = Type::Function(
            vec![Type::Class("Self".to_string())],
            Box::new(Type::Bool)
        );

        vec![
            ("__lt__".to_string(), comparison_type.clone()),
            ("__le__".to_string(), comparison_type.clone()),
            ("__gt__".to_string(), comparison_type.clone()),
            ("__ge__".to_string(), comparison_type.clone()),
            ("__eq__".to_string(), comparison_type.clone()),
            ("__ne__".to_string(), comparison_type),
        ]
    }

    /// Numeric protocol: has arithmetic operators
    pub fn numeric() -> Vec<(String, Type)> {
        let self_type = Type::Class("Self".to_string());
        let binary_op = Type::Function(vec![self_type.clone()], Box::new(self_type.clone()));

        vec![
            ("__add__".to_string(), binary_op.clone()),
            ("__sub__".to_string(), binary_op.clone()),
            ("__mul__".to_string(), binary_op.clone()),
            ("__truediv__".to_string(), binary_op.clone()),
            ("__floordiv__".to_string(), binary_op.clone()),
            ("__mod__".to_string(), binary_op),
        ]
    }

    /// Hashable protocol: has __hash__ method
    pub fn hashable() -> Vec<(String, Type)> {
        vec![
            ("__hash__".to_string(), Type::Function(vec![], Box::new(Type::Int))),
        ]
    }

    /// Equality protocol: has __eq__ method
    pub fn equality() -> Vec<(String, Type)> {
        vec![
            ("__eq__".to_string(), Type::Function(
                vec![Type::Any],
                Box::new(Type::Bool)
            )),
        ]
    }

    /// Container protocol: combination of Sized and Iterable
    pub fn container(elem_type: Type) -> Vec<(String, Type)> {
        ConstraintSolver::compose_protocols(
            &Self::sized(),
            &Self::iterable(elem_type)
        )
    }

    /// Sequence protocol: indexable container
    pub fn sequence(elem_type: Type) -> Vec<(String, Type)> {
        let mut methods = Self::container(elem_type.clone());
        methods.push((
            "__getitem__".to_string(),
            Type::Function(vec![Type::Int], Box::new(elem_type.clone()))
        ));
        methods.push((
            "count".to_string(),
            Type::Function(vec![elem_type.clone()], Box::new(Type::Int))
        ));
        methods.push((
            "index".to_string(),
            Type::Function(vec![elem_type], Box::new(Type::Int))
        ));
        methods
    }

    /// Mapping protocol: dict-like interface
    pub fn mapping(key_type: Type, value_type: Type) -> Vec<(String, Type)> {
        vec![
            ("__getitem__".to_string(), Type::Function(
                vec![key_type.clone()],
                Box::new(value_type.clone())
            )),
            ("__setitem__".to_string(), Type::Function(
                vec![key_type.clone(), value_type.clone()],
                Box::new(Type::None)
            )),
            ("__delitem__".to_string(), Type::Function(
                vec![key_type.clone()],
                Box::new(Type::None)
            )),
            ("__contains__".to_string(), Type::Function(
                vec![key_type],
                Box::new(Type::Bool)
            )),
            ("keys".to_string(), Type::Function(
                vec![],
                Box::new(Type::Generic("KeysView".to_string(), vec![]))
            )),
            ("values".to_string(), Type::Function(
                vec![],
                Box::new(Type::Generic("ValuesView".to_string(), vec![]))
            )),
            ("items".to_string(), Type::Function(
                vec![],
                Box::new(Type::Generic("ItemsView".to_string(), vec![]))
            )),
        ]
    }

    /// Awaitable protocol: can be awaited in async context
    pub fn awaitable(result_type: Type) -> Vec<(String, Type)> {
        vec![
            ("__await__".to_string(), Type::Function(
                vec![],
                Box::new(Type::Generic("Iterator".to_string(), vec![result_type]))
            )),
        ]
    }

    /// AsyncIterable protocol: async iterator support
    pub fn async_iterable(elem_type: Type) -> Vec<(String, Type)> {
        vec![
            ("__aiter__".to_string(), Type::Function(
                vec![],
                Box::new(Type::Generic("AsyncIterator".to_string(), vec![elem_type]))
            )),
        ]
    }

    /// AsyncIterator protocol: has __anext__ method
    pub fn async_iterator(elem_type: Type) -> Vec<(String, Type)> {
        vec![
            ("__aiter__".to_string(), Type::Function(
                vec![],
                Box::new(Type::Class("Self".to_string()))
            )),
            ("__anext__".to_string(), Type::Function(
                vec![],
                Box::new(elem_type)
            )),
        ]
    }

    /// Reversible protocol: supports reversed()
    pub fn reversible(elem_type: Type) -> Vec<(String, Type)> {
        let mut methods = Self::iterable(elem_type.clone());
        methods.push((
            "__reversed__".to_string(),
            Type::Function(
                vec![],
                Box::new(Type::Generic("Iterator".to_string(), vec![elem_type]))
            )
        ));
        methods
    }

    /// SupportsInt protocol: convertible to int
    pub fn supports_int() -> Vec<(String, Type)> {
        vec![
            ("__int__".to_string(), Type::Function(vec![], Box::new(Type::Int))),
        ]
    }

    /// SupportsFloat protocol: convertible to float
    pub fn supports_float() -> Vec<(String, Type)> {
        vec![
            ("__float__".to_string(), Type::Function(vec![], Box::new(Type::Float))),
        ]
    }

    /// SupportsStr protocol: convertible to string
    pub fn supports_str() -> Vec<(String, Type)> {
        vec![
            ("__str__".to_string(), Type::Function(vec![], Box::new(Type::Str))),
        ]
    }

    /// SupportsRepr protocol: has repr
    pub fn supports_repr() -> Vec<(String, Type)> {
        vec![
            ("__repr__".to_string(), Type::Function(vec![], Box::new(Type::Str))),
        ]
    }

    /// SupportsBool protocol: convertible to bool
    pub fn supports_bool() -> Vec<(String, Type)> {
        vec![
            ("__bool__".to_string(), Type::Function(vec![], Box::new(Type::Bool))),
        ]
    }

    /// SupportsBytes protocol: convertible to bytes
    pub fn supports_bytes() -> Vec<(String, Type)> {
        vec![
            ("__bytes__".to_string(), Type::Function(vec![], Box::new(Type::Bytes))),
        ]
    }

    /// Get protocol by name
    pub fn get_protocol(name: &str, type_args: &[Type]) -> Option<Vec<(String, Type)>> {
        match name {
            "Sized" => Some(Self::sized()),
            "Iterable" => type_args.get(0).map(|t| Self::iterable(t.clone())),
            "Iterator" => type_args.get(0).map(|t| Self::iterator(t.clone())),
            "Container" => type_args.get(0).map(|t| Self::container(t.clone())),
            "Sequence" => type_args.get(0).map(|t| Self::sequence(t.clone())),
            "Mapping" => {
                if type_args.len() >= 2 {
                    Some(Self::mapping(type_args[0].clone(), type_args[1].clone()))
                } else {
                    None
                }
            }
            "Comparable" => Some(Self::comparable()),
            "Numeric" => Some(Self::numeric()),
            "Hashable" => Some(Self::hashable()),
            "Equality" => Some(Self::equality()),
            "Callable" => {
                if type_args.len() >= 2 {
                    let params = type_args[..type_args.len()-1].to_vec();
                    let ret = type_args.last().unwrap().clone();
                    Some(Self::callable(params, ret))
                } else {
                    None
                }
            }
            "ContextManager" => type_args.get(0).map(|t| Self::context_manager(t.clone())),
            "Awaitable" => type_args.get(0).map(|t| Self::awaitable(t.clone())),
            "AsyncIterable" => type_args.get(0).map(|t| Self::async_iterable(t.clone())),
            "AsyncIterator" => type_args.get(0).map(|t| Self::async_iterator(t.clone())),
            "Reversible" => type_args.get(0).map(|t| Self::reversible(t.clone())),
            "SupportsInt" => Some(Self::supports_int()),
            "SupportsFloat" => Some(Self::supports_float()),
            "SupportsStr" => Some(Self::supports_str()),
            "SupportsRepr" => Some(Self::supports_repr()),
            "SupportsBool" => Some(Self::supports_bool()),
            "SupportsBytes" => Some(Self::supports_bytes()),
            _ => None,
        }
    }
}

/// Protocol checker for type compatibility
pub struct ProtocolChecker;

impl ProtocolChecker {
    /// Check if a type implements a named protocol
    pub fn implements_protocol(
        ty: &Type,
        protocol_name: &str,
        type_args: &[Type],
        solver: &ConstraintSolver,
    ) -> bool {
        if let Some(methods) = ProtocolLibrary::get_protocol(protocol_name, type_args) {
            solver.check_protocol(ty, &methods).unwrap_or(false)
        } else {
            false
        }
    }

    /// Get all protocols implemented by a type
    pub fn implemented_protocols(ty: &Type, solver: &ConstraintSolver) -> Vec<String> {
        let protocols = vec![
            "Sized", "Hashable", "Equality", "Comparable", "Numeric",
            "SupportsInt", "SupportsFloat", "SupportsStr", "SupportsRepr",
            "SupportsBool", "SupportsBytes",
        ];

        protocols
            .into_iter()
            .filter(|name| {
                if let Some(methods) = ProtocolLibrary::get_protocol(name, &[]) {
                    solver.check_protocol(ty, &methods).unwrap_or(false)
                } else {
                    false
                }
            })
            .map(String::from)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::compiler::types::TypeContext;

    #[test]
    fn test_sized_protocol() {
        let methods = ProtocolLibrary::sized();
        assert_eq!(methods.len(), 1);
        assert_eq!(methods[0].0, "__len__");
    }

    #[test]
    fn test_iterable_protocol() {
        let methods = ProtocolLibrary::iterable(Type::Int);
        assert_eq!(methods.len(), 1);
        assert_eq!(methods[0].0, "__iter__");
    }

    #[test]
    fn test_comparable_protocol() {
        let methods = ProtocolLibrary::comparable();
        assert_eq!(methods.len(), 6);
        assert!(methods.iter().any(|(name, _)| name == "__lt__"));
    }

    #[test]
    fn test_numeric_protocol() {
        let methods = ProtocolLibrary::numeric();
        assert_eq!(methods.len(), 6);
        assert!(methods.iter().any(|(name, _)| name == "__add__"));
    }

    #[test]
    fn test_protocol_composition() {
        let container = ProtocolLibrary::container(Type::Int);
        // Should have both __len__ (Sized) and __iter__ (Iterable)
        assert!(container.iter().any(|(name, _)| name == "__len__"));
        assert!(container.iter().any(|(name, _)| name == "__iter__"));
    }

    #[test]
    fn test_get_protocol() {
        assert!(ProtocolLibrary::get_protocol("Sized", &[]).is_some());
        assert!(ProtocolLibrary::get_protocol("Iterable", &[Type::Int]).is_some());
        assert!(ProtocolLibrary::get_protocol("NonExistent", &[]).is_none());
    }

    #[test]
    fn test_mapping_protocol() {
        let methods = ProtocolLibrary::mapping(Type::Str, Type::Int);
        assert!(methods.iter().any(|(name, _)| name == "__getitem__"));
        assert!(methods.iter().any(|(name, _)| name == "keys"));
        assert!(methods.iter().any(|(name, _)| name == "values"));
    }

    #[test]
    fn test_context_manager_protocol() {
        let methods = ProtocolLibrary::context_manager(Type::Any);
        assert_eq!(methods.len(), 2);
        assert!(methods.iter().any(|(name, _)| name == "__enter__"));
        assert!(methods.iter().any(|(name, _)| name == "__exit__"));
    }
}

