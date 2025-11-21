//! Performance metrics and monitoring

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Performance metrics collector
pub struct PerformanceMetrics {
    /// Individual timing measurements
    timings: RwLock<HashMap<String, Vec<Duration>>>,

    /// Counter metrics
    counters: RwLock<HashMap<String, u64>>,

    /// Start time for uptime
    start_time: Instant,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            timings: RwLock::new(HashMap::new()),
            counters: RwLock::new(HashMap::new()),
            start_time: Instant::now(),
        }
    }

    /// Record a timing measurement
    pub fn record_timing(&self, name: impl Into<String>, duration: Duration) {
        let mut timings = self.timings.write();
        timings.entry(name.into()).or_default().push(duration);
    }

    /// Increment a counter
    pub fn increment(&self, name: impl Into<String>) {
        let mut counters = self.counters.write();
        *counters.entry(name.into()).or_default() += 1;
    }

    /// Add to a counter
    pub fn add(&self, name: impl Into<String>, value: u64) {
        let mut counters = self.counters.write();
        *counters.entry(name.into()).or_default() += value;
    }

    /// Get statistics for a timing metric
    pub fn get_timing_stats(&self, name: &str) -> Option<TimingStats> {
        let timings = self.timings.read();
        timings.get(name).map(|durations| {
            TimingStats::from_durations(durations)
        })
    }

    /// Get counter value
    pub fn get_counter(&self, name: &str) -> u64 {
        self.counters.read().get(name).copied().unwrap_or(0)
    }

    /// Get all timing names
    pub fn timing_names(&self) -> Vec<String> {
        self.timings.read().keys().cloned().collect()
    }

    /// Get all counter names
    pub fn counter_names(&self) -> Vec<String> {
        self.counters.read().keys().cloned().collect()
    }

    /// Get uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.timings.write().clear();
        self.counters.write().clear();
    }

    /// Generate summary report
    pub fn summary(&self) -> MetricsSummary {
        let timings = self.timings.read();
        let counters = self.counters.read();

        let timing_stats: HashMap<String, TimingStats> = timings
            .iter()
            .map(|(name, durations)| {
                (name.clone(), TimingStats::from_durations(durations))
            })
            .collect();

        MetricsSummary {
            uptime: self.uptime(),
            timings: timing_stats,
            counters: counters.clone(),
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for timing measurements
#[derive(Debug, Clone)]
pub struct TimingStats {
    pub count: usize,
    pub total: Duration,
    pub mean: Duration,
    pub min: Duration,
    pub max: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
}

impl TimingStats {
    fn from_durations(durations: &[Duration]) -> Self {
        if durations.is_empty() {
            return Self {
                count: 0,
                total: Duration::ZERO,
                mean: Duration::ZERO,
                min: Duration::ZERO,
                max: Duration::ZERO,
                p50: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
            };
        }

        let mut sorted = durations.to_vec();
        sorted.sort();

        let count = sorted.len();
        let total: Duration = sorted.iter().sum();
        let mean = total / count as u32;
        let min = sorted[0];
        let max = sorted[count - 1];

        let percentile = |p: f64| {
            let idx = ((count as f64 * p) as usize).min(count - 1);
            sorted[idx]
        };

        Self {
            count,
            total,
            mean,
            min,
            max,
            p50: percentile(0.50),
            p95: percentile(0.95),
            p99: percentile(0.99),
        }
    }
}

/// Summary of all metrics
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub uptime: Duration,
    pub timings: HashMap<String, TimingStats>,
    pub counters: HashMap<String, u64>,
}

impl MetricsSummary {
    /// Format as human-readable report
    pub fn report(&self) -> String {
        let mut lines = vec![
            format!("Uptime: {:.2?}", self.uptime),
            String::new(),
            "=== Timings ===".to_string(),
        ];

        for (name, stats) in &self.timings {
            lines.push(format!("{}:", name));
            lines.push(format!("  count: {}", stats.count));
            lines.push(format!("  total: {:.2?}", stats.total));
            lines.push(format!("  mean:  {:.2?}", stats.mean));
            lines.push(format!("  min:   {:.2?}", stats.min));
            lines.push(format!("  max:   {:.2?}", stats.max));
            lines.push(format!("  p50:   {:.2?}", stats.p50));
            lines.push(format!("  p95:   {:.2?}", stats.p95));
            lines.push(format!("  p99:   {:.2?}", stats.p99));
        }

        lines.push(String::new());
        lines.push("=== Counters ===".to_string());

        for (name, value) in &self.counters {
            lines.push(format!("{}: {}", name, value));
        }

        lines.join("\n")
    }
}

/// RAII timer for automatic timing measurement
pub struct Timer<'a> {
    metrics: &'a PerformanceMetrics,
    name: String,
    start: Instant,
}

impl<'a> Timer<'a> {
    pub fn new(metrics: &'a PerformanceMetrics, name: impl Into<String>) -> Self {
        Self {
            metrics,
            name: name.into(),
            start: Instant::now(),
        }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.metrics.record_timing(&self.name, duration);
    }
}

/// Global metrics instance
pub fn global_metrics() -> Arc<PerformanceMetrics> {
    use once_cell::sync::Lazy;
    static METRICS: Lazy<Arc<PerformanceMetrics>> = Lazy::new(|| {
        Arc::new(PerformanceMetrics::new())
    });
    METRICS.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_timing() {
        let metrics = PerformanceMetrics::new();

        metrics.record_timing("test", Duration::from_millis(100));
        metrics.record_timing("test", Duration::from_millis(200));
        metrics.record_timing("test", Duration::from_millis(150));

        let stats = metrics.get_timing_stats("test").unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.min, Duration::from_millis(100));
        assert_eq!(stats.max, Duration::from_millis(200));
    }

    #[test]
    fn test_counter() {
        let metrics = PerformanceMetrics::new();

        metrics.increment("requests");
        metrics.increment("requests");
        metrics.add("bytes", 1000);

        assert_eq!(metrics.get_counter("requests"), 2);
        assert_eq!(metrics.get_counter("bytes"), 1000);
    }

    #[test]
    fn test_timer() {
        let metrics = PerformanceMetrics::new();

        {
            let _timer = Timer::new(&metrics, "sleep");
            thread::sleep(Duration::from_millis(10));
        }

        let stats = metrics.get_timing_stats("sleep").unwrap();
        assert_eq!(stats.count, 1);
        assert!(stats.total >= Duration::from_millis(10));
    }

    #[test]
    fn test_summary() {
        let metrics = PerformanceMetrics::new();

        metrics.record_timing("parse", Duration::from_millis(50));
        metrics.increment("files");

        let summary = metrics.summary();
        assert!(!summary.timings.is_empty());
        assert!(!summary.counters.is_empty());

        let report = summary.report();
        assert!(report.contains("parse"));
        assert!(report.contains("files"));
    }
}

