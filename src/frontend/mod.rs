//! Frontend components
//!
//! This module contains the parser, CLI, and configuration components
//! that form the user-facing interface of Typthon.

pub mod parser;
pub mod cli;
pub mod config;

pub use parser::parse_module;
pub use cli::main as cli_main;
pub use config::Config;

