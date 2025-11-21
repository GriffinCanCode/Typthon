//! Persistent cache for analysis results
//!
//! Content-addressed storage with compression and LRU eviction.

use crate::core::types::Type;
use crate::errors::TypeError;
use crate::performance::incremental::{ModuleId, ContentHash};
use dashmap::DashMap;
use lru::LruCache;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read, Write};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Arc;

/// Cache key for lookup
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    pub module: ModuleId,
    pub hash: ContentHash,
}

impl CacheKey {
    pub fn new(module: ModuleId, hash: ContentHash) -> Self {
        Self { module, hash }
    }

    /// Generate filename for this cache entry
    fn filename(&self) -> String {
        let hash_str = hex::encode(&self.hash.as_bytes()[..16]);
        format!("{}_{}.cache", self.module.as_str(), hash_str)
    }
}

/// Cache entry containing analysis results (note: serialization disabled due to Type complexity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Module ID
    pub module: ModuleId,

    /// Content hash
    pub hash: ContentHash,

    /// Inferred types (variable name -> type)
    pub types: Vec<(String, Type)>,

    /// Type errors found
    pub errors: Vec<CachedError>,

    /// Timestamp when cached
    pub timestamp: u64,

    /// Size in bytes (for LRU eviction)
    pub size_bytes: usize,
}

/// Cached error (serializable version of TypeError)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedError {
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub file: String,
}

impl From<&TypeError> for CachedError {
    fn from(error: &TypeError) -> Self {
        Self {
            message: error.kind.to_string(),
            line: error.location.line,
            col: error.location.col,
            file: error.file.clone(),
        }
    }
}

impl From<&crate::analysis::checker::TypeError> for CachedError {
    fn from(error: &crate::analysis::checker::TypeError) -> Self {
        Self {
            message: error.message.clone(),
            line: error.line,
            col: error.col,
            file: String::new(),
        }
    }
}

/// Disk cache with compression
pub struct DiskCache {
    /// Cache directory
    root: PathBuf,

    /// Compression level (1-22)
    compression_level: i32,
}

impl DiskCache {
    pub fn new(root: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&root)?;

        Ok(Self {
            root,
            compression_level: 3, // Fast compression
        })
    }

    /// Get cache file path
    fn cache_path(&self, key: &CacheKey) -> PathBuf {
        self.root.join(key.filename())
    }

    /// Read entry from disk
    pub fn get(&self, key: &CacheKey) -> io::Result<CacheEntry> {
        let path = self.cache_path(key);
        let mut file = fs::File::open(path)?;

        // Read compressed data
        let mut compressed = Vec::new();
        file.read_to_end(&mut compressed)?;

        // Decompress
        let _decompressed = zstd::decode_all(&compressed[..])?;

        // Deserialize (disabled - Type serialization not supported)
        // TODO: Implement custom serialization for Type
        Err(io::Error::new(io::ErrorKind::Unsupported, "Type serialization not implemented"))
    }

    /// Write entry to disk
    pub fn set(&self, key: &CacheKey, entry: &CacheEntry) -> io::Result<()> {
        let path = self.cache_path(key);

        // Serialize
        let serialized = bincode::serialize(entry)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Compress
        let compressed = zstd::encode_all(&serialized[..], self.compression_level)?;

        // Atomic write: write to temp file, then rename
        let temp_path = path.with_extension("tmp");
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(&compressed)?;
        file.sync_all()?;

        fs::rename(temp_path, path)?;
        Ok(())
    }

    /// Delete entry from disk
    pub fn remove(&self, key: &CacheKey) -> io::Result<()> {
        let path = self.cache_path(key);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Get total cache size in bytes
    pub fn total_size(&self) -> io::Result<u64> {
        let mut total = 0u64;
        for entry in fs::read_dir(&self.root)? {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    total += metadata.len();
                }
            }
        }
        Ok(total)
    }

    /// Clear entire cache
    pub fn clear(&self) -> io::Result<()> {
        for entry in fs::read_dir(&self.root)? {
            if let Ok(entry) = entry {
                if entry.path().extension() == Some(std::ffi::OsStr::new("cache")) {
                    fs::remove_file(entry.path())?;
                }
            }
        }
        Ok(())
    }
}

/// LRU eviction policy
pub struct LruPolicy {
    /// LRU cache tracking access order
    lru: LruCache<CacheKey, usize>,

    /// Current total size
    total_size: usize,

    /// Maximum size in bytes
    max_size: usize,
}

impl LruPolicy {
    pub fn new(max_size: usize) -> Self {
        let capacity = NonZeroUsize::new(10000).unwrap();
        Self {
            lru: LruCache::new(capacity),
            total_size: 0,
            max_size,
        }
    }

    /// Record access to cache entry
    pub fn access(&mut self, key: &CacheKey, size: usize) {
        self.lru.put(key.clone(), size);
        self.total_size += size;
    }

    /// Get keys to evict to make room
    pub fn evict_candidates(&mut self, needed: usize) -> Vec<CacheKey> {
        let mut candidates = Vec::new();
        let mut freed = 0usize;

        while freed < needed && self.total_size + needed > self.max_size {
            if let Some((key, size)) = self.lru.pop_lru() {
                candidates.push(key);
                freed += size;
                self.total_size = self.total_size.saturating_sub(size);
            } else {
                break;
            }
        }

        candidates
    }

