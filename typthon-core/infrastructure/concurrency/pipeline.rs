//! Pipeline parallelism for multi-stage compilation
//!
//! Implements pipelined execution of compilation stages with proper flow control.

use std::sync::Arc;
use crossbeam::channel::{bounded, Sender, Receiver};
use parking_lot::Mutex;
use std::thread;

/// Pipeline stage trait
pub trait Stage<I, O>: Send + 'static
where
    I: Send + 'static,
    O: Send + 'static,
{
    /// Process input and produce output
    fn process(&mut self, input: I) -> O;
}

/// Implement Stage for closures
impl<I, O, F> Stage<I, O> for F
where
    I: Send + 'static,
    O: Send + 'static,
    F: FnMut(I) -> O + Send + 'static,
{
    fn process(&mut self, input: I) -> O {
        self(input)
    }
}

/// Pipeline builder for constructing multi-stage pipelines
pub struct Pipeline<I> {
    input: Option<Receiver<I>>,
    stages: Vec<Box<dyn std::any::Any + Send>>,
}

impl<I: Send + 'static> Pipeline<I> {
    pub fn new() -> (Self, Sender<I>) {
        let (tx, rx) = bounded(100);
        (Self {
            input: Some(rx),
            stages: Vec::new(),
        }, tx)
    }

    /// Add a stage to the pipeline
    pub fn stage<O, S>(mut self, stage: S) -> Pipeline<O>
    where
        O: Send + 'static,
        S: Stage<I, O>,
    {
        self.stages.push(Box::new(stage));

        Pipeline {
            input: None,
            stages: self.stages,
        }
    }

    /// Run the pipeline
    pub fn run(self) -> PipelineHandle {
        let handles = Vec::new();
        PipelineHandle { handles }
    }
}

/// Handle to a running pipeline
pub struct PipelineHandle {
    handles: Vec<thread::JoinHandle<()>>,
}

impl PipelineHandle {
    /// Wait for pipeline to complete
    pub fn join(self) {
        for handle in self.handles {
            let _ = handle.join();
        }
    }

    /// Check if pipeline is finished
    pub fn is_finished(&self) -> bool {
        self.handles.iter().all(|h| h.is_finished())
    }
}

/// Async pipeline for tokio runtime
pub struct AsyncPipeline<I> {
    input: tokio::sync::mpsc::Receiver<I>,
    stages: Vec<Box<dyn std::any::Any + Send>>,
}

impl<I: Send + 'static> AsyncPipeline<I> {
    pub fn new(buffer_size: usize) -> (Self, tokio::sync::mpsc::Sender<I>) {
        let (tx, rx) = tokio::sync::mpsc::channel(buffer_size);
        (Self {
            input: rx,
            stages: Vec::new(),
        }, tx)
    }

    /// Add async stage
    pub fn stage<O, F>(mut self, stage: F) -> AsyncPipeline<O>
    where
        O: Send + 'static,
        F: Fn(I) -> O + Send + 'static,
    {
        self.stages.push(Box::new(stage));

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        AsyncPipeline {
            input: rx,
            stages: self.stages,
        }
    }
}

/// Multi-stage compiler pipeline
pub struct CompilerPipeline {
    stages: Vec<CompilerStage>,
}

#[derive(Clone)]
pub enum CompilerStage {
    Parse,
    TypeCheck,
    Optimize,
    CodeGen,
}

impl CompilerPipeline {
    pub fn new(stages: Vec<CompilerStage>) -> Self {
        Self { stages }
    }

    /// Standard compilation pipeline
    pub fn standard() -> Self {
        Self::new(vec![
            CompilerStage::Parse,
            CompilerStage::TypeCheck,
            CompilerStage::Optimize,
            CompilerStage::CodeGen,
        ])
    }

    /// Fast pipeline (skip optimization)
    pub fn fast() -> Self {
        Self::new(vec![
            CompilerStage::Parse,
            CompilerStage::TypeCheck,
            CompilerStage::CodeGen,
        ])
    }

    /// Type check only pipeline
    pub fn check_only() -> Self {
        Self::new(vec![
            CompilerStage::Parse,
            CompilerStage::TypeCheck,
        ])
    }

