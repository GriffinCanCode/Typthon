//! Typthon - High-performance gradual type system for Python
//!
//! Main library entry point that exposes the typhon Python package.

// Internal module structure (for organization)
pub mod internal {
    pub mod core;
    pub mod compiler;
    pub mod runtime;
}

// Include typthon-core as the main implementation
#[path = "../typthon-core/lib.rs"]
mod typthon_core;

// Re-export everything from typthon-core
pub use typthon_core::*;

// Include typthon Python package API (directory is called typhton)
#[path = "typhton/lib.rs"]
pub mod typthon;

