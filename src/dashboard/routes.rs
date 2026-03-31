//! Dashboard API routes
//!
//! Implements the REST API endpoints for the dashboard.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;

use super::server::AppState;
use crate::dashboard::{CampaignSummary, BugReport, DashboardMetrics, CampaignStatus, BugSeverity};

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Get dashboard metrics
pub async fn get_metrics(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let metrics = DashboardMetrics {
        active_campaigns: 2,
        total_campaigns: 5,
        total_bugs_found: 12,
        total_inputs_tested: 150000,
        average_coverage: 78.5,
        uptime_seconds: 3600,
    };
    
    Json(metrics)
}

/// List all fuzzing campaigns
pub async fn list_campaigns(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let campaigns = vec![
        CampaignSummary {
            id: "camp_001".to_string(),
            name: "Circom Mainnet".to_string(),
            target_type: "circom".to_string(),
            status: CampaignStatus::Running,
            progress: 67.3,
            total_inputs: 50000,
            interesting_inputs: 45,
            crashes_found: 3,
            start_time: "2026-03-31T08:00:00Z".to_string(),
            last_update: "2026-03-31T10:30:00Z".to_string(),
        },
        CampaignSummary {
            id: "camp_002".to_string(),
            name: "Noir Testnet".to_string(),
            target_type: "noir".to_string(),
            status: CampaignStatus::Running,
            progress: 34.1,
            total_inputs: 25000,
            interesting_inputs: 18,
            crashes_found: 1,
            start_time: "2026-03-31T09:15:00Z".to_string(),
            last_update: "2026-03-31T10:25:00Z".to_string(),
        },
    ];
    
    Json(json!({
        "campaigns": campaigns,
        "total": campaigns.len()
    }))
}

/// Get specific campaign details
pub async fn get_campaign(
    Path(campaign_id): Path<String>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // In a real implementation, this would fetch from storage
    let campaign = CampaignSummary {
        id: campaign_id.clone(),
        name: format!("Campaign {}", campaign_id),
        target_type: "circom".to_string(),
        status: CampaignStatus::Running,
        progress: 45.2,
        total_inputs: 10000,
        interesting_inputs: 8,
        crashes_found: 1,
        start_time: "2026-03-31T09:00:00Z".to_string(),
        last_update: "2026-03-31T10:30:00Z".to_string(),
    };
    
    Json(campaign)
}

/// List all discovered bugs
pub async fn list_bugs(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let bugs = vec![
        BugReport {
            id: "bug_001".to_string(),
            campaign_id: "camp_001".to_string(),
            bug_type: "ConstraintUnsatisfiable".to_string(),
            severity: BugSeverity::High,
            description: "Found input that violates constraint system".to_string(),
            input_data: vec![0x01, 0x02, 0x03],
            timestamp: "2026-03-31T09:45:00Z".to_string(),
        },
        BugReport {
            id: "bug_002".to_string(),
            campaign_id: "camp_002".to_string(),
            bug_type: "SignalNotAssigned".to_string(),
            severity: BugSeverity::Medium,
            description: "Signal not properly assigned in circuit".to_string(),
            input_data: vec![0x04, 0x05, 0x06],
            timestamp: "2026-03-31T10:15:00Z".to_string(),
        },
    ];
    
    Json(json!({
        "bugs": bugs,
        "total": bugs.len()
    }))
}

/// Get specific bug details
pub async fn get_bug(
    Path(bug_id): Path<String>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let bug = BugReport {
        id: bug_id.clone(),
        campaign_id: "camp_001".to_string(),
        bug_type: "ConstraintUnsatisfiable".to_string(),
        severity: BugSeverity::High,
        description: "Detailed bug report for constraint violation".to_string(),
        input_data: vec![0x01, 0x02, 0x03],
        timestamp: "2026-03-31T09:45:00Z".to_string(),
    };
    
    Json(bug)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        assert_eq!(response.into_response().status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_metrics() {
        let state = Arc::new(AppState::new(Default::default()));
        let response = get_metrics(State(state)).await;
        assert_eq!(response.into_response().status(), StatusCode::OK);
    }
}
