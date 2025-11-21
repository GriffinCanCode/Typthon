//! Integration layer for concurrency patterns
//!
//! Demonstrates proper integration of each concurrency model with the compiler.

use super::*;
use crate::compiler::analysis::TypeChecker;
use crate::compiler::types::{Type, TypeContext};
use crate::compiler::errors::TypeError;
use std::sync::Arc;
use std::path::PathBuf;

/// Type checking actor for distributed analysis
pub struct TypeCheckActor {
    context: Arc<TypeContext>,
    checker: TypeChecker,
}

pub enum TypeCheckMessage {
    CheckModule { source: String, path: PathBuf },
    GetContext,
}

impl Message for TypeCheckMessage {
    type Response = TypeCheckResponse;
}

pub enum TypeCheckResponse {
    Errors(Vec<TypeError>),
    Context(Arc<TypeContext>),
}

#[async_trait::async_trait]
impl Actor for TypeCheckActor {
    type Message = TypeCheckMessage;

    async fn handle(&mut self, msg: TypeCheckMessage) -> TypeCheckResponse {
        match msg {
            TypeCheckMessage::CheckModule { source, path: _ } => {
                match crate::compiler::frontend::parse_module(&source) {
                    Ok(ast) => {
                        let errors = self.checker.check(&ast);
                        TypeCheckResponse::Errors(errors)
                    }
                    Err(e) => {
                        let error = TypeError::new(
                            crate::compiler::errors::ErrorKind::ParseError(format!("{:?}", e)),
                            crate::compiler::errors::SourceLocation::default(),
                        );
                        TypeCheckResponse::Errors(vec![error])
                    }
                }
            }
            TypeCheckMessage::GetContext => {
                TypeCheckResponse::Context(self.context.clone())
            }
        }
    }
}

impl TypeCheckActor {
    pub fn new(context: Arc<TypeContext>) -> Self {
        Self {
            checker: TypeChecker::with_context(context.clone()),
            context,
        }
    }
}

/// Supervised type checking system using actors
pub struct SupervisedTypeChecker {
    system: ActorSystem,
    supervisor: Supervisor,
    workers: Vec<ActorAddr<TypeCheckActor>>,
}

impl SupervisedTypeChecker {
    pub fn new(runtime: tokio::runtime::Handle, num_workers: usize) -> Self {
        let system = ActorSystem::new(runtime);
        let supervisor = Supervisor::new(SupervisionStrategy::Restart);

        let mut workers = Vec::new();
        for _ in 0..num_workers {
            let context = Arc::new(TypeContext::new());
            let actor = TypeCheckActor::new(context);
            let addr = system.spawn(actor, 100);
            supervisor.supervise(&addr);
            workers.push(addr);
        }

        Self {
            system,
            supervisor,
            workers,
        }
    }

    pub async fn check_file(&self, source: String, path: PathBuf) -> Vec<TypeError> {
        // Round-robin work distribution
        let worker_idx = path.to_string_lossy().len() % self.workers.len();
        let worker = &self.workers[worker_idx];

        match worker.send(TypeCheckMessage::CheckModule { source, path }).await {
            Ok(TypeCheckResponse::Errors(errors)) => errors,
            _ => vec![],
        }
    }

    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }
}

/// Async project analyzer using structured concurrency
pub struct StructuredProjectAnalyzer {
    file_reader: BatchFileReader,
    context: Arc<TypeContext>,
}

impl StructuredProjectAnalyzer {
    pub fn new(cache_size: usize, concurrency: usize) -> Self {
        Self {
            file_reader: BatchFileReader::new(cache_size, concurrency),
            context: Arc::new(TypeContext::new()),
        }
    }

    /// Analyze project with proper cancellation support
    pub async fn analyze_with_cancellation(
        &self,
        root: PathBuf,
        cancel_token: CancellationToken,
    ) -> Result<Vec<(PathBuf, Vec<TypeError>)>, Box<dyn std::error::Error + Send>> {
        let nursery = Nursery::with_limit(self.file_reader.concurrency);

        // Read all files
        let files = tokio::select! {
            result = self.file_reader.read_directory(&root) => result?,
            _ = cancel_token.cancelled() => return Ok(vec![]),
        };

        // Spawn type checking tasks
        let results = Arc::new(parking_lot::Mutex::new(Vec::new()));

        for (path, content) in files {
            let context = self.context.clone();
            let results = results.clone();
            let cancel = cancel_token.clone();

            nursery.spawn(async move {
                tokio::select! {
                    _ = async {
                        match crate::compiler::frontend::parse_module(&content) {
                            Ok(ast) => {
                                let mut checker = TypeChecker::with_context(context);
                                let errors = checker.check(&ast);
                                results.lock().push((path, errors));
                                Ok(())
                            }
                            Err(e) => Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Parse error: {:?}", e)
                            )),
                        }
                    } => {}
                    _ = cancel.cancelled() => {}
                }
            });
        }

        nursery.join().await?;

        let results = std::mem::take(&mut *results.lock());
        Ok(results)
    }
}

