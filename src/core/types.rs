use std::fmt;
use dashmap::DashMap;
use serde::{Serialize, Deserialize};

pub type TypeId = u64;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    Any,
    Never,
    None,
    Bool,
    Int,
    Float,
    Str,
    Bytes,

    // Composite types
    List(Box<Type>),
    Tuple(Vec<Type>),
    Dict(Box<Type>, Box<Type>),
    Set(Box<Type>),

    // Function types: (params) -> return
    Function(Vec<Type>, Box<Type>),

    // Advanced types (Phase 2)
    Union(Vec<Type>),
    Intersection(Vec<Type>),
    Generic(String, Vec<Type>),

    // Nominal types
    Class(String),

    // Type variables for inference
    Var(u64),

    // Phase 3: Advanced Types
    /// Effect type: tracks side effects (IO, network, mutation)
    Effect(Box<Type>, EffectSet),

    /// Refinement type: type with value-level predicate (e.g., x: Int where x > 0)
    Refinement(Box<Type>, Predicate),

    /// Dependent type: type that depends on value (e.g., Vec[n] for length n)
    Dependent(Box<Type>, DependentConstraint),

    /// Nominal type: opaque wrapper for structural escape hatch
    Nominal(String, Box<Type>),

    /// Conditional type: if-then-else at type level
    Conditional {
        condition: Box<TypeCondition>,
        then_type: Box<Type>,
        else_type: Box<Type>,
    },

    /// Recursive type: self-referential type with fixpoint
    Recursive(String, Box<Type>),

    /// Higher-kinded type: type constructor (e.g., Functor[F])
    HigherKinded(String, Vec<TypeParam>),
}

/// Effect set for tracking side effects
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EffectSet {
    effects: Vec<Effect>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Effect {
    Pure,          // No side effects
    IO,            // File/console I/O
    Network,       // Network operations
    Mutation,      // State mutation
    Exception,     // Can throw exceptions
    Async,         // Async/await
    Random,        // Non-deterministic
    Time,          // Time-dependent
    Custom(String), // User-defined effect
}

/// Predicate for refinement types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Predicate {
    /// True predicate (always satisfied)
    True,

    /// Comparison: x > y, x < y, x == y, etc.
    Compare {
        op: CompareOp,
        left: PredicateExpr,
        right: PredicateExpr,
    },

    /// Logical AND
    And(Vec<Predicate>),

    /// Logical OR
    Or(Vec<Predicate>),

    /// Logical NOT
    Not(Box<Predicate>),

    /// Custom predicate expression
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompareOp {
    Eq, Ne, Lt, Le, Gt, Ge,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PredicateExpr {
    Value,              // The refined value itself
    Literal(i64),       // Constant
    Property(String),   // Property access (e.g., len, abs)
    BinOp(Box<PredicateExpr>, BinOp, Box<PredicateExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
}

/// Dependent type constraint
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependentConstraint {
    /// Length constraint: len(x) == n
    Length(usize),

    /// Range constraint: min <= len(x) <= max
    LengthRange(usize, usize),

    /// Value equals some expression
    ValueEq(String),

    /// Custom constraint
    Custom(String),
}

/// Type-level condition for conditional types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeCondition {
    /// T extends U
    Extends(Type, Type),

    /// T == U
    Equal(Type, Type),

    /// T has property P
    HasProperty(Type, String),

    /// Custom condition
    Custom(String),
}

/// Type parameter for higher-kinded types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeParam {
    pub name: String,
    pub kind: TypeKind,
}

/// Kind of a type (for higher-kinded types)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeKind {
    /// * (concrete type)
    Star,

    /// * -> * (type constructor taking one type)
    Arrow(Box<TypeKind>, Box<TypeKind>),
}

impl EffectSet {
    pub fn empty() -> Self {
        Self { effects: vec![] }
    }

    pub fn pure() -> Self {
        Self { effects: vec![Effect::Pure] }
    }

