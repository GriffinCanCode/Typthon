use typthon::{Type, TypeContext};
use typthon::compiler::types::ClassSchema;
use std::sync::Arc;

#[test]
fn test_str_attributes() {
    let ctx = Arc::new(TypeContext::new());

    // Test built-in str methods
    assert!(ctx.has_attribute(&Type::Str, "upper").is_some());
    assert!(ctx.has_attribute(&Type::Str, "lower").is_some());
    assert!(ctx.has_attribute(&Type::Str, "strip").is_some());
    assert!(ctx.has_attribute(&Type::Str, "split").is_some());

    // Test that invalid attributes return None
    assert!(ctx.has_attribute(&Type::Str, "invalid_method").is_none());

    // Verify return types
    let upper_ty = ctx.has_attribute(&Type::Str, "upper").unwrap();
    assert!(matches!(upper_ty, Type::Function(_, _)));
}

#[test]
fn test_list_attributes() {
    let ctx = Arc::new(TypeContext::new());
    let list_ty = Type::List(Box::new(Type::Int));

    // Test list methods
    assert!(ctx.has_attribute(&list_ty, "append").is_some());
    assert!(ctx.has_attribute(&list_ty, "extend").is_some());
    assert!(ctx.has_attribute(&list_ty, "pop").is_some());
    assert!(ctx.has_attribute(&list_ty, "remove").is_some());
    assert!(ctx.has_attribute(&list_ty, "sort").is_some());

    // Invalid attribute
    assert!(ctx.has_attribute(&list_ty, "invalid").is_none());
}

#[test]
fn test_dict_attributes() {
    let ctx = Arc::new(TypeContext::new());
    let dict_ty = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));

    // Test dict methods
    assert!(ctx.has_attribute(&dict_ty, "keys").is_some());
    assert!(ctx.has_attribute(&dict_ty, "values").is_some());
    assert!(ctx.has_attribute(&dict_ty, "items").is_some());
    assert!(ctx.has_attribute(&dict_ty, "get").is_some());

    // Invalid attribute
    assert!(ctx.has_attribute(&dict_ty, "append").is_none());
}

#[test]
fn test_set_attributes() {
    let ctx = Arc::new(TypeContext::new());
    let set_ty = Type::Set(Box::new(Type::Int));

    // Test set methods
    assert!(ctx.has_attribute(&set_ty, "add").is_some());
    assert!(ctx.has_attribute(&set_ty, "remove").is_some());
    assert!(ctx.has_attribute(&set_ty, "union").is_some());
    assert!(ctx.has_attribute(&set_ty, "intersection").is_some());

    // Invalid attribute
    assert!(ctx.has_attribute(&set_ty, "append").is_none());
}

#[test]
fn test_custom_class_attributes() {
    let ctx = Arc::new(TypeContext::new());

    // Register custom class
    let person_schema = ClassSchema::new("Person".to_string());
    person_schema.add_property("name".to_string(), Type::Str);
    person_schema.add_property("age".to_string(), Type::Int);
    person_schema.add_method(
        "greet".to_string(),
        Type::Function(vec![], Box::new(Type::Str))
    );

    ctx.register_class(person_schema);

    let person_ty = Type::Class("Person".to_string());

    // Test attributes exist
    assert!(ctx.has_attribute(&person_ty, "name").is_some());
    assert!(ctx.has_attribute(&person_ty, "age").is_some());
    assert!(ctx.has_attribute(&person_ty, "greet").is_some());

    // Verify types
    let name_ty = ctx.has_attribute(&person_ty, "name").unwrap();
    assert_eq!(name_ty, Type::Str);

    let age_ty = ctx.has_attribute(&person_ty, "age").unwrap();
    assert_eq!(age_ty, Type::Int);

    // Invalid attribute
    assert!(ctx.has_attribute(&person_ty, "invalid").is_none());
}

#[test]
fn test_union_type_attributes() {
    let ctx = Arc::new(TypeContext::new());

    // Union of str and list should only support attributes both have
    let union_ty = Type::Union(vec![
        Type::Str,
        Type::List(Box::new(Type::Str))
    ]);

    // Neither str nor list has a common "append" method
    // str doesn't have append, so union shouldn't either
    assert!(ctx.has_attribute(&union_ty, "append").is_none());

    // Both have specific methods, but not shared
    assert!(ctx.has_attribute(&union_ty, "upper").is_none()); // Only str has this
}

