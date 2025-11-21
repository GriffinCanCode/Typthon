//! Core type system components
//!
//! This module contains the fundamental type definitions and type context
//! used throughout the Typthon type checker.

pub mod types;
pub mod intern;

pub use types::{Type, TypeContext, ClassSchema, MemberKind};