    pub fn single(effect: Effect) -> Self {
        Self { effects: vec![effect] }
    }

    pub fn union(mut self, other: Self) -> Self {
        for effect in other.effects {
            if !self.effects.contains(&effect) {
                self.effects.push(effect);
            }
        }
        // Remove Pure if other effects present
        if self.effects.len() > 1 {
            self.effects.retain(|e| *e != Effect::Pure);
        }
        self
    }

    pub fn is_pure(&self) -> bool {
        self.effects.is_empty() || self.effects == vec![Effect::Pure]
    }

    pub fn contains(&self, effect: &Effect) -> bool {
        self.effects.contains(effect)
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        self.effects.iter().all(|e| other.contains(e))
    }
}

impl Predicate {
    pub fn and(self, other: Self) -> Self {
        match (self, other) {
            (Predicate::And(mut preds), Predicate::And(others)) => {
                preds.extend(others);
                Predicate::And(preds)
            }
            (Predicate::And(mut preds), other) => {
                preds.push(other);
                Predicate::And(preds)
            }
            (pred, Predicate::And(mut preds)) => {
                preds.insert(0, pred);
                Predicate::And(preds)
            }
            (a, b) => Predicate::And(vec![a, b]),
        }
    }

    pub fn or(self, other: Self) -> Self {
        match (self, other) {
            (Predicate::Or(mut preds), Predicate::Or(others)) => {
                preds.extend(others);
                Predicate::Or(preds)
            }
            (Predicate::Or(mut preds), other) => {
                preds.push(other);
                Predicate::Or(preds)
            }
            (pred, Predicate::Or(mut preds)) => {
                preds.insert(0, pred);
                Predicate::Or(preds)
            }
            (a, b) => Predicate::Or(vec![a, b]),
        }
    }

    /// Check if this predicate implies another (conservative approximation)
    pub fn implies(&self, other: &Self) -> bool {
        if self == other {
            return true;
        }

        match (self, other) {
            // True implies nothing except True
            (Predicate::True, Predicate::True) => true,
            (Predicate::True, _) => false,

            // Everything implies True
            (_, Predicate::True) => true,

            // P && Q implies P and implies Q
            (Predicate::And(preds), other) => {
                preds.iter().any(|p| p.implies(other))
            }

            // P implies P || Q
            (pred, Predicate::Or(preds)) => {
                preds.iter().any(|p| pred.implies(p))
            }

            // Conservative: assume no implication
            _ => false,
        }
    }
}

impl Type {
    pub fn is_subtype(&self, other: &Type) -> bool {
        use Type::*;

        match (self, other) {
            (_, Any) => true,
            (Never, _) => true,
            (a, b) if a == b => true,

            // Union handling: A <: B | C if A <: B or A <: C
            (a, Union(types)) => types.iter().any(|t| a.is_subtype(t)),
            (Union(types), b) => types.iter().all(|t| t.is_subtype(b)),

            // Intersection: A & B <: C if A <: C or B <: C
            (Intersection(types), c) => types.iter().any(|t| t.is_subtype(c)),

            // Structural subtyping for containers
            (List(a), List(b)) => a.is_subtype(b),
            (Set(a), Set(b)) => a.is_subtype(b),
            (Dict(k1, v1), Dict(k2, v2)) => k1.is_subtype(k2) && v1.is_subtype(v2),

            // Tuple covariance
            (Tuple(a), Tuple(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.is_subtype(y))
            }

            // Function contravariance in params, covariance in return
            (Function(p1, r1), Function(p2, r2)) => {
                p1.len() == p2.len()
                    && p2.iter().zip(p1.iter()).all(|(a, b)| a.is_subtype(b))
                    && r1.is_subtype(r2)
            }

            // Effect types: covariant in type, must have subset of effects
            (Effect(t1, e1), Effect(t2, e2)) => {
                t1.is_subtype(t2) && e1.is_subset(e2)
            }
            (t, Effect(inner, _)) => t.is_subtype(inner), // Can drop effects going up

            // Refinement types: covariant in base type, must satisfy predicate
            (Refinement(t1, p1), Refinement(t2, p2)) => {
                t1.is_subtype(t2) && p1.implies(p2)
            }
            (Refinement(t, _), other) => t.is_subtype(other), // Can drop refinement
            (t, Refinement(inner, _)) => t.is_subtype(inner), // Conservatively allow

            // Dependent types: must match constraint
            (Dependent(t1, c1), Dependent(t2, c2)) => {
                t1.is_subtype(t2) && c1 == c2
            }
            (Dependent(t, _), other) => t.is_subtype(other),

            // Nominal types: must have same name (no structural subtyping)
            (Nominal(n1, _), Nominal(n2, _)) => n1 == n2,
            (Nominal(_, inner), other) if other == &Class(String::new()) => {
                inner.is_subtype(other)
            }

            // Recursive types: unfold and check
            (Recursive(_, t1), Recursive(_, t2)) => t1.is_subtype(t2),

            // Conditional types: evaluate and check
            (Conditional { .. }, _) => false, // TODO: Implement evaluation

            // Higher-kinded types: structural equality for now
            (HigherKinded(n1, p1), HigherKinded(n2, p2)) => n1 == n2 && p1 == p2,

            _ => false,
        }
    }

