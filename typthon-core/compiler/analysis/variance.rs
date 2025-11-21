use crate::compiler::types::Type;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variance {
    Covariant,     // T_A <: T_B => Generic[T_A] <: Generic[T_B]
    Contravariant, // T_A <: T_B => Generic[T_B] <: Generic[T_A]
    Invariant,     // No subtyping relationship
    Bivariant,     // Both directions (rare, usually a sign of error)
}

impl Variance {
    /// Compose two variances (for nested types)
    pub fn compose(self, other: Self) -> Self {
        use Variance::*;
        match (self, other) {
            (Covariant, Covariant) => Covariant,
            (Contravariant, Contravariant) => Covariant,
            (Contravariant, Covariant) | (Covariant, Contravariant) => Contravariant,
            (Invariant, _) | (_, Invariant) => Invariant,
            (Bivariant, v) | (v, Bivariant) => v,
        }
    }

    /// Flip variance (for contravariant positions)
    pub fn flip(self) -> Self {
        match self {
            Self::Covariant => Self::Contravariant,
            Self::Contravariant => Self::Covariant,
            Self::Invariant => Self::Invariant,
            Self::Bivariant => Self::Bivariant,
        }
    }
}

pub struct VarianceAnalyzer {
    cache: HashMap<String, Variance>,
}

