//! Async I/O for file operations
//!
//! Provides async file reading/writing optimized for compiler workloads.
//! Uses buffering and batch operations for efficiency.

use std::path::{Path, PathBuf};
use std::io;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use dashmap::DashMap;
use std::sync::Arc;

/// Async file cache for frequently accessed files
pub struct FileCache {
    cache: Arc<DashMap<PathBuf, Arc<String>>>,
    max_size: usize,
}

impl FileCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            max_size,
        }
    }

    /// Read file with caching
    pub async fn read(&self, path: impl AsRef<Path>) -> io::Result<Arc<String>> {
        let path = path.as_ref().to_path_buf();

        // Check cache first
        if let Some(content) = self.cache.get(&path) {
            return Ok(content.clone());
        }

        // Read from disk
        let content = fs::read_to_string(&path).await?;
        let content = Arc::new(content);

        // Cache if under size limit
        if self.cache.len() < self.max_size {
            self.cache.insert(path, content.clone());
        }

        Ok(content)
    }

    /// Invalidate cached file
    pub fn invalidate(&self, path: impl AsRef<Path>) {
        self.cache.remove(path.as_ref());
    }

    /// Clear entire cache
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

/// Batch file reader for parallel loading
pub struct BatchFileReader {
    cache: FileCache,
    concurrency: usize,
}

impl BatchFileReader {
    pub fn new(cache_size: usize, concurrency: usize) -> Self {
        Self {
            cache: FileCache::new(cache_size),
            concurrency,
        }
    }

    /// Read multiple files concurrently
    pub async fn read_batch(&self, paths: Vec<PathBuf>) -> Vec<(PathBuf, io::Result<Arc<String>>)> {
        use futures::stream::{self, StreamExt};

        stream::iter(paths)
            .map(|path| {
                let cache = &self.cache;
                async move {
                    let result = cache.read(&path).await;
                    (path, result)
                }
            })
            .buffer_unordered(self.concurrency)
            .collect()
            .await
    }

    /// Read all Python files in directory
    pub async fn read_directory(&self, root: impl AsRef<Path>) -> io::Result<Vec<(PathBuf, Arc<String>)>> {
        let paths = self.find_python_files(root).await?;
        let results = self.read_batch(paths).await;

        Ok(results.into_iter()
            .filter_map(|(path, result)| result.ok().map(|content| (path, content)))
            .collect())
    }

    /// Find all Python files recursively
    async fn find_python_files(&self, root: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut stack = vec![root.as_ref().to_path_buf()];

        while let Some(path) = stack.pop() {
            if path.is_dir() {
                let mut entries = fs::read_dir(&path).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        stack.push(entry_path);
                    } else if entry_path.extension() == Some(std::ffi::OsStr::new("py")) {
                        files.push(entry_path);
                    }
                }
            }
        }

        Ok(files)
    }

    pub fn cache(&self) -> &FileCache {
        &self.cache
    }
}

/// Async file watcher for incremental compilation
pub struct FileWatcher {
    watched: Arc<DashMap<PathBuf, tokio::time::Instant>>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            watched: Arc::new(DashMap::new()),
        }
    }

    /// Watch file for changes
    pub fn watch(&self, path: impl AsRef<Path>) {
        let path = path.as_ref().to_path_buf();
        self.watched.insert(path, tokio::time::Instant::now());
    }

    /// Check if file has changed since last watch
    pub async fn has_changed(&self, path: impl AsRef<Path>) -> io::Result<bool> {
        let path = path.as_ref();

        let watched_time = self.watched.get(path)
            .map(|entry| *entry.value());

        if let Some(watched) = watched_time {
            let metadata = fs::metadata(path).await?;
            if let Ok(modified) = metadata.modified() {
                let elapsed = modified.duration_since(std::time::UNIX_EPOCH).ok()
                    .and_then(|d| {
                        let watched_unix = std::time::UNIX_EPOCH + watched.elapsed();
                        modified.duration_since(watched_unix).ok()
                    });

                return Ok(elapsed.is_some());
            }
        }

        Ok(false)
    }

    /// Get all watched files that have changed
    pub async fn get_changed(&self) -> io::Result<Vec<PathBuf>> {
        let mut changed = Vec::new();

        for entry in self.watched.iter() {
            if self.has_changed(entry.key()).await? {
                changed.push(entry.key().clone());
            }
        }

        Ok(changed)
    }

    /// Unwatch file
    pub fn unwatch(&self, path: impl AsRef<Path>) {
        self.watched.remove(path.as_ref());
    }

    /// Clear all watches
    pub fn clear(&self) {
        self.watched.clear();
    }
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Buffered async writer for compilation output
pub struct BufferedWriter {
    buffer: Vec<u8>,
    capacity: usize,
}

impl BufferedWriter {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Write data to buffer
    pub fn write(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    /// Flush buffer to file
    pub async fn flush_to(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        let mut file = fs::File::create(path).await?;
        file.write_all(&self.buffer).await?;
        file.flush().await?;
        self.buffer.clear();
        Ok(())
    }

    /// Get buffer size
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Check if buffer should be flushed
    pub fn should_flush(&self) -> bool {
        self.buffer.len() >= self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_cache() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.py");
        fs::write(&file_path, "x = 1").await.unwrap();

        let cache = FileCache::new(10);

        // First read - cache miss
        let content1 = cache.read(&file_path).await.unwrap();
        assert_eq!(*content1, "x = 1");

        // Second read - cache hit
        let content2 = cache.read(&file_path).await.unwrap();
        assert_eq!(*content2, "x = 1");
        assert!(Arc::ptr_eq(&content1, &content2));
    }

    #[tokio::test]
    async fn test_batch_reader() {
        let temp = TempDir::new().unwrap();

        for i in 0..5 {
            let path = temp.path().join(format!("test{}.py", i));
            fs::write(&path, format!("x = {}", i)).await.unwrap();
        }

        let reader = BatchFileReader::new(10, 4);
        let files = reader.read_directory(temp.path()).await.unwrap();

        assert_eq!(files.len(), 5);
    }

    #[tokio::test]
    async fn test_buffered_writer() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("output.txt");

        let mut writer = BufferedWriter::new(1024);
        writer.write(b"Hello, ");
        writer.write(b"World!");

        writer.flush_to(&file_path).await.unwrap();

        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "Hello, World!");
    }
}