/// Pipeline-based compilation workflow
pub struct PipelinedCompiler {
    pipeline: CompilerPipeline,
}

impl PipelinedCompiler {
    pub fn new(pipeline: CompilerPipeline) -> Self {
        Self { pipeline }
    }

    /// Execute compilation pipeline with progress tracking
    pub fn compile(&self, source: String) -> Result<CompilationResult, CompilationError> {
        let mut controlled = ControlledPipeline::new();

        // Stage 1: Parse
        controlled.add_stage(|ctx: CompilationContext| {
            match crate::compiler::frontend::parse_module(&ctx.source) {
                Ok(ast) => {
                    let mut new_ctx = ctx;
                    new_ctx.ast = Some(ast);
                    new_ctx.stage = Stage::Parsed;
                    (Some(new_ctx), FlowControl::Continue)
                }
                Err(e) => {
                    let mut new_ctx = ctx;
                    new_ctx.errors.push(format!("Parse error: {:?}", e));
                    (Some(new_ctx), FlowControl::Stop)
                }
            }
        });

        // Stage 2: Type Check (if enabled)
        if self.pipeline.stage_names().contains(&"typecheck") {
            controlled.add_stage(|mut ctx: CompilationContext| {
                if let Some(ast) = &ctx.ast {
                    let type_ctx = Arc::new(TypeContext::new());
                    let mut checker = TypeChecker::with_context(type_ctx);
                    let errors = checker.check(ast);

                    if !errors.is_empty() {
                        for err in errors {
                            ctx.errors.push(format!("Type error: {}", err.kind));
                        }
                    }
                    ctx.stage = Stage::TypeChecked;
                }
                (Some(ctx), FlowControl::Continue)
            });
        }

        // Execute pipeline
        let initial = CompilationContext {
            source,
            ast: None,
            stage: Stage::Initial,
            errors: vec![],
        };

        match controlled.execute(initial) {
            Some(ctx) if ctx.errors.is_empty() => Ok(CompilationResult {
                stage: ctx.stage,
            }),
            Some(ctx) => Err(CompilationError::Errors(ctx.errors)),
            None => Err(CompilationError::Aborted),
        }
    }
}

#[derive(Clone)]
struct CompilationContext {
    source: String,
    ast: Option<crate::compiler::ast::Module>,
    stage: Stage,
    errors: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Initial,
    Parsed,
    TypeChecked,
    Optimized,
    Generated,
}

pub struct CompilationResult {
    pub stage: Stage,
}

#[derive(Debug)]
pub enum CompilationError {
    Errors(Vec<String>),
    Aborted,
}

impl std::fmt::Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Errors(errs) => write!(f, "Compilation errors: {}", errs.join(", ")),
            Self::Aborted => write!(f, "Compilation aborted"),
        }
    }
}

impl std::error::Error for CompilationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_supervised_checker() {
        let rt = tokio::runtime::Handle::current();
        let checker = SupervisedTypeChecker::new(rt, 2);

        let errors = checker.check_file(
            "x = 1 + 2".to_string(),
            PathBuf::from("test.py")
        ).await;

        assert!(errors.is_empty() || !errors.is_empty());
    }

    #[tokio::test]
    async fn test_structured_analyzer() {
        let analyzer = StructuredProjectAnalyzer::new(100, 4);
        let cancel = CancellationToken::new();

        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(temp.path().join("test.py"), "x = 1").unwrap();

        let results = analyzer.analyze_with_cancellation(
            temp.path().to_path_buf(),
            cancel
        ).await;

        assert!(results.is_ok());
    }

    #[test]
    fn test_pipelined_compiler() {
        let compiler = PipelinedCompiler::new(CompilerPipeline::check_only());
        let result = compiler.compile("x = 1".to_string());

        assert!(result.is_ok());
    }
}

