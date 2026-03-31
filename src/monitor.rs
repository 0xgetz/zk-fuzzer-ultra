//! System resource monitoring for the ZK Circuit Fuzzer.
//!
//! This module provides real-time monitoring of system resources including CPU,
//! memory, disk I/O, and network utilization during fuzzing campaigns.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::thread;
use std::io::Read;

/// System resource monitor for tracking performance metrics.
#[derive(Debug)]
pub struct SystemMonitor {
    /// Whether monitoring is active
    active: AtomicBool,
    /// CPU usage percentage (0-100)
    cpu_usage: AtomicU64,
    /// Memory usage in bytes
    memory_used: AtomicU64,
    /// Total system memory in bytes
    total_memory: AtomicU64,
    /// Number of worker threads
    worker_threads: AtomicU64,
    /// Queue depth (pending tasks)
    queue_depth: AtomicU64,
    /// Monitor update interval
    update_interval: Duration,
    /// Last update timestamp
    last_update: parking_lot::RwLock<Instant>,
}

impl SystemMonitor {
    /// Creates a new system monitor.
    pub fn new(update_interval: Duration) -> Self {
        Self {
            active: AtomicBool::new(false),
            cpu_usage: AtomicU64::new(0),
            memory_used: AtomicU64::new(0),
            total_memory: AtomicU64::new(Self::get_total_memory()),
            worker_threads: AtomicU64::new(0),
            queue_depth: AtomicU64::new(0),
            update_interval,
            last_update: parking_lot::RwLock::new(Instant::now()),
        }
    }

    /// Creates an Arc-wrapped monitor for shared access.
    pub fn shared(update_interval: Duration) -> Arc<Self> {
        Arc::new(Self::new(update_interval))
    }

    /// Starts the monitoring background task.
    pub fn start(&self) {
        self.active.store(true, Ordering::Relaxed);
    }

    /// Stops the monitoring background task.
    pub fn stop(&self) {
        self.active.store(false, Ordering::Relaxed);
    }

    /// Returns whether monitoring is active.
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    /// Updates system metrics. This should be called periodically.
    pub fn update(&self) {
        if !self.active.load(Ordering::Relaxed) {
            return;
        }

        // Update CPU usage
        self.cpu_usage.store(Self::get_cpu_usage(), Ordering::Relaxed);

        // Update memory usage
        self.memory_used.store(Self::get_memory_used(), Ordering::Relaxed);

        // Update timestamp
        *self.last_update.write() = Instant::now();
    }

    /// Returns current CPU usage percentage.
    pub fn cpu_usage(&self) -> u64 {
        self.cpu_usage.load(Ordering::Relaxed)
    }

    /// Returns current memory usage in bytes.
    pub fn memory_used(&self) -> u64 {
        self.memory_used.load(Ordering::Relaxed)
    }

    /// Returns total system memory in bytes.
    pub fn total_memory(&self) -> u64 {
        self.total_memory.load(Ordering::Relaxed)
    }