impl VarianceAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            cache: HashMap::new(),
        };
        analyzer.init_builtins();
        analyzer
    }

    fn init_builtins(&mut self) {
        // Python builtin types and their variances
        self.cache.insert("List".to_string(), Variance::Invariant); // Mutable
        self.cache.insert("Tuple".to_string(), Variance::Covariant);
        self.cache.insert("Dict".to_string(), Variance::Invariant); // Mutable
        self.cache.insert("Set".to_string(), Variance::Invariant); // Mutable
        self.cache.insert("FrozenSet".to_string(), Variance::Covariant);
        self.cache.insert("Callable".to_string(), Variance::Contravariant); // Params
        self.cache.insert("Iterator".to_string(), Variance::Covariant);
        self.cache.insert("Iterable".to_string(), Variance::Covariant);
        self.cache.insert("Sequence".to_string(), Variance::Covariant);
        self.cache.insert("Mapping".to_string(), Variance::Covariant); // Immutable
    }

    /// Infer variance of a type parameter in a generic type
    pub fn infer_variance(&mut self, type_name: &str, param_index: usize) -> Variance {
        // Check cache
        let cache_key = format!("{}[{}]", type_name, param_index);
        if let Some(&variance) = self.cache.get(&cache_key) {
            return variance;
        }

        // Default heuristics
        let variance = match type_name {
            "List" | "Dict" | "Set" => Variance::Invariant, // Mutable containers
            "Tuple" | "FrozenSet" => Variance::Covariant,   // Immutable containers
            _ => Variance::Invariant, // Safe default
        };

        self.cache.insert(cache_key, variance);
        variance
    }

    /// Check if subtyping relationship holds with variance
    pub fn check_subtype_with_variance(
        &mut self,
        sub_generic: &str,
        sub_args: &[Type],
        super_generic: &str,
        super_args: &[Type],
    ) -> bool {
        if sub_generic != super_generic {
            return false;
        }

        if sub_args.len() != super_args.len() {
            return false;
        }

        for (i, (sub_arg, super_arg)) in sub_args.iter().zip(super_args.iter()).enumerate() {
            let variance = self.infer_variance(sub_generic, i);

            let valid = match variance {
                Variance::Covariant => sub_arg.is_subtype(super_arg),
                Variance::Contravariant => super_arg.is_subtype(sub_arg),
                Variance::Invariant => sub_arg == super_arg,
                Variance::Bivariant => true,
            };

            if !valid {
                return false;
            }
        }

        true
    }

    /// Validate variance annotation on a generic type definition
    pub fn validate_variance_annotation(
        &self,
        type_name: &str,
        declared_variance: Variance,
        actual_usage: Variance,
    ) -> Result<(), String> {
        // Check if declared variance is safe given actual usage
        match (declared_variance, actual_usage) {
            (Variance::Covariant, Variance::Covariant) => Ok(()),
            (Variance::Contravariant, Variance::Contravariant) => Ok(()),
            (Variance::Invariant, _) => Ok(()), // Always safe
            (Variance::Bivariant, _) => Ok(()), // Always safe (but unusual)
            (declared, actual) => Err(format!(
                "Type parameter of {} declared as {:?} but used as {:?}",
                type_name, declared, actual
            )),
        }
    }

    /// Compute variance of a type parameter based on its usage in type definition
    pub fn compute_variance(&mut self, ty: &Type, param_var: u64, position: Variance) -> Variance {
        match ty {
            Type::Var(id) if *id == param_var => position,

            Type::List(inner) | Type::Set(inner) => {
                // Mutable containers: invariant
                if self.contains_var(inner, param_var) {
                    Variance::Invariant
                } else {
                    Variance::Covariant
                }
            }

            Type::Tuple(elems) => {
                // Immutable: preserve variance
                elems
                    .iter()
                    .map(|elem| self.compute_variance(elem, param_var, position))
                    .fold(Variance::Covariant, |acc, v| {
                        if v == Variance::Invariant {
                            Variance::Invariant
                        } else {
                            acc
                        }
                    })
            }

            Type::Dict(k, v) => {
                // Mutable: invariant
                if self.contains_var(k, param_var) || self.contains_var(v, param_var) {
                    Variance::Invariant
                } else {
                    Variance::Covariant
                }
            }

            Type::Function(params, ret) => {
                // Parameters: contravariant, return: covariant
                let param_variance = params
                    .iter()
                    .map(|p| self.compute_variance(p, param_var, position.flip()))
                    .fold(Variance::Covariant, |acc, v| {
                        if v == Variance::Invariant {
                            Variance::Invariant
                        } else {
                            acc
                        }
                    });

                let ret_variance = self.compute_variance(ret, param_var, position);

                if param_variance == Variance::Invariant || ret_variance == Variance::Invariant {
                    Variance::Invariant
                } else {
                    position
                }
            }

            Type::Union(types) | Type::Intersection(types) => {
                // Preserve variance
                types
                    .iter()
                    .map(|t| self.compute_variance(t, param_var, position))
                    .fold(Variance::Covariant, |acc, v| {
                        if v == Variance::Invariant {
                            Variance::Invariant
                        } else {
                            acc
                        }
                    })
            }

            Type::Generic(_, args) => {
                // Recurse into generic arguments
                args.iter()
                    .map(|arg| self.compute_variance(arg, param_var, position))
                    .fold(Variance::Covariant, |acc, v| {
                        if v == Variance::Invariant {
                            Variance::Invariant
                        } else {
                            acc
                        }
                    })
            }

            _ => Variance::Covariant,
        }
    }

    fn contains_var(&self, ty: &Type, var: u64) -> bool {
        match ty {
            Type::Var(id) => *id == var,
            Type::List(inner) | Type::Set(inner) => self.contains_var(inner, var),
            Type::Dict(k, v) => self.contains_var(k, var) || self.contains_var(v, var),
            Type::Tuple(elems) | Type::Union(elems) | Type::Intersection(elems) => {
                elems.iter().any(|e| self.contains_var(e, var))
            }
            Type::Function(params, ret) => {
                params.iter().any(|p| self.contains_var(p, var)) || self.contains_var(ret, var)
            }
            Type::Generic(_, args) => args.iter().any(|a| self.contains_var(a, var)),
            _ => false,
        }
    }

    /// Check if a generic type parameter usage is variance-safe
    pub fn is_variance_safe(&mut self, ty: &Type, param_var: u64) -> bool {
        let covariant = self.compute_variance(ty, param_var, Variance::Covariant);
        let contravariant = self.compute_variance(ty, param_var, Variance::Contravariant);

        // Safe if consistent in all positions
        covariant != Variance::Invariant || contravariant != Variance::Invariant
    }
}

impl Default for VarianceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variance_composition() {
        assert_eq!(
            Variance::Covariant.compose(Variance::Covariant),
            Variance::Covariant
        );
        assert_eq!(
            Variance::Contravariant.compose(Variance::Contravariant),
            Variance::Covariant
        );
        assert_eq!(
            Variance::Covariant.compose(Variance::Contravariant),
            Variance::Contravariant
        );
    }

    #[test]
    fn test_variance_flip() {
        assert_eq!(Variance::Covariant.flip(), Variance::Contravariant);
        assert_eq!(Variance::Contravariant.flip(), Variance::Covariant);
        assert_eq!(Variance::Invariant.flip(), Variance::Invariant);
    }

    #[test]
    fn test_builtin_variances() {
        let analyzer = VarianceAnalyzer::new();
        assert_eq!(
            analyzer.cache.get("Tuple"),
            Some(&Variance::Covariant)
        );
        assert_eq!(
            analyzer.cache.get("List"),
            Some(&Variance::Invariant)
        );
    }
}

