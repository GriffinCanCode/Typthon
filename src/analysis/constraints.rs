use crate::core::types::{Type, TypeContext};
use crate::errors::{TypeError, SourceLocation};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constraint {
    /// T must be a subtype of U
    Subtype(Type, Type),

    /// T must equal U
    Equal(Type, Type),

    /// T must have attribute with given name and type
    HasAttribute(Type, String, Type),

    /// T must be callable with given signature
    Callable(Type, Vec<Type>, Type),

    /// T must be a protocol/structural type
    Protocol(Type, Vec<(String, Type)>),

    /// T must satisfy a bound (upper bound)
    Bounded(Type, Type),

    /// T must be numeric
    Numeric(Type),

    /// T must be comparable
    Comparable(Type),

    /// T must be hashable
    Hashable(Type),
}

pub struct ConstraintSolver {
    constraints: Vec<Constraint>,
    bounds: HashMap<u64, Type>, // Type variable bounds
    errors: Vec<TypeError>,
    ctx: Option<Arc<TypeContext>>,
}

impl ConstraintSolver {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            bounds: HashMap::new(),
            errors: Vec::new(),
            ctx: None,
        }
    }

    pub fn with_context(ctx: Arc<TypeContext>) -> Self {
        Self {
            constraints: Vec::new(),
            bounds: HashMap::new(),
            errors: Vec::new(),
            ctx: Some(ctx),
        }
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    pub fn add_bound(&mut self, var: u64, bound: Type) {
        self.bounds.entry(var).or_insert(bound);
    }

    pub fn solve(&mut self) -> Result<(), Vec<TypeError>> {
        // Iteratively solve constraints
        let mut changed = true;
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 100;

        while changed && iterations < MAX_ITERATIONS {
            changed = false;
            iterations += 1;

            let constraints = std::mem::take(&mut self.constraints);
            for constraint in constraints {
                match self.solve_constraint(&constraint) {
                    Ok(true) => changed = true,
                    Ok(false) => self.constraints.push(constraint),
                    Err(error) => {
                        self.errors.push(error);
                    }
                }
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    fn solve_constraint(&mut self, constraint: &Constraint) -> Result<bool, TypeError> {
        match constraint {
            Constraint::Subtype(sub, sup) => self.check_subtype(sub, sup),
            Constraint::Equal(a, b) => self.check_equal(a, b),
            Constraint::HasAttribute(ty, attr, attr_ty) => {
                self.check_has_attribute(ty, attr, attr_ty)
            }
            Constraint::Callable(ty, params, ret) => self.check_callable(ty, params, ret),
            Constraint::Protocol(ty, methods) => self.check_protocol(ty, methods),
            Constraint::Bounded(var, bound) => self.check_bounded(var, bound),
            Constraint::Numeric(ty) => self.check_numeric(ty),
            Constraint::Comparable(ty) => self.check_comparable(ty),
            Constraint::Hashable(ty) => self.check_hashable(ty),
        }
    }

    fn check_subtype(&mut self, sub: &Type, sup: &Type) -> Result<bool, TypeError> {
        if sub.is_subtype(sup) {
            Ok(true)
        } else {
            // Try to extract type variables and add bounds
            if let Type::Var(id) = sub {
                if let Some(bound) = self.bounds.get(id) {
                    if bound.is_subtype(sup) {
                        return Ok(true);
                    }
                }
                // Add bound
                self.bounds.insert(*id, sup.clone());
                Ok(true)
            } else {
                Err(TypeError::type_mismatch(
                    sup.clone(),
                    sub.clone(),
                    SourceLocation::new(0, 0, 0, 0),
                ))
            }
        }
    }

    fn check_equal(&mut self, a: &Type, b: &Type) -> Result<bool, TypeError> {
        if a == b {
            Ok(true)
        } else if let Type::Var(id) = a {
            self.bounds.insert(*id, b.clone());
            Ok(true)
        } else if let Type::Var(id) = b {
            self.bounds.insert(*id, a.clone());
            Ok(true)
        } else {
            Err(TypeError::type_mismatch(
                a.clone(),
                b.clone(),
                SourceLocation::new(0, 0, 0, 0),
            ))
        }
    }

    fn check_has_attribute(
        &self,
        ty: &Type,
        attr: &str,
        expected_ty: &Type,
    ) -> Result<bool, TypeError> {
        // Get context if available
        let ctx = match &self.ctx {
            Some(c) => c,
            None => return Ok(false), // Defer if no context
        };

        // Check if attribute exists
        match ctx.has_attribute(ty, attr) {
            Some(actual_ty) => {
                // Verify attribute type matches expected
                if actual_ty.is_subtype(expected_ty) || expected_ty.is_subtype(&actual_ty) {
                    Ok(true)
                } else {
                    Err(TypeError::new(
                        crate::errors::ErrorKind::TypeMismatch {
                            expected: expected_ty.to_string(),
                            found: actual_ty.to_string(),
                        },
                        SourceLocation::new(0, 0, 0, 0),
                    ))
                }
            }
            None => {
                // Attribute doesn't exist - generate helpful error
                let available = ctx.get_attributes(ty);
                let similar = crate::errors::find_similar_names(attr, &available, 2);

                let mut error = TypeError::new(
                    crate::errors::ErrorKind::InvalidAttribute {
                        ty: ty.to_string(),
                        attr: attr.to_string(),
                    },
                    SourceLocation::new(0, 0, 0, 0),
                );

                if !similar.is_empty() {
                    error = error.with_suggestions(
                        similar.iter()
                            .take(3)
                            .map(|s| format!("Did you mean '{}'?", s))
                            .collect()
                    );
                }

                Err(error)
            }
        }
    }

    fn check_callable(
        &self,
        ty: &Type,
        params: &[Type],
        ret: &Type,
    ) -> Result<bool, TypeError> {
        if let Type::Function(fn_params, fn_ret) = ty {
            if fn_params.len() != params.len() {
                return Err(TypeError::invalid_arg_count(
                    params.len(),
                    fn_params.len(),
                    SourceLocation::new(0, 0, 0, 0),
                ));
            }

            // Check parameter types (contravariant)
            for (expected, actual) in params.iter().zip(fn_params.iter()) {
                if !expected.is_subtype(actual) {
                    return Err(TypeError::type_mismatch(
                        expected.clone(),
                        actual.clone(),
                        SourceLocation::new(0, 0, 0, 0),
                    ));
                }
            }

            // Check return type (covariant)
            if !fn_ret.is_subtype(ret) {
                return Err(TypeError::type_mismatch(
                    ret.clone(),
                    *fn_ret.clone(),
                    SourceLocation::new(0, 0, 0, 0),
                ));
            }

            Ok(true)
        } else {
            Ok(false) // Not a function, defer
        }
    }

    fn check_protocol(&self, _ty: &Type, _methods: &[(String, Type)]) -> Result<bool, TypeError> {
        // TODO: Implement protocol checking
        Ok(false)
    }

    fn check_bounded(&mut self, var: &Type, bound: &Type) -> Result<bool, TypeError> {
        if let Type::Var(id) = var {
            if let Some(existing_bound) = self.bounds.get(id) {
                // Check consistency
                if !existing_bound.is_subtype(bound) && !bound.is_subtype(existing_bound) {
                    return Err(TypeError::type_mismatch(
                        existing_bound.clone(),
                        bound.clone(),
                        SourceLocation::new(0, 0, 0, 0),
                    ));
                }
            } else {
                self.bounds.insert(*id, bound.clone());
            }
            Ok(true)
        } else {
            self.check_subtype(var, bound)
        }
    }

    fn check_numeric(&self, ty: &Type) -> Result<bool, TypeError> {
        match ty {
            Type::Int | Type::Float => Ok(true),
            Type::Var(_) => Ok(false), // Defer
            Type::Union(types) => {
                // All must be numeric
                for t in types {
                    self.check_numeric(t)?;
                }
                Ok(true)
            }
            _ => Err(TypeError::new(
                crate::errors::ErrorKind::TypeMismatch {
                    expected: "numeric type".to_string(),
                    found: ty.to_string(),
                },
                SourceLocation::new(0, 0, 0, 0),
            )),
        }
    }

    fn check_comparable(&self, ty: &Type) -> Result<bool, TypeError> {
        match ty {
            Type::Int | Type::Float | Type::Str | Type::Bool => Ok(true),
            Type::Var(_) => Ok(false), // Defer
            _ => Ok(false), // Not comparable by default
        }
    }

    fn check_hashable(&self, ty: &Type) -> Result<bool, TypeError> {
        match ty {
            Type::Int | Type::Float | Type::Str | Type::Bool | Type::Bytes | Type::None => Ok(true),
            Type::Tuple(elems) => {
                // All elements must be hashable
                for elem in elems {
                    self.check_hashable(elem)?;
                }
                Ok(true)
            }
            Type::List(_) | Type::Dict(_, _) | Type::Set(_) => Err(TypeError::new(
                crate::errors::ErrorKind::TypeMismatch {
                    expected: "hashable type".to_string(),
                    found: ty.to_string(),
                },
                SourceLocation::new(0, 0, 0, 0),
            )),
            Type::Var(_) => Ok(false), // Defer
            _ => Ok(false),
        }
    }

    pub fn get_bound(&self, var: u64) -> Option<&Type> {
        self.bounds.get(&var)
    }

    pub fn errors(&self) -> &[TypeError] {
        &self.errors
    }
}

impl Default for ConstraintSolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Generic type parameter with constraints
#[derive(Debug, Clone)]
pub struct TypeParameter {
    pub name: String,
    pub bound: Option<Type>,
    pub constraints: Vec<Type>,
    pub variance: super::variance::Variance,
}

impl TypeParameter {
    pub fn new(name: String) -> Self {
        Self {
            name,
            bound: None,
            constraints: Vec::new(),
            variance: super::variance::Variance::Invariant,
        }
    }

    pub fn with_bound(mut self, bound: Type) -> Self {
        self.bound = Some(bound);
        self
    }

    pub fn with_constraint(mut self, constraint: Type) -> Self {
        self.constraints.push(constraint);
        self
    }

    pub fn with_variance(mut self, variance: super::variance::Variance) -> Self {
        self.variance = variance;
        self
    }

    pub fn validate(&self, ty: &Type) -> Result<(), String> {
        // Check upper bound
        if let Some(bound) = &self.bound {
            if !ty.is_subtype(bound) {
                return Err(format!(
                    "Type {} does not satisfy bound {}",
                    ty, bound
                ));
            }
        }

        // Check constraints
        for constraint in &self.constraints {
            if !ty.is_subtype(constraint) {
                return Err(format!(
                    "Type {} does not satisfy constraint {}",
                    ty, constraint
                ));
            }
        }

        Ok(())
    }
}

/// Generic type definition with type parameters
#[derive(Debug, Clone)]
pub struct GenericType {
    pub name: String,
    pub params: Vec<TypeParameter>,
    pub definition: Type,
}

impl GenericType {
    pub fn new(name: String, params: Vec<TypeParameter>, definition: Type) -> Self {
        Self {
            name,
            params,
            definition,
        }
    }

    pub fn instantiate(&self, args: &[Type]) -> Result<Type, String> {
        if args.len() != self.params.len() {
            return Err(format!(
                "Wrong number of type arguments: expected {}, got {}",
                self.params.len(),
                args.len()
            ));
        }

        // Validate each argument against parameter constraints
        for (param, arg) in self.params.iter().zip(args.iter()) {
            param.validate(arg)?;
        }

        // Substitute type parameters with arguments
        Ok(self.substitute(&self.definition, args))
    }

    fn substitute(&self, ty: &Type, args: &[Type]) -> Type {
        match ty {
            Type::Generic(name, inner_args) if name == &self.name => {
                // Replace with actual arguments
                Type::Generic(
                    name.clone(),
                    inner_args
                        .iter()
                        .map(|t| self.substitute(t, args))
                        .collect(),
                )
            }
            Type::List(inner) => Type::List(Box::new(self.substitute(inner, args))),
            Type::Set(inner) => Type::Set(Box::new(self.substitute(inner, args))),
            Type::Dict(k, v) => Type::Dict(
                Box::new(self.substitute(k, args)),
                Box::new(self.substitute(v, args)),
            ),
            Type::Tuple(elems) => Type::Tuple(elems.iter().map(|e| self.substitute(e, args)).collect()),
            Type::Function(params, ret) => Type::Function(
                params.iter().map(|p| self.substitute(p, args)).collect(),
                Box::new(self.substitute(ret, args)),
            ),
            Type::Union(types) => {
                Type::union(types.iter().map(|t| self.substitute(t, args)).collect())
            }
            Type::Intersection(types) => Type::intersection(
                types.iter().map(|t| self.substitute(t, args)).collect(),
            ),
            other => other.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtype_constraint() {
        let mut solver = ConstraintSolver::new();
        solver.add_constraint(Constraint::Subtype(Type::Int, Type::Any));
        assert!(solver.solve().is_ok());
    }

    #[test]
    fn test_numeric_constraint() {
        let mut solver = ConstraintSolver::new();
        solver.add_constraint(Constraint::Numeric(Type::Int));
        assert!(solver.solve().is_ok());

        let mut solver = ConstraintSolver::new();
        solver.add_constraint(Constraint::Numeric(Type::Str));
        assert!(solver.solve().is_err());
    }

    #[test]
    fn test_hashable_constraint() {
        let mut solver = ConstraintSolver::new();
        solver.add_constraint(Constraint::Hashable(Type::Int));
        assert!(solver.solve().is_ok());

        let mut solver = ConstraintSolver::new();
        solver.add_constraint(Constraint::Hashable(Type::List(Box::new(Type::Int))));
        assert!(solver.solve().is_err());
    }
}

