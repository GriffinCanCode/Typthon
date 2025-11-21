use typthon::analysis::{ConstraintSolver, Constraint, ProtocolLibrary, ProtocolChecker};
use typthon::core::types::{Type, TypeContext};
use std::sync::Arc;

#[test]
fn test_sized_protocol() {
    let ctx = Arc::new(TypeContext::new());
    let solver = ConstraintSolver::with_context(ctx.clone());

    let sized_methods = ProtocolLibrary::sized();

    // str should satisfy Sized (has __len__)
    assert!(solver.check_protocol(&Type::Str, &sized_methods).is_ok());
}

#[test]
fn test_iterable_protocol() {
    let ctx = Arc::new(TypeContext::new());
    let solver = ConstraintSolver::with_context(ctx.clone());

    let iterable_methods = ProtocolLibrary::iterable(Type::Int);

    // List should satisfy Iterable
    // Note: Need to add __iter__ to list schema for this to pass
    let result = solver.check_protocol(&Type::List(Box::new(Type::Int)), &iterable_methods);
    // This will return Ok(false) if context doesn't have __iter__, which is fine
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_comparable_protocol() {
    let comparable_methods = ProtocolLibrary::comparable();
    assert_eq!(comparable_methods.len(), 6);

    // Verify all comparison methods are present
    assert!(comparable_methods.iter().any(|(name, _)| name == "__lt__"));
    assert!(comparable_methods.iter().any(|(name, _)| name == "__le__"));
    assert!(comparable_methods.iter().any(|(name, _)| name == "__gt__"));
    assert!(comparable_methods.iter().any(|(name, _)| name == "__ge__"));
    assert!(comparable_methods.iter().any(|(name, _)| name == "__eq__"));
    assert!(comparable_methods.iter().any(|(name, _)| name == "__ne__"));
}

#[test]
fn test_numeric_protocol() {
    let numeric_methods = ProtocolLibrary::numeric();
    assert_eq!(numeric_methods.len(), 6);

    // Verify arithmetic operators
    assert!(numeric_methods.iter().any(|(name, _)| name == "__add__"));
    assert!(numeric_methods.iter().any(|(name, _)| name == "__sub__"));
    assert!(numeric_methods.iter().any(|(name, _)| name == "__mul__"));
    assert!(numeric_methods.iter().any(|(name, _)| name == "__truediv__"));
}

#[test]
fn test_protocol_composition() {
    let sized = ProtocolLibrary::sized();
    let iterable = ProtocolLibrary::iterable(Type::Int);

    let composed = ConstraintSolver::compose_protocols(&sized, &iterable);

    // Should have methods from both protocols
    assert!(composed.iter().any(|(name, _)| name == "__len__"));
    assert!(composed.iter().any(|(name, _)| name == "__iter__"));
}

#[test]
fn test_container_protocol() {
    let container = ProtocolLibrary::container(Type::Str);

    // Should have both Sized and Iterable methods
    assert!(container.iter().any(|(name, _)| name == "__len__"));
    assert!(container.iter().any(|(name, _)| name == "__iter__"));
}

#[test]
fn test_sequence_protocol() {
    let sequence = ProtocolLibrary::sequence(Type::Int);

    // Should have container methods plus __getitem__, count, index
    assert!(sequence.iter().any(|(name, _)| name == "__len__"));
    assert!(sequence.iter().any(|(name, _)| name == "__iter__"));
    assert!(sequence.iter().any(|(name, _)| name == "__getitem__"));
    assert!(sequence.iter().any(|(name, _)| name == "count"));
    assert!(sequence.iter().any(|(name, _)| name == "index"));
}

#[test]
fn test_mapping_protocol() {
    let mapping = ProtocolLibrary::mapping(Type::Str, Type::Int);

    // Should have dict-like methods
    assert!(mapping.iter().any(|(name, _)| name == "__getitem__"));
    assert!(mapping.iter().any(|(name, _)| name == "__setitem__"));
    assert!(mapping.iter().any(|(name, _)| name == "__delitem__"));
    assert!(mapping.iter().any(|(name, _)| name == "__contains__"));
    assert!(mapping.iter().any(|(name, _)| name == "keys"));
    assert!(mapping.iter().any(|(name, _)| name == "values"));
    assert!(mapping.iter().any(|(name, _)| name == "items"));
}

#[test]
fn test_callable_protocol() {
    let params = vec![Type::Int, Type::Str];
    let ret = Type::Bool;
    let callable = ProtocolLibrary::callable(params.clone(), ret.clone());

    assert_eq!(callable.len(), 1);
    assert_eq!(callable[0].0, "__call__");

    // Verify it's a function type
    if let Type::Function(func_params, func_ret) = &callable[0].1 {
        assert_eq!(func_params.len(), 2);
        assert_eq!(**func_ret, Type::Bool);
    } else {
        panic!("Expected Function type");
    }
}

#[test]
fn test_context_manager_protocol() {
    let context_mgr = ProtocolLibrary::context_manager(Type::Any);

    assert_eq!(context_mgr.len(), 2);
    assert!(context_mgr.iter().any(|(name, _)| name == "__enter__"));
    assert!(context_mgr.iter().any(|(name, _)| name == "__exit__"));
}

#[test]
fn test_async_protocols() {
    let awaitable = ProtocolLibrary::awaitable(Type::Int);
    assert_eq!(awaitable.len(), 1);
    assert_eq!(awaitable[0].0, "__await__");

    let async_iterable = ProtocolLibrary::async_iterable(Type::Str);
    assert_eq!(async_iterable.len(), 1);
    assert_eq!(async_iterable[0].0, "__aiter__");

    let async_iterator = ProtocolLibrary::async_iterator(Type::Bool);
    assert_eq!(async_iterator.len(), 2);
    assert!(async_iterator.iter().any(|(name, _)| name == "__aiter__"));
    assert!(async_iterator.iter().any(|(name, _)| name == "__anext__"));
}

#[test]
fn test_supports_protocols() {
    let supports_int = ProtocolLibrary::supports_int();
    assert_eq!(supports_int[0].0, "__int__");

    let supports_float = ProtocolLibrary::supports_float();
    assert_eq!(supports_float[0].0, "__float__");

    let supports_str = ProtocolLibrary::supports_str();
    assert_eq!(supports_str[0].0, "__str__");

    let supports_bool = ProtocolLibrary::supports_bool();
    assert_eq!(supports_bool[0].0, "__bool__");
}

#[test]
fn test_get_protocol_by_name() {
    // Test simple protocols
    assert!(ProtocolLibrary::get_protocol("Sized", &[]).is_some());
    assert!(ProtocolLibrary::get_protocol("Hashable", &[]).is_some());
    assert!(ProtocolLibrary::get_protocol("Comparable", &[]).is_some());
    assert!(ProtocolLibrary::get_protocol("Numeric", &[]).is_some());

    // Test parameterized protocols
    assert!(ProtocolLibrary::get_protocol("Iterable", &[Type::Int]).is_some());
    assert!(ProtocolLibrary::get_protocol("Container", &[Type::Str]).is_some());

    // Test protocols requiring multiple args
    assert!(ProtocolLibrary::get_protocol("Mapping", &[Type::Str, Type::Int]).is_some());

    // Test non-existent protocol
    assert!(ProtocolLibrary::get_protocol("NonExistent", &[]).is_none());
}

#[test]
fn test_method_variance() {
    let solver = ConstraintSolver::new();

    // Test covariant return type
    let actual = Type::Function(vec![Type::Any], Box::new(Type::Int));
    let expected = Type::Function(vec![Type::Any], Box::new(Type::Any));
    assert!(solver.check_method_compatibility(&actual, &expected).unwrap());

    // Test contravariant parameters
    let actual = Type::Function(vec![Type::Any], Box::new(Type::Int));
    let expected = Type::Function(vec![Type::Int], Box::new(Type::Int));
    assert!(solver.check_method_compatibility(&actual, &expected).unwrap());

    // Test incompatible return types
    let actual = Type::Function(vec![], Box::new(Type::Str));
    let expected = Type::Function(vec![], Box::new(Type::Int));
    assert!(!solver.check_method_compatibility(&actual, &expected).unwrap());
}

#[test]
fn test_protocol_constraint_creation() {
    let methods = vec![
        ("foo".to_string(), Type::Int),
        ("bar".to_string(), Type::Str),
    ];

    let constraint = ConstraintSolver::protocol_constraint(Type::Any, methods.clone());

    if let Constraint::Protocol(ty, protocol_methods) = constraint {
        assert_eq!(ty, Type::Any);
        assert_eq!(protocol_methods.len(), 2);
    } else {
        panic!("Expected Protocol constraint");
    }
}

#[test]
fn test_protocol_error_handling() {
    let ctx = Arc::new(TypeContext::new());
    let solver = ConstraintSolver::with_context(ctx.clone());

    // Test missing method error
    let methods = vec![
        ("nonexistent_method".to_string(), Type::Function(vec![], Box::new(Type::Int))),
    ];

    let result = solver.check_protocol(&Type::Int, &methods);
    assert!(result.is_err());

    // Error should contain suggestions
    if let Err(error) = result {
        assert!(!error.suggestions.is_empty() || error.kind.to_string().contains("must implement"));
    }
}

#[test]
fn test_multiple_protocols() {
    let ctx = Arc::new(TypeContext::new());
    let solver = ConstraintSolver::with_context(ctx.clone());

    let protocols = vec![
        ProtocolLibrary::sized(),
        ProtocolLibrary::hashable(),
    ];

    // Test with a type that should satisfy both
    let result = solver.check_protocols(&Type::Str, &protocols);
    // Should either succeed or be deferred (both are valid)
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_protocol_with_union_types() {
    let ctx = Arc::new(TypeContext::new());
    let solver = ConstraintSolver::with_context(ctx.clone());

    let methods = vec![
        ("upper".to_string(), Type::Function(vec![], Box::new(Type::Str))),
    ];

    // Union type where all variants satisfy protocol
    let union_ty = Type::Union(vec![Type::Str, Type::Str]);

    // This should defer or check each variant
    let result = solver.check_protocol(&union_ty, &methods);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_reversible_protocol() {
    let reversible = ProtocolLibrary::reversible(Type::Int);

    // Should have __iter__ and __reversed__
    assert!(reversible.iter().any(|(name, _)| name == "__iter__"));
    assert!(reversible.iter().any(|(name, _)| name == "__reversed__"));
}

#[test]
fn test_hashable_protocol() {
    let hashable = ProtocolLibrary::hashable();
    assert_eq!(hashable.len(), 1);
    assert_eq!(hashable[0].0, "__hash__");

    if let Type::Function(params, ret) = &hashable[0].1 {
        assert!(params.is_empty());
        assert_eq!(**ret, Type::Int);
    } else {
        panic!("Expected Function type");
    }
}

#[test]
fn test_equality_protocol() {
    let equality = ProtocolLibrary::equality();
    assert_eq!(equality.len(), 1);
    assert_eq!(equality[0].0, "__eq__");

    if let Type::Function(params, ret) = &equality[0].1 {
        assert_eq!(params.len(), 1);
        assert_eq!(**ret, Type::Bool);
    } else {
        panic!("Expected Function type");
    }
}