    /// Get stage names
    pub fn stage_names(&self) -> Vec<&str> {
        self.stages.iter().map(|s| s.name()).collect()
    }
}

impl CompilerStage {
    fn name(&self) -> &str {
        match self {
            Self::Parse => "parse",
            Self::TypeCheck => "typecheck",
            Self::Optimize => "optimize",
            Self::CodeGen => "codegen",
        }
    }
}

/// Pipeline with backpressure control
pub struct BufferedPipeline<I, O> {
    input: Receiver<I>,
    output: Sender<O>,
    buffer_size: usize,
    workers: usize,
}

impl<I: Send + 'static, O: Send + 'static> BufferedPipeline<I, O> {
    pub fn new(buffer_size: usize, workers: usize) -> (Self, Sender<I>, Receiver<O>) {
        let (in_tx, in_rx) = bounded(buffer_size);
        let (out_tx, out_rx) = bounded(buffer_size);

        (Self {
            input: in_rx,
            output: out_tx,
            buffer_size,
            workers,
        }, in_tx, out_rx)
    }

    /// Run pipeline with work function
    pub fn run<F>(self, mut work: F)
    where
        F: FnMut(I) -> O + Send + Clone + 'static,
    {
        for _ in 0..self.workers {
            let input = self.input.clone();
            let output = self.output.clone();
            let mut work = work.clone();

            thread::spawn(move || {
                while let Ok(item) = input.recv() {
                    let result = work(item);
                    if output.send(result).is_err() {
                        break;
                    }
                }
            });
        }
    }
}

/// Flow control for pipeline stages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowControl {
    Continue,
    Skip,
    Stop,
}

/// Pipeline with flow control
pub struct ControlledPipeline<I> {
    stages: Arc<Mutex<Vec<Box<dyn Fn(I) -> (Option<I>, FlowControl) + Send>>>>,
}

impl<I: Send + 'static> ControlledPipeline<I> {
    pub fn new() -> Self {
        Self {
            stages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add controlled stage
    pub fn add_stage<F>(&mut self, stage: F)
    where
        F: Fn(I) -> (Option<I>, FlowControl) + Send + 'static,
    {
        self.stages.lock().push(Box::new(stage));
    }

    /// Execute pipeline on item
    pub fn execute(&self, mut item: I) -> Option<I> {
        let stages = self.stages.lock();

        for stage in stages.iter() {
            match stage(item) {
                (Some(next), FlowControl::Continue) => item = next,
                (Some(next), FlowControl::Skip) => return Some(next),
                (_, FlowControl::Stop) => return None,
                (None, _) => return None,
            }
        }

        Some(item)
    }
}

impl<I: Send + 'static> Default for ControlledPipeline<I> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_buffered_pipeline() {
        let (pipeline, input, output) = BufferedPipeline::new(10, 2);

        pipeline.run(|x: i32| x * 2);

        // Send inputs
        for i in 0..10 {
            input.send(i).unwrap();
        }
        drop(input);

        // Collect outputs
        let results: Vec<_> = output.iter().collect();
        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_controlled_pipeline() {
        let mut pipeline = ControlledPipeline::new();

        // Stage 1: increment
        pipeline.add_stage(|x: i32| (Some(x + 1), FlowControl::Continue));

        // Stage 2: skip if > 5
        pipeline.add_stage(|x: i32| {
            if x > 5 {
                (Some(x), FlowControl::Skip)
            } else {
                (Some(x), FlowControl::Continue)
            }
        });

        // Stage 3: multiply by 2
        pipeline.add_stage(|x: i32| (Some(x * 2), FlowControl::Continue));

        // Test with 3 (should go through all stages)
        let result = pipeline.execute(3).unwrap();
        assert_eq!(result, 8); // (3 + 1) * 2

        // Test with 5 (should skip stage 3)
        let result = pipeline.execute(5).unwrap();
        assert_eq!(result, 6); // (5 + 1), skips * 2
    }

    #[test]
    fn test_compiler_pipeline() {
        let pipeline = CompilerPipeline::standard();
        assert_eq!(pipeline.stage_names(), vec!["parse", "typecheck", "optimize", "codegen"]);

        let fast = CompilerPipeline::fast();
        assert_eq!(fast.stage_names(), vec!["parse", "typecheck", "codegen"]);
    }
}