#[test]
fn test_intersection_type_attributes() {
    let ctx = Arc::new(TypeContext::new());

    // Create custom schemas
    let readable = ClassSchema::new("Readable".to_string());
    readable.add_method("read".to_string(), Type::Function(vec![], Box::new(Type::Str)));
    ctx.register_class(readable);

    let writable = ClassSchema::new("Writable".to_string());
    writable.add_method("write".to_string(), Type::Function(vec![Type::Str], Box::new(Type::None)));
    ctx.register_class(writable);

    // Intersection should have both attributes
    let intersection_ty = Type::Intersection(vec![
        Type::Class("Readable".to_string()),
        Type::Class("Writable".to_string())
    ]);

    assert!(ctx.has_attribute(&intersection_ty, "read").is_some());
    assert!(ctx.has_attribute(&intersection_ty, "write").is_some());
}

#[test]
fn test_inheritance_attributes() {
    let ctx = Arc::new(TypeContext::new());

    // Create base class
    let animal = ClassSchema::new("Animal".to_string());
    animal.add_property("name".to_string(), Type::Str);
    animal.add_method("speak".to_string(), Type::Function(vec![], Box::new(Type::Str)));
    ctx.register_class(animal);

    // Create derived class
    let dog = ClassSchema::new("Dog".to_string())
        .with_bases(vec!["Animal".to_string()]);
    dog.add_property("breed".to_string(), Type::Str);
    dog.add_method("fetch".to_string(), Type::Function(vec![], Box::new(Type::Str)));
    ctx.register_class(dog);

    let dog_ty = Type::Class("Dog".to_string());

    // Test derived class has its own attributes
    assert!(ctx.has_attribute(&dog_ty, "breed").is_some());
    assert!(ctx.has_attribute(&dog_ty, "fetch").is_some());

    // Test derived class has base class attributes
    assert!(ctx.has_attribute(&dog_ty, "name").is_some());
    assert!(ctx.has_attribute(&dog_ty, "speak").is_some());
}

#[test]
fn test_refinement_type_attributes() {
    use typthon::Type;
    use typthon::compiler::types::Predicate;

    let ctx = Arc::new(TypeContext::new());

    // Refinement type: str where length > 0
    let refined_str = Type::Refinement(
        Box::new(Type::Str),
        Predicate::True
    );

    // Should have str's attributes
    assert!(ctx.has_attribute(&refined_str, "upper").is_some());
    assert!(ctx.has_attribute(&refined_str, "lower").is_some());
}

#[test]
fn test_get_attributes_for_suggestions() {
    let ctx = Arc::new(TypeContext::new());

    // Get all str attributes
    let str_attrs = ctx.get_attributes(&Type::Str);
    assert!(str_attrs.contains(&"upper".to_string()));
    assert!(str_attrs.contains(&"lower".to_string()));
    assert!(str_attrs.contains(&"split".to_string()));

    // Get all list attributes
    let list_attrs = ctx.get_attributes(&Type::List(Box::new(Type::Int)));
    assert!(list_attrs.contains(&"append".to_string()));
    assert!(list_attrs.contains(&"extend".to_string()));
    assert!(list_attrs.contains(&"pop".to_string()));
}

#[test]
fn test_attribute_type_checking() {
    let ctx = Arc::new(TypeContext::new());

    // Get attribute type and verify it
    let upper_ty = ctx.has_attribute(&Type::Str, "upper").unwrap();

    match upper_ty {
        Type::Function(params, ret) => {
            assert_eq!(params.len(), 0); // upper() takes no args
            assert_eq!(*ret, Type::Str); // returns str
        }
        _ => panic!("Expected function type"),
    }

    // Test list.append type
    let append_ty = ctx.has_attribute(&Type::List(Box::new(Type::Int)), "append").unwrap();

    match append_ty {
        Type::Function(params, ret) => {
            assert_eq!(params.len(), 1); // append(value)
            assert_eq!(*ret, Type::None); // returns None
        }
        _ => panic!("Expected function type"),
    }
}

#[test]
fn test_no_attributes_on_primitives() {
    let ctx = Arc::new(TypeContext::new());

    // Primitives like Int, Float, Bool don't have attributes (for now)
    assert!(ctx.has_attribute(&Type::Int, "invalid").is_none());
    assert!(ctx.has_attribute(&Type::Float, "invalid").is_none());
    assert!(ctx.has_attribute(&Type::Bool, "invalid").is_none());
}

