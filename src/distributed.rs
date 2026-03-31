//! Distributed fuzzing coordination for the ZK Circuit Fuzzer.
//!
//! This module provides distributed fuzzing capabilities including worker coordination,
//! task distribution, result aggregation, and cluster management for scaling across
//! multiple machines.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::net::SocketAddr;

/// Types of nodes in a distributed fuzzing cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    /// Coordinator node that manages the cluster
    Coordinator,
    /// Worker node that performs fuzzing
    Worker,
    /// Aggregator node that collects results
    Aggregator,
}

/// Status of a distributed node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    /// Node is initializing
    Initializing,
    /// Node is ready and accepting work
    Ready,
    /// Node is actively processing tasks
    Working,
    /// Node is paused
    Paused,
    /// Node has stopped
    Stopped,
    /// Node is unreachable
    Unreachable,
}

/// A node in the distributed fuzzing cluster.
#[derive(Debug, Clone)]
pub struct ClusterNode {
    /// Unique node identifier
    pub id: String,
    /// Type of node
    pub node_type: NodeType,
    /// Current status
    pub status: NodeStatus,
    /// Network address
    pub address: SocketAddr,
    /// Number of tasks completed
    pub tasks_completed: u64,
    /// Number of tasks in progress
    pub tasks_in_progress: u64,
    /// Last heartbeat timestamp
    pub last_heartbeat: Instant,
    /// Node capabilities (e.g., supported circuit types)
    pub capabilities: Vec<String>,
}

impl ClusterNode {
    /// Creates a new worker node.
    pub fn worker(id: String, address: SocketAddr) -> Self {
        Self {
            id,
            node_type: NodeType::Worker,
            status: NodeStatus::Initializing,
            address,
            tasks_completed: 0,
            tasks_in_progress: 0,
            last_heartbeat: Instant::now(),
            capabilities: vec![],
        }
    }

    /// Creates a new coordinator node.
    pub fn coordinator(id: String, address: SocketAddr) -> Self {
        Self {
            id,
            node_type: NodeType::Coordinator,
            status: NodeStatus::Ready,
            address,
            tasks_completed: 0,
            tasks_in_progress: 0,
            last_heartbeat: Instant::now(),
            capabilities: vec!["coordination".to_string()],
        }
    }

    /// Marks the node as having sent a heartbeat.
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
    }

    /// Checks if the node is considered alive based on heartbeat timeout.
    pub fn is_alive(&self, timeout: Duration) -> bool {
        self.last_heartbeat.elapsed() < timeout
    }
}

/// Distributed fuzzing cluster manager.
#[derive(Debug)]
pub struct ClusterManager {
    /// Unique cluster identifier
    cluster_id: String,
    /// Nodes in the cluster
    nodes: parking_lot::RwLock<HashMap<String, ClusterNode>>,
    /// Whether the cluster is active
    active: AtomicBool,
    /// Total tasks distributed
    total_tasks_distributed: AtomicU64,
    /// Total tasks completed
    total_tasks_completed: AtomicU64,
    /// Cluster start time
    start_time: Instant,
    /// Heartbeat timeout duration
    heartbeat_timeout: Duration,
}

impl ClusterManager {
    /// Creates a new cluster manager.
    pub fn new(cluster_id: String) -> Self {
        Self {
            cluster_id,
            nodes: parking_lot::RwLock::new(HashMap::new()),
            active: AtomicBool::new(false),
            total_tasks_distributed: AtomicU64::new(0),
            total_tasks_completed: AtomicU64::new(0),
            start_time: Instant::now(),
            heartbeat_timeout: Duration::from_secs(30),
        }
    }

    /// Creates an Arc-wrapped cluster manager.
    pub fn shared(cluster_id: String) -> Arc<Self> {
        Arc::new(Self::new(cluster_id))
    }

    /// Activates the cluster manager.
    pub fn activate(&self) {
        self.active.store(true, Ordering::Relaxed);
    }

    /// Deactivates the cluster manager.
    pub fn deactivate(&self) {
        self.active.store(false, Ordering::Relaxed);
    }

