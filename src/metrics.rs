//! Performance metrics collection and reporting for the ZK Circuit Fuzzer.
//!
//! This module provides comprehensive metrics tracking for fuzzing campaigns,
//! including execution statistics, coverage data, and performance counters.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Core metrics collector for tracking fuzzing campaign statistics.
#[derive(Debug, Default)]
pub struct MetricsCollector {
    /// Total number of test cases executed
    total_executions: AtomicU64,
    /// Number of unique code paths discovered
    unique_paths: AtomicU64,
    /// Number of crashes detected
    crashes: AtomicU64,
    /// Number of timeouts encountered
    timeouts: AtomicU64,
    /// Number of hangs detected
    hangs: AtomicU64,
    /// Start time of the fuzzing campaign
    start_time: Instant,
    /// Whether metrics collection is active
    active: AtomicBool,
    /// Coverage map: path hash -> execution count
    coverage_map: parking_lot::RwLock<HashMap<u64, u64>>,
    /// Performance counters per circuit type
    circuit_stats: parking_lot::RwLock<HashMap<String, CircuitStats>>,
}

/// Statistics for a specific circuit type.
#[derive(Debug, Clone, Default)]
pub struct CircuitStats {
    /// Number of executions for this circuit
    pub executions: u64,
    /// Average execution time in microseconds
    pub avg_time_us: f64,
    /// Minimum execution time in microseconds
    pub min_time_us: u64,
    /// Maximum execution time in microseconds
    pub max_time_us: u64,
    /// Number of unique paths found
    pub unique_paths: u64,
    /// Number of crashes found
    pub crashes: u64,
}

impl MetricsCollector {
    /// Creates a new metrics collector.
    pub fn new() -> Self {
        Self {
            total_executions: AtomicU64::new(0),
            unique_paths: AtomicU64::new(0),
            crashes: AtomicU64::new(0),
            timeouts: AtomicU64::new(0),
            hangs: AtomicU64::new(0),
            start_time: Instant::now(),
            active: AtomicBool::new(true),
            coverage_map: parking_lot::RwLock::new(HashMap::new()),
            circuit_stats: parking_lot::RwLock::new(HashMap::new()),
        }
    }

    /// Creates an Arc-wrapped metrics collector for shared access.
    pub fn shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Records a successful test execution.
    pub fn record_execution(&self, circuit_name: &str, duration: Duration, path_hash: u64) {
        self.total_executions.fetch_add(1, Ordering::Relaxed);
        
        // Update coverage map
        {
            let mut map = self.coverage_map.write();
            *map.entry(path_hash).or_insert(0) += 1;
            self.unique_paths.store(map.len() as u64, Ordering::Relaxed);
        }

        // Update circuit-specific stats
        self.update_circuit_stats(circuit_name, duration.as_micros() as u64);
    }

    /// Records a crash detection.
    pub fn record_crash(&self, circuit_name: &str) {
        self.crashes.fetch_add(1, Ordering::Relaxed);
        self.increment_circuit_crashes(circuit_name);
    }

    /// Records a timeout.
    pub fn record_timeout(&self) {
        self.timeouts.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a hang detection.
    pub fn record_hang(&self) {
        self.hangs.fetch_add(1, Ordering::Relaxed);
    }

    /// Returns the total number of executions.
    pub fn total_executions(&self) -> u64 {
        self.total_executions.load(Ordering::Relaxed)
    }

    /// Returns the number of unique paths discovered.
    pub fn unique_paths(&self) -> u64 {
        self.unique_paths.load(Ordering::Relaxed)
    }

    /// Returns the number of crashes detected.
    pub fn crashes(&self) -> u64 {
        self.crashes.load(Ordering::Relaxed)
    }

    /// Returns the elapsed time since the collector was created.
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Returns executions per second.
    pub fn execs_per_second(&self) -> f64 {
        let elapsed = self.elapsed();
        if elapsed.as_secs_f64() > 0.0 {
            self.total_executions() as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Returns a snapshot of current metrics.
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_executions: self.total_executions(),
            unique_paths: self.unique_paths(),
            crashes: self.crashes(),
            timeouts: self.timeouts.load(Ordering::Relaxed),
            hangs: self.hangs.load(Ordering::Relaxed),
            elapsed: self.elapsed(),
            execs_per_second: self.execs_per_second(),
            coverage_entries: self.coverage_map.read().len(),
        }
    }

    /// Returns circuit-specific statistics.
    pub fn circuit_stats(&self) -> HashMap<String, CircuitStats> {
        self.circuit_stats.read().clone()
    }

    fn update_circuit_stats(&self, circuit_name: &str, duration_us: u64) {
        let mut stats = self.circuit_stats.write();
        let entry = stats.entry(circuit_name.to_string()).or_default();
        entry.executions += 1;
        
        // Update running average
        entry.avg_time_us = ((entry.avg_time_us * (entry.executions - 1) as f64) + duration_us as f64) 
            / entry.executions as f64;
        
        entry.min_time_us = if entry.min_time_us == 0 { duration_us } else { entry.min_time_us.min(duration_us) };
        entry.max_time_us = entry.max_time_us.max(duration_us);
    }

    fn increment_circuit_crashes(&self, circuit_name: &str) {
        let mut stats = self.circuit_stats.write();
        if let Some(entry) = stats.get_mut(circuit_name) {
            entry.crashes += 1;
        }
    }
}

/// A point-in-time snapshot of metrics for reporting.
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub total_executions: u64,
    pub unique_paths: u64,
    pub crashes: u64,
    pub timeouts: u64,
    pub hangs: u64,
    pub elapsed: Duration,
    pub execs_per_second: f64,
    pub coverage_entries: usize,
}

impl std::fmt::Display for MetricsSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Fuzzing Metrics Snapshot ===")?;
        writeln!(f, "Total Executions: {}", self.total_executions)?;
        writeln!(f, "Unique Paths: {}", self.unique_paths)?;
        writeln!(f, "Crashes: {}", self.crashes)?;
        writeln!(f, "Timeouts: {}", self.timeouts)?;
        writeln!(f, "Hangs: {}", self.hangs)?;
        writeln!(f, "Elapsed: {:.2?}", self.elapsed)?;
        writeln!(f, "Execs/sec: {:.2}", self.execs_per_second)?;
        writeln!(f, "Coverage Entries: {}", self.coverage_entries)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_basic() {
        let collector = MetricsCollector::new();
        assert_eq!(collector.total_executions(), 0);
        assert_eq!(collector.unique_paths(), 0);
        assert_eq!(collector.crashes(), 0);

        collector.record_execution("test_circuit", Duration::from_millis(10), 12345);
        assert_eq!(collector.total_executions(), 1);
        assert_eq!(collector.unique_paths(), 1);
    }

    #[test]
    fn test_crash_recording() {
        let collector = MetricsCollector::new();
        collector.record_crash("test_circuit");
        assert_eq!(collector.crashes(), 1);
    }
}
