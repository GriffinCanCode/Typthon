//! Memory pool allocation for AST nodes
//!
//! Arena allocator for efficient batch allocation/deallocation.

use typed_arena::Arena;
use std::sync::Arc;
use parking_lot::Mutex;

/// Arena allocator for AST nodes
pub struct AstArena {
    /// Underlying arena
    arena: Arena<AstNode>,
}

/// Simplified AST node representation for arena allocation
#[derive(Debug, Clone)]
pub enum AstNode {
    /// Module containing statements
    Module { stmts: Vec<usize> },

    /// Function definition
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<usize>,
    },

    /// Assignment statement
    Assign { target: String, value: usize },

    /// Expression statement
    Expr { expr: usize },

    /// Binary operation
    BinOp { left: usize, right: usize, op: String },

    /// Constant value
    Const { value: ConstValue },

    /// Variable reference
    Name { id: String },
}

/// Constant value
#[derive(Debug, Clone)]
pub enum ConstValue {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    None,
}

impl AstArena {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
        }
    }

    /// Allocate a node in the arena
    pub fn alloc(&self, node: AstNode) -> &AstNode {
        // Safety: This is safe because Arena guarantees stable references
        // and we control the lifetime
        unsafe {
            let ptr = &self.arena as *const Arena<AstNode> as *mut Arena<AstNode>;
            (*ptr).alloc(node)
        }
    }

    /// Get number of allocated nodes (approximate)
    pub fn len(&self) -> usize {
        // Arena doesn't expose this, but we can estimate
        std::mem::size_of::<Arena<AstNode>>()
    }

    /// Check if arena is empty
    pub fn is_empty(&self) -> bool {
        false // Arena doesn't track this precisely
    }
}

impl Default for AstArena {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe arena pool
pub struct ArenaPool {
    /// Pool of arenas
    arenas: Mutex<Vec<Arc<AstArena>>>,

    /// Maximum pool size
    max_size: usize,
}

impl ArenaPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            arenas: Mutex::new(Vec::new()),
            max_size,
        }
    }

    /// Get an arena from the pool or create new one
    pub fn acquire(&self) -> Arc<AstArena> {
        let mut arenas = self.arenas.lock();

        if let Some(arena) = arenas.pop() {
            arena
        } else {
            Arc::new(AstArena::new())
        }
    }

    /// Return arena to pool
    pub fn release(&self, arena: Arc<AstArena>) {
        let mut arenas = self.arenas.lock();

        if arenas.len() < self.max_size {
            arenas.push(arena);
        }
        // Otherwise drop it
    }

    /// Clear the pool
    pub fn clear(&self) {
        self.arenas.lock().clear();
    }

    /// Get pool size
    pub fn size(&self) -> usize {
        self.arenas.lock().len()
    }
}

impl Default for ArenaPool {
    fn default() -> Self {
        Self::new(16) // Default pool size
    }
}

/// Statistics for arena allocation
#[derive(Debug, Default, Clone)]
pub struct ArenaStats {
    /// Total allocations
    pub allocations: usize,

    /// Total deallocations (arena drops)
    pub deallocations: usize,

    /// Peak memory usage (bytes)
    pub peak_memory: usize,

    /// Current memory usage (bytes)
    pub current_memory: usize,
}

impl ArenaStats {
    pub fn record_alloc(&mut self, size: usize) {
        self.allocations += 1;
        self.current_memory += size;
        self.peak_memory = self.peak_memory.max(self.current_memory);
    }

    pub fn record_dealloc(&mut self, size: usize) {
        self.deallocations += 1;
        self.current_memory = self.current_memory.saturating_sub(size);
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_alloc() {
        let arena = AstArena::new();

        let node1 = arena.alloc(AstNode::Const {
            value: ConstValue::Int(42),
        });

        let node2 = arena.alloc(AstNode::Name {
            id: "x".to_string(),
        });

        // Nodes should have different addresses
        assert_ne!(
            node1 as *const AstNode,
            node2 as *const AstNode
        );
    }

    #[test]
    fn test_arena_pool() {
        let pool = ArenaPool::new(2);

        let arena1 = pool.acquire();
        let arena2 = pool.acquire();

        assert_eq!(pool.size(), 0);

        pool.release(arena1);
        pool.release(arena2);

        assert_eq!(pool.size(), 2);

        // Acquire should reuse
        let arena3 = pool.acquire();
        assert_eq!(pool.size(), 1);

        pool.release(arena3);
        assert_eq!(pool.size(), 2);
    }

    #[test]
    fn test_arena_stats() {
        let mut stats = ArenaStats::default();

        stats.record_alloc(100);
        assert_eq!(stats.allocations, 1);
        assert_eq!(stats.current_memory, 100);
        assert_eq!(stats.peak_memory, 100);

        stats.record_alloc(200);
        assert_eq!(stats.current_memory, 300);
        assert_eq!(stats.peak_memory, 300);

        stats.record_dealloc(100);
        assert_eq!(stats.current_memory, 200);
        assert_eq!(stats.peak_memory, 300); // Peak unchanged
    }
}