    /// Create an effect type
    pub fn with_effect(self, effect: Effect) -> Type {
        Type::Effect(Box::new(self), EffectSet::single(effect))
    }

    /// Create a refinement type
    pub fn refine(self, predicate: Predicate) -> Type {
        Type::Refinement(Box::new(self), predicate)
    }

    /// Create a nominal wrapper
    pub fn nominal(name: String, inner: Type) -> Type {
        Type::Nominal(name, Box::new(inner))
    }

    pub fn union(types: Vec<Type>) -> Type {
        // Threshold for SIMD optimization: use SIMD for large unions
        const SIMD_THRESHOLD: usize = 10;

        let mut simplified = Vec::new();

        // Flatten nested unions
        for ty in types {
            if let Type::Union(inner) = ty {
                simplified.extend(inner);
            } else if ty != Type::Never {
                simplified.push(ty);
            }
        }

        // Fast path for simple cases
        match simplified.len() {
            0 => return Type::Never,
            1 => return simplified.into_iter().next().unwrap(),
            _ => {}
        }

        // Use SIMD for large unions
        if simplified.len() >= SIMD_THRESHOLD {
            return Self::union_simd(simplified);
        }

        // Standard path for small unions: remove subtypes
        let mut result = Vec::new();
        for ty in simplified {
            if !result.iter().any(|t| ty.is_subtype(t)) {
                result.retain(|t| !t.is_subtype(&ty));
                result.push(ty);
            }
        }

        match result.len() {
            0 => Type::Never,
            1 => result.into_iter().next().unwrap(),
            _ => Type::Union(result),
        }
    }

    /// SIMD-optimized union for large type sets
    fn union_simd(types: Vec<Type>) -> Type {
        use crate::core::intern::{intern, get_type};
        use crate::ffi::TypeSet;

        // Intern all types and create TypeSet
        let mut type_set = TypeSet::new();
        let mut original_types = std::collections::HashMap::new();

        for ty in types {
            let id = intern(ty.clone());
            type_set.insert(id);
            original_types.insert(id, ty);
        }

        // Convert back to types (already deduplicated by TypeSet)
        let ids = type_set.to_ids();
        let result_types: Vec<Type> = ids
            .iter()
            .filter_map(|id| get_type(*id))
            .collect();

        // Remove subtypes using Rust logic (structure-aware)
        let mut simplified = Vec::new();
        for ty in result_types {
            if !simplified.iter().any(|t| ty.is_subtype(t)) {
                simplified.retain(|t| !t.is_subtype(&ty));
                simplified.push(ty);
            }
        }

        match simplified.len() {
            0 => Type::Never,
            1 => simplified.into_iter().next().unwrap(),
            _ => Type::Union(simplified),
        }
    }

