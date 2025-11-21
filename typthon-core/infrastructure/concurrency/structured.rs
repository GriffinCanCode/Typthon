//! Structured concurrency primitives
//!
//! Provides scoped tasks, nurseries, and proper cancellation propagation.
//! Ensures all spawned tasks complete before scope exits.

use std::future::Future;
use std::sync::Arc;
use parking_lot::Mutex;
use tokio::task::JoinHandle;
use tokio::sync::Semaphore;

/// Scoped task group - all tasks must complete before dropping
pub struct TaskScope {
    handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
    semaphore: Option<Arc<Semaphore>>,
}

impl TaskScope {
    /// Create new task scope
    pub fn new() -> Self {
        Self {
            handles: Arc::new(Mutex::new(Vec::new())),
            semaphore: None,
        }
    }

    /// Create new task scope with concurrency limit
    pub fn with_limit(limit: usize) -> Self {
        Self {
            handles: Arc::new(Mutex::new(Vec::new())),
            semaphore: Some(Arc::new(Semaphore::new(limit))),
        }
    }

    /// Spawn task in scope
    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let handles = self.handles.clone();
        let semaphore = self.semaphore.clone();

        let handle = tokio::spawn(async move {
            let _permit = if let Some(sem) = semaphore {
                Some(sem.acquire().await.unwrap())
            } else {
                None
            };
            future.await;
        });

        self.handles.lock().push(handle);
    }

    /// Wait for all tasks to complete
    pub async fn join_all(&self) {
        let handles = {
            let mut h = self.handles.lock();
            std::mem::take(&mut *h)
        };

        for handle in handles {
            let _ = handle.await;
        }
    }
}

impl Drop for TaskScope {
    fn drop(&mut self) {
        // Abort all remaining tasks on drop
        let handles = self.handles.lock();
        for handle in handles.iter() {
            handle.abort();
        }
    }
}

/// Nursery pattern for structured concurrency
pub struct Nursery {
    scope: TaskScope,
    errors: Arc<Mutex<Vec<Box<dyn std::error::Error + Send>>>>,
}

impl Nursery {
    pub fn new() -> Self {
        Self {
            scope: TaskScope::new(),
            errors: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_limit(limit: usize) -> Self {
        Self {
            scope: TaskScope::with_limit(limit),
            errors: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Spawn task that may fail
    pub fn spawn<F, T, E>(&self, future: F)
    where
        F: Future<Output = Result<T, E>> + Send + 'static,
        T: Send + 'static,
        E: std::error::Error + Send + 'static,
    {
        let errors = self.errors.clone();
        self.scope.spawn(async move {
            if let Err(e) = future.await {
                errors.lock().push(Box::new(e));
            }
        });
    }

    /// Wait for all tasks and return errors if any
    pub async fn join(self) -> Result<(), Vec<Box<dyn std::error::Error + Send>>> {
        self.scope.join_all().await;

        let errors = {
            let mut e = self.errors.lock();
            std::mem::take(&mut *e)
        };

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Cancellation token for coordinated shutdown
pub struct CancellationToken {
    cancelled: Arc<Mutex<bool>>,
    notify: Arc<tokio::sync::Notify>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(Mutex::new(false)),
            notify: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Cancel the token
    pub fn cancel(&self) {
        *self.cancelled.lock() = true;
        self.notify.notify_waiters();
    }

    /// Check if cancelled
    pub fn is_cancelled(&self) -> bool {
        *self.cancelled.lock()
    }

    /// Wait for cancellation
    pub async fn cancelled(&self) {
        if self.is_cancelled() {
            return;
        }
        self.notify.notified().await;
    }

    /// Create child token
    pub fn child(&self) -> Self {
        let child = Self::new();
        let parent = self.clone();
        let child_clone = child.clone();

        tokio::spawn(async move {
            parent.cancelled().await;
            child_clone.cancel();
        });

        child
    }
}

impl Clone for CancellationToken {
    fn clone(&self) -> Self {
        Self {
            cancelled: self.cancelled.clone(),
            notify: self.notify.clone(),
        }
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Run scoped concurrent tasks
pub async fn scoped<F, R>(f: F) -> R
where
    F: FnOnce(&TaskScope) -> R,
{
    let scope = TaskScope::new();
    let result = f(&scope);
    scope.join_all().await;
    result
}

/// Run scoped concurrent tasks with concurrency limit
pub async fn scoped_with_limit<F, R>(limit: usize, f: F) -> R
where
    F: FnOnce(&TaskScope) -> R,
{
    let scope = TaskScope::with_limit(limit);
    let result = f(&scope);
    scope.join_all().await;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_task_scope() {
        let counter = Arc::new(AtomicUsize::new(0));
        let scope = TaskScope::new();

        for _ in 0..10 {
            let counter = counter.clone();
            scope.spawn(async move {
                counter.fetch_add(1, Ordering::SeqCst);
            });
        }

        scope.join_all().await;
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    async fn test_nursery() {
        let nursery = Nursery::new();

        for i in 0..5 {
            nursery.spawn(async move {
                if i == 3 {
                    Err::<(), _>(std::io::Error::new(std::io::ErrorKind::Other, "error"))
                } else {
                    Ok(())
                }
            });
        }

        let result = nursery.join().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().len(), 1);
    }

    #[tokio::test]
    async fn test_cancellation() {
        let token = CancellationToken::new();
        let token_clone = token.clone();

        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            token_clone.cancel();
        });

        token.cancelled().await;
        assert!(token.is_cancelled());
    }
}