    /// Remove entry from tracking
    pub fn remove(&mut self, key: &CacheKey) {
        if let Some(size) = self.lru.pop(key) {
            self.total_size = self.total_size.saturating_sub(size);
        }
    }
}

/// Result cache with memory and disk layers
pub struct ResultCache {
    /// In-memory cache
    memory: DashMap<CacheKey, Arc<CacheEntry>>,

    /// Disk storage
    disk: Arc<DiskCache>,

    /// LRU eviction policy
    eviction: Arc<RwLock<LruPolicy>>,

    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
}

/// Cache statistics for monitoring
#[derive(Debug, Default)]
#[derive(Clone, Copy)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub disk_reads: usize,
    pub disk_writes: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

impl ResultCache {
    pub fn new(cache_dir: PathBuf, max_size_mb: usize) -> io::Result<Self> {
        let disk = Arc::new(DiskCache::new(cache_dir)?);
        let max_size = max_size_mb * 1024 * 1024;
        let eviction = Arc::new(RwLock::new(LruPolicy::new(max_size)));

        Ok(Self {
            memory: DashMap::new(),
            disk,
            eviction,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        })
    }

    /// Get entry from cache
    pub fn get(&self, key: &CacheKey) -> Option<Arc<CacheEntry>> {
        // Try memory first
        if let Some(entry) = self.memory.get(key) {
            self.stats.write().hits += 1;

            // Update LRU
            self.eviction.write().access(key, entry.size_bytes);

            return Some(entry.clone());
        }

        // Try disk
        if let Ok(entry) = self.disk.get(key) {
            self.stats.write().hits += 1;
            self.stats.write().disk_reads += 1;

            let entry = Arc::new(entry);

            // Promote to memory
            self.memory.insert(key.clone(), entry.clone());
            self.eviction.write().access(key, entry.size_bytes);

            return Some(entry);
        }

        self.stats.write().misses += 1;
        None
    }

    /// Store entry in cache
    pub fn set(&self, key: CacheKey, entry: CacheEntry) -> io::Result<()> {
        let size = entry.size_bytes;

        // Check if we need to evict
        {
            let mut eviction = self.eviction.write();
            let candidates = eviction.evict_candidates(size);

            for candidate in candidates {
                // Remove from memory
                self.memory.remove(&candidate);

                // Remove from disk
                let _ = self.disk.remove(&candidate);

                self.stats.write().evictions += 1;
            }
        }

        // Add to memory
        let entry = Arc::new(entry);
        self.memory.insert(key.clone(), entry.clone());

        // Write to disk
        self.disk.set(&key, &entry)?;
        self.stats.write().disk_writes += 1;

        // Update LRU
        self.eviction.write().access(&key, size);

        Ok(())
    }

    /// Remove entry from cache
    pub fn remove(&self, key: &CacheKey) -> io::Result<()> {
        self.memory.remove(key);
        self.eviction.write().remove(key);
        self.disk.remove(key)?;
        Ok(())
    }

    /// Clear entire cache
    pub fn clear(&self) -> io::Result<()> {
        self.memory.clear();
        self.eviction.write().lru.clear();
        self.disk.clear()
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        *self.stats.read()
    }

    /// Get cache size
    pub fn size(&self) -> io::Result<u64> {
        self.disk.total_size()
    }
}

// Add hex encoding trait for hash display
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_roundtrip() {
        let temp = TempDir::new().unwrap();
        let cache = ResultCache::new(temp.path().to_path_buf(), 100).unwrap();

        let key = CacheKey {
            module: ModuleId(1),
            hash: ContentHash([0u8; 32]),
        };

        let entry = CacheEntry {
            module: ModuleId(1),
            hash: ContentHash([0u8; 32]),
            types: vec![("x".to_string(), Type::Int)],
            errors: vec![],
            timestamp: 0,
            size_bytes: 100,
        };

        cache.set(key.clone(), entry.clone()).unwrap();

        let retrieved = cache.get(&key).unwrap();
        assert_eq!(retrieved.module, entry.module);
        assert_eq!(retrieved.types.len(), 1);
    }

    #[test]
    fn test_lru_eviction() {
        let mut policy = LruPolicy::new(1000);

        let key1 = CacheKey {
            module: ModuleId(1),
            hash: ContentHash([0u8; 32]),
        };

        let key2 = CacheKey {
            module: ModuleId(2),
            hash: ContentHash([1u8; 32]),
        };

        policy.access(&key1, 600);
        policy.access(&key2, 600);

        // Adding 600 more should evict key1
        let evicted = policy.evict_candidates(600);
        assert_eq!(evicted.len(), 1);
        assert_eq!(evicted[0], key1);
    }

    #[test]
    fn test_cache_stats() {
        let temp = TempDir::new().unwrap();
        let cache = ResultCache::new(temp.path().to_path_buf(), 100).unwrap();

        let key = CacheKey {
            module: ModuleId(1),
            hash: ContentHash([0u8; 32]),
        };

        // Miss
        assert!(cache.get(&key).is_none());
        assert_eq!(cache.stats().misses, 1);

        // Set and hit
        let entry = CacheEntry {
            module: ModuleId(1),
            hash: ContentHash([0u8; 32]),
            types: vec![],
            errors: vec![],
            timestamp: 0,
            size_bytes: 100,
        };
        cache.set(key.clone(), entry).unwrap();

        assert!(cache.get(&key).is_some());
        assert_eq!(cache.stats().hits, 1);
    }
}

