//! Type analysis and checking
//!
//! This module contains all type analysis components including the type checker,
//! inference engine, bidirectional type checking, constraint solving, variance analysis,
//! effect tracking, refinement types, and advanced types (recursive, HKTs, conditional).

pub mod checker;
pub mod inference;
pub mod bidirectional;
pub mod constraints;
pub mod variance;
pub mod effects;
pub mod refinement;
pub mod advanced;

pub use checker::TypeChecker;
pub use inference::InferenceEngine;
pub use bidirectional::BiInfer;
pub use constraints::{Constraint, ConstraintSolver, TypeParameter, GenericType};
pub use variance::{Variance, VarianceAnalyzer};
pub use effects::EffectAnalyzer;
pub use refinement::RefinementAnalyzer;
pub use advanced::AdvancedTypeAnalyzer;