    pub fn intersection(types: Vec<Type>) -> Type {
        const SIMD_THRESHOLD: usize = 10;

        let mut result = types;
        result.retain(|t| *t != Type::Any);

        match result.len() {
            0 => return Type::Any,
            1 => return result.into_iter().next().unwrap(),
            _ => {}
        }

        // Use SIMD for large intersections
        if result.len() >= SIMD_THRESHOLD {
            return Self::intersection_simd(result);
        }

        Type::Intersection(result)
    }

    /// SIMD-optimized intersection for large type sets
    fn intersection_simd(types: Vec<Type>) -> Type {
        use crate::core::intern::{intern, get_type};
        use crate::ffi::TypeSet;

        // Convert to TypeSets
        let type_sets: Vec<TypeSet> = types
            .iter()
            .map(|ty| {
                let mut set = TypeSet::new();
                set.insert(intern(ty.clone()));
                set
            })
            .collect();

        // Use SIMD intersection
        let refs: Vec<&TypeSet> = type_sets.iter().collect();
        let result_set = TypeSet::intersection_many(&refs);

        // Convert back to types
        let ids = result_set.to_ids();
        let result_types: Vec<Type> = ids
            .iter()
            .filter_map(|id| get_type(*id))
            .collect();

        match result_types.len() {
            0 => Type::Never,
            1 => result_types.into_iter().next().unwrap(),
            _ => Type::Intersection(result_types),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Any => write!(f, "Any"),
            Type::Never => write!(f, "Never"),
            Type::None => write!(f, "None"),
            Type::Bool => write!(f, "bool"),
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Str => write!(f, "str"),
            Type::Bytes => write!(f, "bytes"),
            Type::List(t) => write!(f, "list[{}]", t),
            Type::Tuple(ts) => write!(f, "({})", ts.iter().map(|t| format!("{}", t)).collect::<Vec<_>>().join(", ")),
            Type::Dict(k, v) => write!(f, "dict[{}, {}]", k, v),
            Type::Set(t) => write!(f, "set[{}]", t),
            Type::Function(params, ret) => {
                write!(f, "({}) -> {}",
                    params.iter().map(|t| format!("{}", t)).collect::<Vec<_>>().join(", "),
                    ret)
            }
            Type::Union(ts) => write!(f, "{}", ts.iter().map(|t| format!("{}", t)).collect::<Vec<_>>().join(" | ")),
            Type::Intersection(ts) => write!(f, "{}", ts.iter().map(|t| format!("{}", t)).collect::<Vec<_>>().join(" & ")),
            Type::Generic(name, args) => {
                if args.is_empty() {
                    write!(f, "{}", name)
                } else {
                    write!(f, "{}[{}]", name, args.iter().map(|t| format!("{}", t)).collect::<Vec<_>>().join(", "))
                }
            }
            Type::Class(name) => write!(f, "{}", name),
            Type::Var(id) => write!(f, "T{}", id),

            // Phase 3 types
            Type::Effect(t, effects) => {
                if effects.is_pure() {
                    write!(f, "{}", t)
                } else {
                    write!(f, "{} ! {}", t, effects)
                }
            }
            Type::Refinement(t, pred) => write!(f, "{}[{}]", t, pred),
            Type::Dependent(t, constraint) => write!(f, "{}[{}]", t, constraint),
            Type::Nominal(name, _) => write!(f, "{}", name),
            Type::Conditional { condition, then_type, else_type } => {
                write!(f, "{} ? {} : {}", condition, then_type, else_type)
            }
            Type::Recursive(name, body) => write!(f, "rec {}. {}", name, body),
            Type::HigherKinded(name, params) => {
                write!(f, "{}[{}]", name,
                    params.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", "))
            }
        }
    }
}

