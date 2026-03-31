//! Constraint system extraction and analysis
//!
//! This module handles extraction of constraints from parsed circuits
//! and provides analysis capabilities for constraint systems.

use crate::core::parser::{Circuit, Constraint, Expression, Template};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Constraint system representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintSystem {
    pub constraints: Vec<Constraint>,
    pub signals: HashMap<String, SignalInfo>,
    pub metadata: ConstraintMetadata,
}

/// Information about a signal in the constraint system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalInfo {
    pub name: String,
    pub is_input: bool,
    pub is_output: bool,
    pub is_witness: bool,
    pub constraints_involved: Vec<usize>,
    pub domain: SignalDomain,
}

/// Domain of a signal (field, binary, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignalDomain {
    Field,
    Binary,
    Range { min: i128, max: i128 },
    Custom(String),
}

/// Metadata about the constraint system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintMetadata {
    pub total_constraints: usize,
    pub total_signals: usize,
    pub input_signals: usize,
    pub output_signals: usize,
    pub witness_signals: usize,
    pub quadratic_constraints: usize,
    pub linear_constraints: usize,
    pub constant_constraints: usize,
    pub is_under_constrained: Option<bool>,
    pub is_over_constrained: Option<bool>,
}

/// Constraint extraction engine
pub struct ConstraintExtractor;

impl ConstraintExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extract constraint system from a parsed circuit
    pub fn extract(&self, circuit: &Circuit) -> ConstraintSystem {
        let mut signals = HashMap::new();
        let mut metadata = ConstraintMetadata {
            total_constraints: circuit.constraints.len(),
            total_signals: circuit.signals.len(),
            input_signals: 0,
            output_signals: 0,
            witness_signals: 0,
            quadratic_constraints: 0,
            linear_constraints: 0,
            constant_constraints: 0,
            is_under_constrained: None,
            is_over_constrained: None,
        };

        // Process signals
        for signal in &circuit.signals {
            let signal_info = SignalInfo {
                name: signal.name.clone(),
                is_input: matches!(signal.direction, crate::core::parser::SignalDirection::Input),
                is_output: matches!(signal.direction, crate::core::parser::SignalDirection::Output),
                is_witness: matches!(signal.direction, crate::core::parser::SignalDirection::Intermediate),
                constraints_involved: vec![],
                domain: match signal.signal_type {
                    crate::core::parser::SignalType::Field => SignalDomain::Field,
                    crate::core::parser::SignalType::Binary => SignalDomain::Binary,
                },
            };

            if signal_info.is_input {
                metadata.input_signals += 1;
            }
            if signal_info.is_output {
                metadata.output_signals += 1;
            }
            if signal_info.is_witness {
                metadata.witness_signals += 1;
            }

            signals.insert(signal.name.clone(), signal_info);
        }

        // Classify constraints
        for (i, constraint) in circuit.constraints.iter().enumerate() {
            self.classify_constraint(&constraint, &mut metadata);
            self.link_signal_constraints(&constraint, i, &mut signals);
        }

        // Analyze constraint system
        metadata.is_under_constrained = Some(self.is_under_constrained(&metadata));
        metadata.is_over_constrained = Some(self.is_over_constrained(&metadata));

        ConstraintSystem {
            constraints: circuit.constraints.clone(),
            signals,
            metadata,
        }
    }

    fn classify_constraint(&self, constraint: &Constraint, metadata: &mut ConstraintMetadata) {
        let degree = self.get_expression_degree(&constraint.left)
            + self.get_expression_degree(&constraint.right);

        match degree {
            0 => metadata.constant_constraints += 1,
            1 => metadata.linear_constraints += 1,
            2 => metadata.quadratic_constraints += 1,
            _ => metadata.quadratic_constraints += 1, // Higher degree treated as quadratic for R1CS
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
                    crate::core::parser::BinaryOperator::Pow => {
                        // For power, we need to check if exponent is constant
                        left_degree * right_degree // Simplified
                    }
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

    fn link_signal_constraints(
        &self,
        constraint: &Constraint,
        constraint_idx: usize,
        signals: &mut HashMap<String, SignalInfo>,
    ) {
        let signal_names = self.extract_signal_names(&constraint.left);
        let signal_names_right = self.extract_signal_names(&constraint.right);
        
        for signal_name in signal_names.into_iter().chain(signal_names_right) {
            if let Some(signal_info) = signals.get_mut(&signal_name) {
                signal_info.constraints_involved.push(constraint_idx);
            }
        }
    }

    fn extract_signal_names(&self, expr: &Expression) -> Vec<String> {
        match expr {
            Expression::Signal(name) => vec![name.clone()],
            Expression::BinaryOp(left, _, right) => {
                let mut names = self.extract_signal_names(left);
                names.extend(self.extract_signal_names(right));
                names
            }
            Expression::UnaryOp(_, inner) => self.extract_signal_names(inner),
            Expression::FunctionCall(_, args) => {
                args.iter().flat_map(|a| self.extract_signal_names(a)).collect()
            }
            Expression::ComponentOutput(component, signal) => vec![format!("{}.{}", component, signal)],
            Expression::Constant(_) => vec![],
        }
    }

    fn is_under_constrained(&self, metadata: &ConstraintMetadata) -> bool {
        // Heuristic: if there are more witness signals than constraints
        metadata.witness_signals > metadata.total_constraints
    }

    fn is_over_constrained(&self, metadata: &ConstraintMetadata) -> bool {
        // Heuristic: if there are more constraints than signals
        metadata.total_constraints > metadata.total_signals * 2
    }
}

impl Default for ConstraintExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extractor_creation() {
        let extractor = ConstraintExtractor::new();
        assert!(true);
    }
}
