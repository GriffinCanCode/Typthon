//! Advanced concurrency patterns for type checking and compilation
//!
//! This module provides multiple concurrency models optimized for compiler workloads:
//!
//! - **Actor Model** (`actor`): Message-passing concurrency with supervision
//! - **Structured Concurrency** (`structured`): Scoped tasks with proper cancellation
//! - **Async I/O** (`async_io`): Non-blocking file operations with caching
//! - **Query System** (`query`): Salsa-based incremental computation
//! - **Pipeline Parallelism** (`pipeline`): Multi-stage compilation with flow control
//! - **Integration** (`integration`): Practical integration examples

pub mod actor;
pub mod structured;
pub mod async_io;
pub mod query;
pub mod pipeline;
pub mod integration;

// Re-export key types
pub use actor::{
    Actor, ActorAddr, ActorSystem, ActorId, ActorError, Message,
    Supervisor, SupervisionStrategy,
};
pub use structured::{
    TaskScope, Nursery, CancellationToken, scoped, scoped_with_limit,
};
pub use async_io::{
    FileCache, BatchFileReader, FileWatcher, BufferedWriter,
};
pub use query::{
    TypeCheckingDatabase, CompilerDatabase, QueryCoordinator,
    ModuleId as QueryModuleId, QueryStats,
};
pub use pipeline::{
    Stage, Pipeline, PipelineHandle, AsyncPipeline, CompilerPipeline, CompilerStage,
    BufferedPipeline, ControlledPipeline, FlowControl,
};
pub use integration::{
    TypeCheckActor, TypeCheckMessage, TypeCheckResponse,
    SupervisedTypeChecker, StructuredProjectAnalyzer, PipelinedCompiler,
};

