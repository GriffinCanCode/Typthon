//! Incremental type checking with dependency tracking
//!
//! This module implements fine-grained incremental checking:
//! - Content-addressed hashing for change detection
//! - Dependency graph for invalidation
//! - Query-based memoization

use blake3::Hasher;
use dashmap::{DashMap, DashSet};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// Unique identifier for a module
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleId(u64);

impl ModuleId {
    pub fn from_path(path: &Path) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(path.to_string_lossy().as_bytes());
        let hash = hasher.finalize();
        Self(u64::from_le_bytes(hash.as_bytes()[..8].try_into().unwrap()))
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn as_str(&self) -> String {
        format!("{:016x}", self.0)
    }
}

/// Content hash for change detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash([u8; 32]);

impl ContentHash {
    pub fn from_content(content: &[u8]) -> Self {
        let hash = blake3::hash(content);
        Self(*hash.as_bytes())
    }

    pub fn from_str(content: &str) -> Self {
        Self::from_content(content.as_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Module metadata for incremental checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMetadata {
    pub id: ModuleId,
    pub path: PathBuf,
    pub hash: ContentHash,
    pub timestamp: u64,
    pub imports: Vec<ModuleId>,
}

/// Dependency graph tracking module dependencies
pub struct DependencyGraph {
    /// Module -> modules it depends on
    dependencies: DashMap<ModuleId, HashSet<ModuleId>>,

    /// Module -> modules that depend on it
    dependents: DashMap<ModuleId, HashSet<ModuleId>>,

    /// Module -> content hash
    hashes: DashMap<ModuleId, ContentHash>,

    /// Module -> metadata
    metadata: DashMap<ModuleId, ModuleMetadata>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            dependencies: DashMap::new(),
            dependents: DashMap::new(),
            hashes: DashMap::new(),
            metadata: DashMap::new(),
        }
    }

    /// Add a module to the graph
    pub fn add_module(&self, meta: ModuleMetadata) {
        let id = meta.id;
        let hash = meta.hash;

        // Add dependencies
        let imports = meta.imports.clone();
        if !imports.is_empty() {
            self.dependencies.insert(id, imports.iter().copied().collect());

            // Update reverse dependencies
            for dep in &imports {
                self.dependents.entry(*dep).or_default().insert(id);
            }
        }

        self.hashes.insert(id, hash);
        self.metadata.insert(id, meta);
    }

    /// Check if a module has changed
    pub fn has_changed(&self, id: ModuleId, new_hash: ContentHash) -> bool {
        self.hashes.get(&id).map_or(true, |h| *h != new_hash)
    }

    /// Get all modules that need to be rechecked due to changes
    pub fn invalidate(&self, changed: &[ModuleId]) -> HashSet<ModuleId> {
        let mut invalid = HashSet::new();
        let mut worklist: Vec<ModuleId> = changed.to_vec();

        while let Some(id) = worklist.pop() {
            if invalid.insert(id) {
                // Add all dependents to worklist
                if let Some(deps) = self.dependents.get(&id) {
                    worklist.extend(deps.iter());
                }
            }
        }

        invalid
    }

    /// Get modules in dependency layers for parallel processing
    pub fn dependency_layers(&self) -> Vec<Vec<ModuleId>> {
        let mut layers = Vec::new();
        let mut processed: HashSet<ModuleId> = HashSet::new();
        let mut current_layer = Vec::new();

        // Find modules with no dependencies (layer 0)
        for entry in self.metadata.iter() {
            let id = entry.key();
            let deps = self.dependencies.get(id);
            if deps.is_none() || deps.unwrap().is_empty() {
                current_layer.push(*id);
            }
        }

        while !current_layer.is_empty() {
            processed.extend(&current_layer);
            layers.push(current_layer.clone());

            // Find next layer: modules whose dependencies are all processed
            let mut next_layer = Vec::new();
            for entry in self.metadata.iter() {
                let id = *entry.key();
                if processed.contains(&id) {
                    continue;
                }

                if let Some(deps) = self.dependencies.get(&id) {
                    if deps.iter().all(|d| processed.contains(d)) {
                        next_layer.push(id);
                    }
                }
            }

            current_layer = next_layer;
        }

        layers
    }

    /// Get module metadata
    pub fn get_metadata(&self, id: ModuleId) -> Option<ModuleMetadata> {
        self.metadata.get(&id).map(|m| m.clone())
    }

    /// Update module hash
    pub fn update_hash(&self, id: ModuleId, hash: ContentHash) {
        self.hashes.insert(id, hash);
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Incremental engine coordinating incremental checking
pub struct IncrementalEngine {
    /// Dependency graph
    graph: Arc<DependencyGraph>,

    /// Modules that have changed
    changed: DashSet<ModuleId>,

    /// Enable incremental checking
    enabled: bool,
}

impl IncrementalEngine {
    pub fn new(graph: Arc<DependencyGraph>) -> Self {
        Self {
            graph,
            changed: DashSet::new(),
            enabled: true,
        }
    }

    /// Mark a module as changed
    pub fn mark_changed(&self, id: ModuleId) {
        self.changed.insert(id);
    }

    /// Get all modules that need rechecking
    pub fn get_invalid_modules(&self) -> HashSet<ModuleId> {
        if !self.enabled {
            // If incremental disabled, recheck everything
            self.graph.metadata.iter()
                .map(|entry| *entry.key())
                .collect()
        } else {
            let changed: Vec<ModuleId> = self.changed.iter()
                .map(|r| *r.key())
                .collect();
            self.graph.invalidate(&changed)
        }
    }

    /// Clear changed set after processing
    pub fn clear_changed(&self) {
        self.changed.clear();
    }

    /// Check if a file needs reanalysis
    pub fn needs_recheck(&self, path: &Path) -> bool {
        if !self.enabled {
            return true;
        }

        let id = ModuleId::from_path(path);

        // Read file content
        let Ok(content) = std::fs::read_to_string(path) else {
            return true;
        };

        let new_hash = ContentHash::from_str(&content);
        self.graph.has_changed(id, new_hash)
    }

    /// Register a module with its dependencies
    pub fn register_module(&self, path: PathBuf, content: &str, imports: Vec<PathBuf>) {
        let id = ModuleId::from_path(&path);
        let hash = ContentHash::from_str(content);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let import_ids = imports.iter()
            .map(|p| ModuleId::from_path(p))
            .collect();

        let meta = ModuleMetadata {
            id,
            path,
            hash,
            timestamp,
            imports: import_ids,
        };

        self.graph.add_module(meta);
    }

    /// Get dependency layers for parallel analysis
    pub fn get_layers(&self) -> Vec<Vec<ModuleId>> {
        self.graph.dependency_layers()
    }

    /// Enable/disable incremental checking
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash() {
        let content1 = "def foo(): pass";
        let content2 = "def foo(): pass";
        let content3 = "def bar(): pass";

        let hash1 = ContentHash::from_str(content1);
        let hash2 = ContentHash::from_str(content2);
        let hash3 = ContentHash::from_str(content3);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_dependency_graph() {
        let graph = DependencyGraph::new();

        let id_a = ModuleId(1);
        let id_b = ModuleId(2);
        let id_c = ModuleId(3);

        // A <- B <- C (C depends on B, B depends on A)
        graph.add_module(ModuleMetadata {
            id: id_a,
            path: PathBuf::from("a.py"),
            hash: ContentHash::from_str("a"),
            timestamp: 0,
            imports: vec![],
        });

        graph.add_module(ModuleMetadata {
            id: id_b,
            path: PathBuf::from("b.py"),
            hash: ContentHash::from_str("b"),
            timestamp: 0,
            imports: vec![id_a],
        });

        graph.add_module(ModuleMetadata {
            id: id_c,
            path: PathBuf::from("c.py"),
            hash: ContentHash::from_str("c"),
            timestamp: 0,
            imports: vec![id_b],
        });

        // Changing A should invalidate B and C
        let invalid = graph.invalidate(&[id_a]);
        assert_eq!(invalid.len(), 3);
        assert!(invalid.contains(&id_a));
        assert!(invalid.contains(&id_b));
        assert!(invalid.contains(&id_c));
    }

    #[test]
    fn test_dependency_layers() {
        let graph = DependencyGraph::new();

        let id_a = ModuleId(1);
        let id_b = ModuleId(2);
        let id_c = ModuleId(3);

        graph.add_module(ModuleMetadata {
            id: id_a,
            path: PathBuf::from("a.py"),
            hash: ContentHash::from_str("a"),
            timestamp: 0,
            imports: vec![],
        });

        graph.add_module(ModuleMetadata {
            id: id_b,
            path: PathBuf::from("b.py"),
            hash: ContentHash::from_str("b"),
            timestamp: 0,
            imports: vec![id_a],
        });

        graph.add_module(ModuleMetadata {
            id: id_c,
            path: PathBuf::from("c.py"),
            hash: ContentHash::from_str("c"),
            timestamp: 0,
            imports: vec![id_b],
        });

        let layers = graph.dependency_layers();
        assert_eq!(layers.len(), 3);
        assert_eq!(layers[0], vec![id_a]);
        assert_eq!(layers[1], vec![id_b]);
        assert_eq!(layers[2], vec![id_c]);
    }
}

