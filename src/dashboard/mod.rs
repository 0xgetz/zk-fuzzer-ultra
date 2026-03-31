//! Web Dashboard for ZK Circuit Fuzzer
//!
//! Provides a web-based UI for monitoring fuzzing campaigns, viewing results,
//! and managing circuit targets.

pub mod routes;
pub mod server;

use serde::{Deserialize, Serialize};

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub host: String,
    pub port: u16,
    pub enable_cors: bool,
    pub static_dir: String,
}

impl DashboardConfig {
    pub fn new() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            enable_cors: true,
            static_dir: "static".to_string(),
        }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn with_host(mut self, host: String) -> Self {
        self.host = host;
        self
    }
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Fuzzing campaign summary for dashboard display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignSummary {
    pub id: String,
    pub name: String,
    pub target_type: String,
    pub status: CampaignStatus,
    pub progress: f64,
    pub total_inputs: u64,
    pub interesting_inputs: u64,
    pub crashes_found: u64,
    pub start_time: String,
    pub last_update: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CampaignStatus {
    Running,
    Paused,
    Completed,
    Failed,
}

/// Bug report for dashboard display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugReport {
    pub id: String,
    pub campaign_id: String,
    pub bug_type: String,
    pub severity: BugSeverity,
    pub description: String,
    pub input_data: Vec<u8>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BugSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Metrics for dashboard display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetrics {
    pub active_campaigns: usize,
    pub total_campaigns: usize,
    pub total_bugs_found: usize,
    pub total_inputs_tested: u64,
    pub average_coverage: f64,
    pub uptime_seconds: u64,
}

impl DashboardMetrics {
    pub fn new() -> Self {
        Self {
            active_campaigns: 0,
            total_campaigns: 0,
            total_bugs_found: 0,
            total_inputs_tested: 0,
            average_coverage: 0.0,
            uptime_seconds: 0,
        }
    }
}

impl Default for DashboardMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_config_default() {
        let config = DashboardConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert!(config.enable_cors);
    }

    #[test]
    fn test_campaign_summary_creation() {
        let summary = CampaignSummary {
            id: "camp_001".to_string(),
            name: "Test Campaign".to_string(),
            target_type: "circom".to_string(),
            status: CampaignStatus::Running,
            progress: 45.2,
            total_inputs: 1000,
            interesting_inputs: 15,
            crashes_found: 2,
            start_time: "2026-03-31T09:00:00Z".to_string(),
            last_update: "2026-03-31T10:30:00Z".to_string(),
        };
        assert_eq!(summary.id, "camp_001");
        assert_eq!(summary.status, CampaignStatus::Running);
    }
}
