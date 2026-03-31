//! Static analysis for ZK circuits
//!
//! This module provides static analysis capabilities for detecting
//! bugs and vulnerabilities in ZK circuits without execution.

use crate::core::parser::{Circuit, Constraint, Expression, Signal};
use crate::core::constraints::ConstraintSystem;
use num_traits::Zero;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Static analyzer for ZK circuits
pub struct StaticAnalyzer;

/// Analysis result from static analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticAnalysisResult {
    pub issues: Vec<StaticIssue>,
    pub metrics: CircuitMetrics,
    pub warnings: Vec<String>,
    pub summary: String,
}

/// Issue found during static analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticIssue {
    pub issue_type: IssueType,
    pub severity: IssueSeverity,
    pub location: String,
    pub description: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueType {
    UnderConstrained,
    OverConstrained,
    UnusedSignal,
    UnreachableConstraint,
    DivisionByZero,
    CircularDependency,
    TypeMismatch,
    MissingInput,
    MissingOutput,
    InconsistentDimensions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Metrics about a circuit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitMetrics {
    pub total_signals: usize,
    pub input_signals: usize,
    pub output_signals: usize,
    pub witness_signals: usize,
    pub total_constraints: usize,
    pub linear_constraints: usize,
    pub quadratic_constraints: usize,
    pub constant_constraints: usize,
    pub unused_signals: usize,
    pub constraint_signal_ratio: f64,
    pub estimated_complexity: f64,
}

impl StaticAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Perform static analysis on a circuit
    pub fn analyze(&self, circuit: &Circuit) -> StaticAnalysisResult {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check for under-constrained circuits
        self.check_under_constrained(circuit, &mut issues);

        // Check for over-constrained circuits
        self.check_over_constrained(circuit, &mut issues);

        // Check for unused signals
        self.check_unused_signals(circuit, &mut issues);

        // Check for unreachable constraints
        self.check_unreachable_constraints(circuit, &mut issues);

        // Check for potential division by zero
        self.check_division_by_zero(circuit, &mut issues);

        // Compute metrics
        let metrics = self.compute_metrics(circuit);

        // Generate summary
        let summary = self.generate_summary(&issues, &metrics);

        StaticAnalysisResult {
            issues,
            metrics,
            warnings,
            summary,
        }
    }

    fn check_under_constrained(&self, circuit: &Circuit, issues: &mut Vec<StaticIssue>) {
        let witness_signals = circuit.signals.iter()
            .filter(|s| matches!(s.direction, crate::core::parser::SignalDirection::Intermediate))
            .count();
        
        let constraints = circuit.constraints.len();
        
        if witness_signals > constraints {
            issues.push(StaticIssue {
                issue_type: IssueType::UnderConstrained,
                severity: IssueSeverity::High,
                location: "circuit".to_string(),
                description: format!(
                    "Circuit may be under-constrained: {} witness signals but only {} constraints",
                    witness_signals, constraints
                ),
                suggestion: "Add more constraints or reduce the number of intermediate signals".to_string(),
            });
        }
    }

    fn check_over_constrained(&self, circuit: &Circuit, issues: &mut Vec<StaticIssue>) {
        let total_signals = circuit.signals.len();
        let constraints = circuit.constraints.len();
        
        if constraints > total_signals * 2 && total_signals > 0 {
            issues.push(StaticIssue {
                issue_type: IssueType::OverConstrained,
                severity: IssueSeverity::Medium,
                location: "circuit".to_string(),
                description: format!(
                    "Circuit may be over-constrained: {} constraints for {} signals",
                    constraints, total_signals
                ),
                suggestion: "Review constraints for redundancy or conflicts".to_string(),
            });
        }
    }

    fn check_unused_signals(&self, circuit: &Circuit, issues: &mut Vec<StaticIssue>) {
        let mut used_signals = HashSet::new();
        
        // Collect all signals used in constraints
        for constraint in &circuit.constraints {
            self.collect_signal_names(&constraint.left, &mut used_signals);
            self.collect_signal_names(&constraint.right, &mut used_signals);
        }
        
        // Check for unused signals
        for signal in &circuit.signals {
            if !used_signals.contains(&signal.name) && 
               matches!(signal.direction, crate::core::parser::SignalDirection::Intermediate) {
                issues.push(StaticIssue {
                    issue_type: IssueType::UnusedSignal,
                    severity: IssueSeverity::Low,
                    location: format!("signal: {}", signal.name),
                    description: format!("Signal '{}' is declared but never used", signal.name),
                    suggestion: "Remove unused signal or use it in constraints".to_string(),
                });
            }
        }
    }

    fn check_unreachable_constraints(&self, circuit: &Circuit, issues: &mut Vec<StaticIssue>) {
        // Simple heuristic: constraints with only constants
        for (idx, constraint) in circuit.constraints.iter().enumerate() {
            if self.is_constant_expression(&constraint.left) && 
               self.is_constant_expression(&constraint.right) {
                issues.push(StaticIssue {
                    issue_type: IssueType::UnreachableConstraint,
                    severity: IssueSeverity::Medium,
                    location: format!("constraint: {}", idx),
                    description: format!("Constraint {} appears to be constant-only", idx),
                    suggestion: "Review constraint for correctness".to_string(),
                });
            }
        }
    }

    fn check_division_by_zero(&self, circuit: &Circuit, issues: &mut Vec<StaticIssue>) {
        for (idx, constraint) in circuit.constraints.iter().enumerate() {
            if self.contains_division_by_zero_risk(&constraint.left) ||
               self.contains_division_by_zero_risk(&constraint.right) {
                issues.push(StaticIssue {
                    issue_type: IssueType::DivisionByZero,
                    severity: IssueSeverity::High,
                    location: format!("constraint: {}", idx),
                    description: format!("Constraint {} may have division by zero", idx),
                    suggestion: "Add checks to ensure divisor is non-zero".to_string(),
                });
            }
        }
    }

    fn collect_signal_names(&self, expr: &Expression, names: &mut HashSet<String>) {
        match expr {
            Expression::Signal(name) => {
                names.insert(name.clone());
            }
            Expression::BinaryOp(left, _, right) => {
                self.collect_signal_names(left, names);
                self.collect_signal_names(right, names);
            }
            Expression::UnaryOp(_, inner) => {
                self.collect_signal_names(inner, names);
            }
            Expression::FunctionCall(_, args) => {
                for arg in args {
                    self.collect_signal_names(arg, names);
                }
            }
            Expression::ComponentOutput(component, signal) => {
                names.insert(format!("{}.{}", component, signal));
            }
            Expression::Constant(_) => {}
        }
    }

    fn is_constant_expression(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Constant(_) => true,
            Expression::Signal(_) => false,
            Expression::BinaryOp(left, _, right) => {
                self.is_constant_expression(left) && self.is_constant_expression(right)
            }
            Expression::UnaryOp(_, inner) => self.is_constant_expression(inner),
            Expression::FunctionCall(_, args) => {
                args.iter().all(|a| self.is_constant_expression(a))
            }
            Expression::ComponentOutput(_, _) => false,
        }
    }

    fn contains_division_by_zero_risk(&self, expr: &Expression) -> bool {
        match expr {
            Expression::BinaryOp(_, op, right) => {
                if matches!(op, crate::core::parser::BinaryOperator::Div) {
                    // Check if divisor could be zero
                    match right.as_ref() {
                        Expression::Constant(val) => val.is_zero(),
                        Expression::Signal(_) => true, // Could be zero at runtime
                        _ => self.contains_division_by_zero_risk(right),
                    }
                } else {
                    self.contains_division_by_zero_risk(right)
                }
            }
            Expression::UnaryOp(_, inner) => self.contains_division_by_zero_risk(inner),
            Expression::FunctionCall(_, args) => {
                args.iter().any(|a| self.contains_division_by_zero_risk(a))
            }
            _ => false,
        }
    }

    fn compute_metrics(&self, circuit: &Circuit) -> CircuitMetrics {
        let total_signals = circuit.signals.len();
        let input_signals = circuit.signals.iter()
            .filter(|s| matches!(s.direction, crate::core::parser::SignalDirection::Input))
            .count();
        let output_signals = circuit.signals.iter()
            .filter(|s| matches!(s.direction, crate::core::parser::SignalDirection::Output))
            .count();
        let witness_signals = circuit.signals.iter()
            .filter(|s| matches!(s.direction, crate::core::parser::SignalDirection::Intermediate))
            .count();
        
        let total_constraints = circuit.constraints.len();
        
        // Classify constraints by degree (simplified)
        let mut linear = 0;
        let mut quadratic = 0;
        let mut constant = 0;
        
        for constraint in &circuit.constraints {
            let left_degree = self.get_expression_degree(&constraint.left);
            let right_degree = self.get_expression_degree(&constraint.right);
            let total_degree = left_degree + right_degree;
            
            match total_degree {
                0 => constant += 1,
                1 => linear += 1,
                _ => quadratic += 1,
            }
        }
        
        let constraint_signal_ratio = if total_signals > 0 {
            total_constraints as f64 / total_signals as f64
        } else {
            0.0
        };
        
        let estimated_complexity = (total_constraints as f64 * quadratic as f64).sqrt();
        
        CircuitMetrics {
            total_signals,
            input_signals,
            output_signals,
            witness_signals,
            total_constraints,
            linear_constraints: linear,
            quadratic_constraints: quadratic,
            constant_constraints: constant,
            unused_signals: 0, // Would be computed from analysis
            constraint_signal_ratio,
            estimated_complexity,
        }
    }

    fn get_expression_degree(&self, expr: &Expression) -> usize {
        match expr {
            Expression::Constant(_) => 0,
            Expression::Signal(_) => 1,
            Expression::BinaryOp(left, op, right) => {
                let left_degree = self.get_expression_degree(left);
                let right_degree = self.get_expression_degree(right);
                match op {
                    crate::core::parser::BinaryOperator::Mul => left_degree + right_degree,
                    crate::core::parser::BinaryOperator::Pow => left_degree * right_degree,
                    _ => std::cmp::max(left_degree, right_degree),
                }
            }
            Expression::UnaryOp(_, inner) => self.get_expression_degree(inner),
            Expression::FunctionCall(_, args) => {
                args.iter().map(|a| self.get_expression_degree(a)).max().unwrap_or(0)
            }
            Expression::ComponentOutput(_, _) => 1,
        }
    }

    fn generate_summary(&self, issues: &[StaticIssue], metrics: &CircuitMetrics) -> String {
        let critical_count = issues.iter()
            .filter(|i| i.severity == IssueSeverity::Critical)
            .count();
        let high_count = issues.iter()
            .filter(|i| i.severity == IssueSeverity::High)
            .count();
        
        format!(
            "Static analysis complete. Found {} critical and {} high severity issues. \
             Circuit has {} signals, {} constraints, complexity: {:.2}",
            critical_count, high_count, metrics.total_signals, metrics.total_constraints, metrics.estimated_complexity
        )
    }
}

impl Default for StaticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = StaticAnalyzer::new();
        assert!(true);
    }
}
