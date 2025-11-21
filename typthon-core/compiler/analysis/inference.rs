use crate::compiler::types::Type;
use std::collections::HashMap;

pub type Constraint = (Type, Type);

pub struct InferenceEngine {
    constraints: Vec<Constraint>,
    substitution: HashMap<u64, Type>,
}

impl InferenceEngine {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            substitution: HashMap::new(),
        }
    }

    pub fn add_constraint(&mut self, left: Type, right: Type) {
        self.constraints.push((left, right));
    }

    pub fn solve(&mut self) -> Result<(), String> {
        while let Some((left, right)) = self.constraints.pop() {
            self.unify(left, right)?;
        }
        Ok(())
    }

    fn unify(&mut self, left: Type, right: Type) -> Result<(), String> {
        use Type::*;

        let left = self.apply_substitution(left);
        let right = self.apply_substitution(right);

        match (left, right) {
            (a, b) if a == b => Ok(()),

            (Var(id), ty) | (ty, Var(id)) => {
                if self.occurs_check(id, &ty) {
                    Err(format!("Infinite type: T{} = {}", id, ty))
                } else {
                    self.substitution.insert(id, ty);
                    Ok(())
                }
            }

            (List(a), List(b)) => self.unify(*a, *b),
            (Set(a), Set(b)) => self.unify(*a, *b),
            (Dict(k1, v1), Dict(k2, v2)) => {
                self.unify(*k1, *k2)?;
                self.unify(*v1, *v2)
            }

            (Tuple(a), Tuple(b)) if a.len() == b.len() => {
                for (x, y) in a.into_iter().zip(b) {
                    self.unify(x, y)?;
                }
                Ok(())
            }

            (Function(p1, r1), Function(p2, r2)) if p1.len() == p2.len() => {
                for (x, y) in p1.into_iter().zip(p2) {
                    self.unify(x, y)?;
                }
                self.unify(*r1, *r2)
            }

            (a, b) => Err(format!("Cannot unify {} with {}", a, b)),
        }
    }

    fn apply_substitution(&self, ty: Type) -> Type {
        match ty {
            Type::Var(id) => {
                if let Some(substituted) = self.substitution.get(&id) {
                    self.apply_substitution(substituted.clone())
                } else {
                    Type::Var(id)
                }
            }
            Type::List(inner) => Type::List(Box::new(self.apply_substitution(*inner))),
            Type::Set(inner) => Type::Set(Box::new(self.apply_substitution(*inner))),
            Type::Dict(k, v) => Type::Dict(
                Box::new(self.apply_substitution(*k)),
                Box::new(self.apply_substitution(*v)),
            ),
            Type::Tuple(types) => Type::Tuple(
                types.into_iter().map(|t| self.apply_substitution(t)).collect()
            ),
            Type::Function(params, ret) => Type::Function(
                params.into_iter().map(|t| self.apply_substitution(t)).collect(),
                Box::new(self.apply_substitution(*ret)),
            ),
            other => other,
        }
    }

    fn occurs_check(&self, var_id: u64, ty: &Type) -> bool {
        match ty {
            Type::Var(id) => *id == var_id,
            Type::List(inner) | Type::Set(inner) => self.occurs_check(var_id, inner),
            Type::Dict(k, v) => self.occurs_check(var_id, k) || self.occurs_check(var_id, v),
            Type::Tuple(types) | Type::Union(types) | Type::Intersection(types) => {
                types.iter().any(|t| self.occurs_check(var_id, t))
            }
            Type::Function(params, ret) => {
                params.iter().any(|t| self.occurs_check(var_id, t))
                    || self.occurs_check(var_id, ret)
            }
            _ => false,
        }
    }

    pub fn get_solution(&self, ty: Type) -> Type {
        self.apply_substitution(ty)
    }
}

impl Default for InferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

