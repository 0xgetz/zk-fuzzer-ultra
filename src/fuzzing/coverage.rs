//! Coverage tracking and measurement for fuzzing
//!
//! This module provides coverage tracking capabilities to measure
//! how well the fuzzer is exercising the circuit.

use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Coverage tracking for fuzzing campaigns
pub struct CoverageTracker {
    constraint_coverage: HashSet<usize>,
    signal_coverage: HashSet<String>,
    path_coverage: HashSet<u64>,
    branch_coverage: HashMap<usize, BranchInfo>,
    total_constraints: usize,
    total_signals: usize,
    total_branches: usize,
}

/// Information about a branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub branch_id: usize,
    pub taken_count: usize,
    pub not_taken_count: usize,
    pub is_covered: bool,
}

/// Coverage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub constraint_coverage_percent: f64,
    pub signal_coverage_percent: f64,
    pub branch_coverage_percent: f64,
    pub path_count: usize,
    pub total_constraints: usize,
    pub covered_constraints: usize,
    pub total_signals: usize,
    pub covered_signals: usize,
    pub total_branches: usize,
    pub covered_branches: usize,
}

impl CoverageTracker {
    pub fn new() -> Self {
        Self {
            constraint_coverage: HashSet::new(),
            signal_coverage: HashSet::new(),
            path_coverage: HashSet::new(),
            branch_coverage: HashMap::new(),
            total_constraints: 0,
            total_signals: 0,
            total_branches: 0,
        }
    }

    /// Initialize tracker with circuit information
    pub fn initialize(&mut self, total_constraints: usize, total_signals: usize, total_branches: usize) {
        self.total_constraints = total_constraints;
        self.total_signals = total_signals;
        self.total_branches = total_branches;

        // Initialize branch info
        for i in 0..total_branches {
            self.branch_coverage.insert(
                i,
                BranchInfo {
                    branch_id: i,
                    taken_count: 0,
                    not_taken_count: 0,
                    is_covered: false,
                },
            );
        }
    }

    /// Record constraint coverage
    pub fn cover_constraint(&mut self, constraint_idx: usize) {
        self.constraint_coverage.insert(constraint_idx);
    }

    /// Record signal coverage
    pub fn cover_signal(&mut self, signal_name: String) {
        self.signal_coverage.insert(signal_name);
    }

    /// Record path coverage
    pub fn cover_path(&mut self, path_hash: u64) {
        self.path_coverage.insert(path_hash);
    }

    /// Record branch coverage
    pub fn cover_branch(&mut self, branch_id: usize, taken: bool) {
        if let Some(branch_info) = self.branch_coverage.get_mut(&branch_id) {
            if taken {
                branch_info.taken_count += 1;
            } else {
                branch_info.not_taken_count += 1;
            }
            branch_info.is_covered = true;
        }
    }

    /// Record coverage from an execution trace
    pub fn record_trace_coverage(
        &mut self,
        signal_values: &HashMap<String, BigInt>,
        constraints_checked: &[usize],
    ) {
        // Record signal coverage
        for signal_name in signal_values.keys() {
            self.cover_signal(signal_name.clone());
        }

        // Record constraint coverage
        for constraint_idx in constraints_checked {
            self.cover_constraint(*constraint_idx);
        }
    }

    /// Get coverage report
    pub fn get_report(&self) -> CoverageReport {
        let covered_constraints = self.constraint_coverage.len();
        let covered_signals = self.signal_coverage.len();
        let covered_branches = self.branch_coverage.values().filter(|b| b.is_covered).count();

        CoverageReport {
            constraint_coverage_percent: if self.total_constraints > 0 {
                (covered_constraints as f64 / self.total_constraints as f64) * 100.0
            } else {
                0.0
            },
            signal_coverage_percent: if self.total_signals > 0 {
                (covered_signals as f64 / self.total_signals as f64) * 100.0
            } else {
                0.0
            },
            branch_coverage_percent: if self.total_branches > 0 {
                (covered_branches as f64 / self.total_branches as f64) * 100.0
            } else {
                0.0
            },
            path_count: self.path_coverage.len(),
            total_constraints: self.total_constraints,
            covered_constraints,
            total_signals: self.total_signals,
            covered_signals,
            total_branches: self.total_branches,
            covered_branches,
        }
    }

    /// Get uncovered constraints
    pub fn get_uncovered_constraints(&self) -> Vec<usize> {
        (0..self.total_constraints)
            .filter(|i| !self.constraint_coverage.contains(i))
            .collect()
    }

    /// Get uncovered signals
    pub fn get_uncovered_signals(&self) -> Vec<String> {
        // This would require tracking all signal names
        // For now, return empty (implementation would need signal list)
        vec![]
    }

    /// Get coverage summary string
    pub fn get_summary(&self) -> String {
        let report = self.get_report();
        format!(
            "Coverage: Constraints {:.1}%, Signals {:.1}%, Branches {:.1}%, Paths {}",
            report.constraint_coverage_percent,
            report.signal_coverage_percent,
            report.branch_coverage_percent,
            report.path_count
        )
    }
}

impl Default for CoverageTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_creation() {
        let tracker = CoverageTracker::new();
        assert_eq!(tracker.total_constraints, 0);
    }

    #[test]
    fn test_coverage_recording() {
        let mut tracker = CoverageTracker::new();
        tracker.initialize(10, 5, 8);
        
        tracker.cover_constraint(0);
        tracker.cover_constraint(1);
        tracker.cover_signal("input_a".to_string());
        
        let report = tracker.get_report();
        assert_eq!(report.covered_constraints, 2);
        assert_eq!(report.covered_signals, 1);
    }
}