impl fmt::Display for EffectSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{}}}",
            self.effects.iter()
                .map(|e| format!("{:?}", e))
                .collect::<Vec<_>>()
                .join(", "))
    }
}

impl fmt::Display for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Predicate::True => write!(f, "true"),
            Predicate::Compare { op, left, right } => {
                write!(f, "{} {} {}", left, op, right)
            }
            Predicate::And(preds) => write!(f, "({})",
                preds.iter().map(|p| format!("{}", p)).collect::<Vec<_>>().join(" && ")),
            Predicate::Or(preds) => write!(f, "({})",
                preds.iter().map(|p| format!("{}", p)).collect::<Vec<_>>().join(" || ")),
            Predicate::Not(pred) => write!(f, "!{}", pred),
            Predicate::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Display for CompareOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompareOp::Eq => write!(f, "=="),
            CompareOp::Ne => write!(f, "!="),
            CompareOp::Lt => write!(f, "<"),
            CompareOp::Le => write!(f, "<="),
            CompareOp::Gt => write!(f, ">"),
            CompareOp::Ge => write!(f, ">="),
        }
    }
}

impl fmt::Display for PredicateExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PredicateExpr::Value => write!(f, "value"),
            PredicateExpr::Literal(n) => write!(f, "{}", n),
            PredicateExpr::Property(prop) => write!(f, "{}", prop),
            PredicateExpr::BinOp(left, op, right) => write!(f, "({} {} {})", left, op, right),
        }
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Mod => write!(f, "%"),
        }
    }
}

impl fmt::Display for DependentConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependentConstraint::Length(n) => write!(f, "len={}", n),
            DependentConstraint::LengthRange(min, max) => write!(f, "{}<=len<={}", min, max),
            DependentConstraint::ValueEq(expr) => write!(f, "value={}", expr),
            DependentConstraint::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Display for TypeCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeCondition::Extends(t, u) => write!(f, "{} extends {}", t, u),
            TypeCondition::Equal(t, u) => write!(f, "{} == {}", t, u),
            TypeCondition::HasProperty(t, prop) => write!(f, "{}.{}", t, prop),
            TypeCondition::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Class member kinds for attribute resolution
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MemberKind {
    Method(Type),      // Method with function type
    Property(Type),    // Property/field with value type
    ClassVar(Type),    // Class variable
}

/// Class schema: compact representation of class structure
#[derive(Debug, Clone)]
pub struct ClassSchema {
    pub name: String,
    pub members: DashMap<String, MemberKind>,
    pub bases: Vec<String>,  // Base class names for inheritance
}

impl ClassSchema {
    pub fn new(name: String) -> Self {
        Self {
            name,
            members: DashMap::new(),
            bases: Vec::new(),
        }
    }

    pub fn with_bases(mut self, bases: Vec<String>) -> Self {
        self.bases = bases;
        self
    }

    pub fn add_method(&self, name: String, ty: Type) {
        self.members.insert(name, MemberKind::Method(ty));
    }

    pub fn add_property(&self, name: String, ty: Type) {
        self.members.insert(name, MemberKind::Property(ty));
    }

    pub fn add_class_var(&self, name: String, ty: Type) {
        self.members.insert(name, MemberKind::ClassVar(ty));
    }

    pub fn has_member(&self, name: &str) -> bool {
        self.members.contains_key(name)
    }

    pub fn get_member(&self, name: &str) -> Option<Type> {
        self.members.get(name).map(|m| match m.value() {
            MemberKind::Method(ty) | MemberKind::Property(ty) | MemberKind::ClassVar(ty) => ty.clone(),
        })
    }
}

pub struct TypeContext {
    types: DashMap<String, Type>,
    classes: DashMap<String, ClassSchema>,
    next_var: std::sync::atomic::AtomicU64,
}