    /// Returns whether the cluster is active.
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    /// Registers a new node in the cluster.
    pub fn register_node(&self, node: ClusterNode) {
        let mut nodes = self.nodes.write();
        nodes.insert(node.id.clone(), node);
    }

    /// Unregisters a node from the cluster.
    pub fn unregister_node(&self, node_id: &str) {
        let mut nodes = self.nodes.write();
        nodes.remove(node_id);
    }

    /// Updates a node's status.
    pub fn update_node_status(&self, node_id: &str, status: NodeStatus) {
        let mut nodes = self.nodes.write();
        if let Some(node) = nodes.get_mut(node_id) {
            node.status = status;
        }
    }

    /// Records a heartbeat from a node.
    pub fn node_heartbeat(&self, node_id: &str) {
        let mut nodes = self.nodes.write();
        if let Some(node) = nodes.get_mut(node_id) {
            node.heartbeat();
        }
    }

    /// Returns all nodes in the cluster.
    pub fn nodes(&self) -> Vec<ClusterNode> {
        self.nodes.read().values().cloned().collect()
    }

    /// Returns only worker nodes.
    pub fn worker_nodes(&self) -> Vec<ClusterNode> {
        self.nodes.read()
            .values()
            .filter(|n| n.node_type == NodeType::Worker)
            .cloned()
            .collect()
    }

    /// Returns the number of active workers.
    pub fn active_worker_count(&self) -> usize {
        self.nodes.read()
            .values()
            .filter(|n| n.node_type == NodeType::Worker && n.status == NodeStatus::Working)
            .count()
    }

    /// Returns the total number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.read().len()
    }

    /// Distributes a batch of tasks to available workers.
    pub fn distribute_tasks(&self, task_count: u64) -> u64 {
        if !self.active.load(Ordering::Relaxed) {
            return 0;
        }

        let workers = self.worker_nodes();
        if workers.is_empty() {
            return 0;
        }

        // Simple round-robin distribution
        let tasks_per_worker = task_count / workers.len() as u64;
        let mut distributed = 0u64;

        for worker in &workers {
            if worker.status == NodeStatus::Ready || worker.status == NodeStatus::Working {
                // In a real implementation, we'd send tasks to the worker here
                distributed += tasks_per_worker;
            }
        }

        self.total_tasks_distributed.fetch_add(distributed, Ordering::Relaxed);
        distributed
    }

    /// Records task completion.
    pub fn record_task_completion(&self) {
        self.total_tasks_completed.fetch_add(1, Ordering::Relaxed);
    }

    /// Returns cluster statistics.
    pub fn stats(&self) -> ClusterStats {
        let nodes = self.nodes.read();
        let workers = nodes.values().filter(|n| n.node_type == NodeType::Worker).count();
        let active_workers = nodes.values()
            .filter(|n| n.node_type == NodeType::Worker && 
                   (n.status == NodeStatus::Working || n.status == NodeStatus::Ready))
            .count();
        
        ClusterStats {
            cluster_id: self.cluster_id.clone(),
            total_nodes: nodes.len(),
            worker_nodes: workers,
            active_workers,
            tasks_distributed: self.total_tasks_distributed.load(Ordering::Relaxed),
            tasks_completed: self.total_tasks_completed.load(Ordering::Relaxed),
            uptime: self.start_time.elapsed(),
            is_active: self.is_active(),
        }
    }

    /// Checks for and removes dead nodes.
    pub fn cleanup_dead_nodes(&self) -> Vec<String> {
        let mut dead_nodes = Vec::new();
        let mut nodes = self.nodes.write();
        
        nodes.retain(|id, node| {
            if !node.is_alive(self.heartbeat_timeout) && 
               node.status != NodeStatus::Stopped {
                dead_nodes.push(id.clone());
                false
            } else {
                true
            }
        });
        
        dead_nodes
    }

    /// Sets the heartbeat timeout.
    pub fn set_heartbeat_timeout(&mut self, timeout: Duration) {
        self.heartbeat_timeout = timeout;
    }
}

/// Statistics for a distributed cluster.
#[derive(Debug, Clone)]
pub struct ClusterStats {
    pub cluster_id: String,
    pub total_nodes: usize,
    pub worker_nodes: usize,
    pub active_workers: usize,
    pub tasks_distributed: u64,
    pub tasks_completed: u64,
    pub uptime: Duration,
    pub is_active: bool,
}