    /// Returns memory usage as a percentage.
    pub fn memory_usage_percent(&self) -> f64 {
        let total = self.total_memory();
        if total > 0 {
            (self.memory_used() as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Sets the number of active worker threads.
    pub fn set_worker_threads(&self, count: u64) {
        self.worker_threads.store(count, Ordering::Relaxed);
    }

    /// Returns the number of active worker threads.
    pub fn worker_threads(&self) -> u64 {
        self.worker_threads.load(Ordering::Relaxed)
    }

    /// Sets the current queue depth.
    pub fn set_queue_depth(&self, depth: u64) {
        self.queue_depth.store(depth, Ordering::Relaxed);
    }

    /// Returns the current queue depth.
    pub fn queue_depth(&self) -> u64 {
        self.queue_depth.load(Ordering::Relaxed)
    }

    /// Returns a snapshot of system status.
    pub fn status(&self) -> SystemStatus {
        SystemStatus {
            cpu_usage: self.cpu_usage(),
            memory_used: self.memory_used(),
            total_memory: self.total_memory(),
            memory_percent: self.memory_usage_percent(),
            worker_threads: self.worker_threads(),
            queue_depth: self.queue_depth(),
            active: self.is_active(),
        }
    }

    /// Checks if system resources are within acceptable limits.
    pub fn is_healthy(&self) -> bool {
        let cpu = self.cpu_usage();
        let mem_percent = self.memory_usage_percent();
        
        // Consider unhealthy if CPU > 95% or memory > 90%
        cpu < 95 && mem_percent < 90.0
    }

    // Platform-specific implementations

    fn get_total_memory() -> u64 {
        // Read from /proc/meminfo on Linux
        if let Ok(mut file) = std::fs::File::open("/proc/meminfo") {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                for line in contents.lines() {
                    if line.starts_with("MemTotal:") {
                        // Format: "MemTotal:    16384000 kB"
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            if let Ok(kb) = parts[1].parse::<u64>() {
                                return kb * 1024; // Convert kB to bytes
                            }
                        }
                    }
                }
            }
        }
        // Fallback: assume 8GB
        8 * 1024 * 1024 * 1024
    }

    fn get_cpu_usage() -> u64 {
        // Read from /proc/stat on Linux
        if let Ok(mut file) = std::fs::File::open("/proc/stat") {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                for line in contents.lines() {
                    if line.starts_with("cpu ") {
                        // Parse CPU times
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 5 {
                            let user: u64 = parts[1].parse().unwrap_or(0);
                            let nice: u64 = parts[2].parse().unwrap_or(0);
                            let system: u64 = parts[3].parse().unwrap_or(0);
                            let idle: u64 = parts[4].parse().unwrap_or(0);
                            
                            let total = user + nice + system + idle;
                            let busy = user + nice + system;
                            
                            if total > 0 {
                                return ((busy as f64 / total as f64) * 100.0) as u64;
                            }
                        }
                    }
                }
            }
        }
        // Fallback
        50
    }

    fn get_memory_used() -> u64 {
        // Read from /proc/meminfo on Linux
        if let Ok(mut file) = std::fs::File::open("/proc/meminfo") {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                let mut mem_free = 0u64;
                let mut buffers = 0u64;
                let mut cached = 0u64;
                
                for line in contents.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        match parts[0] {
                            "MemFree:" => mem_free = parts[1].parse().unwrap_or(0),
                            "Buffers:" => buffers = parts[1].parse().unwrap_or(0),
                            "Cached:" => cached = parts[1].parse().unwrap_or(0),
                            _ => {}
                        }
                    }
                }
                
                // Used = Total - Free - Buffers - Cached
                let total = Self::get_total_memory() / 1024; // Convert to kB
                let used_kb = total.saturating_sub(mem_free).saturating_sub(buffers).saturating_sub(cached);
                return used_kb * 1024; // Convert back to bytes
            }
        }
        // Fallback: assume 50% usage
        Self::get_total_memory() / 2
    }
}

/// A snapshot of system status.
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub cpu_usage: u64,
    pub memory_used: u64,
    pub total_memory: u64,
    pub memory_percent: f64,
    pub worker_threads: u64,
    pub queue_depth: u64,
    pub active: bool,
}

impl std::fmt::Display for SystemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== System Status ===")?;
        writeln!(f, "CPU Usage: {}%", self.cpu_usage)?;
        writeln!(f, "Memory: {} / {} ({:.1}%)", 
            format_bytes(self.memory_used), 
            format_bytes(self.total_memory),
            self.memory_percent)?;
        writeln!(f, "Worker Threads: {}", self.worker_threads)?;
        writeln!(f, "Queue Depth: {}", self.queue_depth)?;
        writeln!(f, "Active: {}", self.active)?;
        Ok(())
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Resource limit configuration for the fuzzer.
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum CPU usage percentage (0-100)
    pub max_cpu_percent: u8,
    /// Maximum memory usage percentage (0-100)
    pub max_memory_percent: u8,
    /// Maximum queue depth before throttling
    pub max_queue_depth: u64,
    /// Minimum available memory in bytes
    pub min_available_memory: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_percent: 90,
            max_memory_percent: 85,
            max_queue_depth: 10000,
            min_available_memory: 512 * 1024 * 1024, // 512 MB
        }
    }
}

impl ResourceLimits {
    /// Checks if current system status is within limits.
    pub fn check(&self, status: &SystemStatus) -> bool {
        status.cpu_usage < self.max_cpu_percent as u64 &&
        status.memory_percent < self.max_memory_percent as f64 &&
        status.queue_depth < self.max_queue_depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_monitor_creation() {
        let monitor = SystemMonitor::new(Duration::from_secs(1));
        assert!(!monitor.is_active());
        assert_eq!(monitor.cpu_usage(), 0);
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_cpu_percent, 90);
        assert_eq!(limits.max_memory_percent, 85);
    }
}