impl TypeContext {
    pub fn new() -> Self {
        let ctx = Self {
            types: DashMap::new(),
            classes: DashMap::new(),
            next_var: std::sync::atomic::AtomicU64::new(0),
        };
        ctx.init_builtins();
        ctx
    }

    /// Initialize built-in type attributes
    fn init_builtins(&self) {
        // str methods
        let str_schema = ClassSchema::new("str".to_string());
        str_schema.add_method("upper".to_string(), Type::Function(vec![], Box::new(Type::Str)));
        str_schema.add_method("lower".to_string(), Type::Function(vec![], Box::new(Type::Str)));
        str_schema.add_method("strip".to_string(), Type::Function(vec![], Box::new(Type::Str)));
        str_schema.add_method("split".to_string(), Type::Function(vec![Type::Str], Box::new(Type::List(Box::new(Type::Str)))));
        str_schema.add_method("join".to_string(), Type::Function(vec![Type::List(Box::new(Type::Str))], Box::new(Type::Str)));
        str_schema.add_method("replace".to_string(), Type::Function(vec![Type::Str, Type::Str], Box::new(Type::Str)));
        str_schema.add_method("startswith".to_string(), Type::Function(vec![Type::Str], Box::new(Type::Bool)));
        str_schema.add_method("endswith".to_string(), Type::Function(vec![Type::Str], Box::new(Type::Bool)));
        str_schema.add_method("find".to_string(), Type::Function(vec![Type::Str], Box::new(Type::Int)));
        self.classes.insert("str".to_string(), str_schema);

        // list methods
        let list_schema = ClassSchema::new("list".to_string());
        list_schema.add_method("append".to_string(), Type::Function(vec![Type::Any], Box::new(Type::None)));
        list_schema.add_method("extend".to_string(), Type::Function(vec![Type::List(Box::new(Type::Any))], Box::new(Type::None)));
        list_schema.add_method("pop".to_string(), Type::Function(vec![], Box::new(Type::Any)));
        list_schema.add_method("remove".to_string(), Type::Function(vec![Type::Any], Box::new(Type::None)));
        list_schema.add_method("clear".to_string(), Type::Function(vec![], Box::new(Type::None)));
        list_schema.add_method("sort".to_string(), Type::Function(vec![], Box::new(Type::None)));
        list_schema.add_method("reverse".to_string(), Type::Function(vec![], Box::new(Type::None)));
        list_schema.add_method("copy".to_string(), Type::Function(vec![], Box::new(Type::List(Box::new(Type::Any)))));
        self.classes.insert("list".to_string(), list_schema);

        // dict methods
        let dict_schema = ClassSchema::new("dict".to_string());
        dict_schema.add_method("keys".to_string(), Type::Function(vec![], Box::new(Type::List(Box::new(Type::Any)))));
        dict_schema.add_method("values".to_string(), Type::Function(vec![], Box::new(Type::List(Box::new(Type::Any)))));
        dict_schema.add_method("items".to_string(), Type::Function(vec![], Box::new(Type::List(Box::new(Type::Tuple(vec![Type::Any, Type::Any]))))));
        dict_schema.add_method("get".to_string(), Type::Function(vec![Type::Any], Box::new(Type::Any)));
        dict_schema.add_method("pop".to_string(), Type::Function(vec![Type::Any], Box::new(Type::Any)));
        dict_schema.add_method("clear".to_string(), Type::Function(vec![], Box::new(Type::None)));
        dict_schema.add_method("update".to_string(), Type::Function(vec![Type::Dict(Box::new(Type::Any), Box::new(Type::Any))], Box::new(Type::None)));
        self.classes.insert("dict".to_string(), dict_schema);

        // set methods
        let set_schema = ClassSchema::new("set".to_string());
        set_schema.add_method("add".to_string(), Type::Function(vec![Type::Any], Box::new(Type::None)));
        set_schema.add_method("remove".to_string(), Type::Function(vec![Type::Any], Box::new(Type::None)));
        set_schema.add_method("discard".to_string(), Type::Function(vec![Type::Any], Box::new(Type::None)));
        set_schema.add_method("clear".to_string(), Type::Function(vec![], Box::new(Type::None)));
        set_schema.add_method("union".to_string(), Type::Function(vec![Type::Set(Box::new(Type::Any))], Box::new(Type::Set(Box::new(Type::Any)))));
        set_schema.add_method("intersection".to_string(), Type::Function(vec![Type::Set(Box::new(Type::Any))], Box::new(Type::Set(Box::new(Type::Any)))));
        self.classes.insert("set".to_string(), set_schema);
    }