impl std::fmt::Display for ClusterStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Cluster Statistics ===")?;
        writeln!(f, "Cluster ID: {}", self.cluster_id)?;
        writeln!(f, "Total Nodes: {}", self.total_nodes)?;
        writeln!(f, "Worker Nodes: {}", self.worker_nodes)?;
        writeln!(f, "Active Workers: {}", self.active_workers)?;
        writeln!(f, "Tasks Distributed: {}", self.tasks_distributed)?;
        writeln!(f, "Tasks Completed: {}", self.tasks_completed)?;
        writeln!(f, "Uptime: {:.2?}", self.uptime)?;
        writeln!(f, "Active: {}", self.is_active)?;
        Ok(())
    }
}

/// Message types for inter-node communication.
#[derive(Debug, Clone)]
pub enum ClusterMessage {
    /// Register a new worker
    RegisterWorker { id: String, capabilities: Vec<String> },
    /// Assign work to a worker
    AssignWork { worker_id: String, work_items: Vec<WorkItem> },
    /// Report work completion
    WorkComplete { worker_id: String, results: Vec<WorkResult> },
    /// Heartbeat ping
    Heartbeat { node_id: String },
    /// Heartbeat response
    HeartbeatAck { node_id: String },
    /// Shutdown signal
    Shutdown,
    /// Pause fuzzing
    Pause,
    /// Resume fuzzing
    Resume,
    /// Request cluster status
    StatusRequest,
    /// Cluster status response
    StatusResponse(ClusterStats),
}

/// A unit of work to be distributed.
#[derive(Debug, Clone)]
pub struct WorkItem {
    /// Unique work item ID
    pub id: u64,
    /// Circuit type to fuzz
    pub circuit_type: String,
    /// Seed for random input generation
    pub seed: u64,
    /// Maximum number of iterations
    pub max_iterations: u64,
    /// Timeout per iteration in milliseconds
    pub timeout_ms: u64,
}

/// Result from processing a work item.
#[derive(Debug, Clone)]
pub struct WorkResult {
    /// Work item ID
    pub work_item_id: u64,
    /// Whether the work completed successfully
    pub success: bool,
    /// Number of test cases executed
    pub test_cases: u64,
    /// Number of unique paths found
    pub unique_paths: u64,
    /// Number of crashes found
    pub crashes: u64,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Error message if failed
    pub error: Option<String>,
}

/// Configuration for distributed fuzzing.
#[derive(Debug, Clone)]
pub struct DistributedConfig {
    /// Cluster identifier
    pub cluster_id: String,
    /// Coordinator address (host:port)
    pub coordinator_address: String,
    /// Node type (coordinator or worker)
    pub node_type: NodeType,
    /// Maximum concurrent workers
    pub max_workers: usize,
    /// Task batch size
    pub batch_size: u64,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// Connection timeout
    pub connection_timeout: Duration,
}

impl Default for DistributedConfig {
    fn default() -> Self {
        Self {
            cluster_id: "default-cluster".to_string(),
            coordinator_address: "127.0.0.1:9999".to_string(),
            node_type: NodeType::Coordinator,
            max_workers: 10,
            batch_size: 100,
            heartbeat_interval: Duration::from_secs(5),
            connection_timeout: Duration::from_secs(10),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_cluster_manager_creation() {
        let manager = ClusterManager::new("test-cluster".to_string());
        assert!(!manager.is_active());
        assert_eq!(manager.node_count(), 0);
    }

    #[test]
    fn test_node_registration() {
        let manager = ClusterManager::new("test-cluster".to_string());
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let worker = ClusterNode::worker("worker-1".to_string(), addr);
        
        manager.register_node(worker);
        assert_eq!(manager.node_count(), 1);
        assert_eq!(manager.worker_nodes().len(), 1);
    }

    #[test]
    fn test_cluster_stats() {
        let manager = ClusterManager::new("test-cluster".to_string());
        manager.activate();
        
        let stats = manager.stats();
        assert_eq!(stats.cluster_id, "test-cluster");
        assert!(stats.is_active);
    }
}