    pub fn fresh_var(&self) -> Type {
        let id = self.next_var.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Type::Var(id)
    }

    pub fn set_type(&self, name: String, ty: Type) {
        self.types.insert(name, ty);
    }

    pub fn get_type(&self, name: &str) -> Option<Type> {
        self.types.get(name).map(|r| r.value().clone())
    }

    pub fn register_class(&self, schema: ClassSchema) {
        self.classes.insert(schema.name.clone(), schema);
    }

    pub fn get_class(&self, name: &str) -> Option<ClassSchema> {
        self.classes.get(name).map(|r| r.value().clone())
    }

    /// Check if a type has an attribute and return its type
    pub fn has_attribute(&self, ty: &Type, attr: &str) -> Option<Type> {
        match ty {
            Type::Class(name) => self.lookup_class_attribute(name, attr),
            Type::Str => self.lookup_class_attribute("str", attr),
            Type::List(_) => self.lookup_class_attribute("list", attr),
            Type::Dict(_, _) => self.lookup_class_attribute("dict", attr),
            Type::Set(_) => self.lookup_class_attribute("set", attr),
            Type::Union(types) => {
                // Union: attribute must exist in all variants
                let mut attr_ty = None;
                for t in types {
                    match self.has_attribute(t, attr) {
                        Some(ty) => {
                            attr_ty = Some(match attr_ty {
                                None => ty,
                                Some(existing) => Type::union(vec![existing, ty]),
                            });
                        }
                        None => return None,
                    }
                }
                attr_ty
            }
            Type::Intersection(types) => {
                // Intersection: attribute from any type works
                for t in types {
                    if let Some(ty) = self.has_attribute(t, attr) {
                        return Some(ty);
                    }
                }
                None
            }
            Type::Refinement(inner, _) => self.has_attribute(inner, attr),
            Type::Effect(inner, _) => self.has_attribute(inner, attr),
            Type::Dependent(inner, _) => self.has_attribute(inner, attr),
            Type::Nominal(_, inner) => self.has_attribute(inner, attr),
            _ => None,
        }
    }

    fn lookup_class_attribute(&self, class_name: &str, attr: &str) -> Option<Type> {
        if let Some(schema) = self.classes.get(class_name) {
            if let Some(ty) = schema.get_member(attr) {
                return Some(ty);
            }
            // Check base classes
            for base in &schema.bases {
                if let Some(ty) = self.lookup_class_attribute(base, attr) {
                    return Some(ty);
                }
            }
        }
        None
    }

    /// Get all available attributes for a type (for suggestions)
    pub fn get_attributes(&self, ty: &Type) -> Vec<String> {
        match ty {
            Type::Class(name) => self.get_class_attributes(name),
            Type::Str => self.get_class_attributes("str"),
            Type::List(_) => self.get_class_attributes("list"),
            Type::Dict(_, _) => self.get_class_attributes("dict"),
            Type::Set(_) => self.get_class_attributes("set"),
            _ => Vec::new(),
        }
    }

    fn get_class_attributes(&self, class_name: &str) -> Vec<String> {
        let mut attrs = Vec::new();
        if let Some(schema) = self.classes.get(class_name) {
            attrs.extend(schema.members.iter().map(|r| r.key().clone()));
            for base in &schema.bases {
                attrs.extend(self.get_class_attributes(base));
            }
        }
        attrs
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

