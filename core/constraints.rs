//! Constraint extraction module
//!
//! Provides tools for extracting and analyzing constraint systems from Circom circuits.

use crate::core::parser::{CircomCircuit, Constraint, Expression, SignalDirection};
use num_bigint::BigInt;
use std::collections::HashMap;

/// Represents a constraint system extracted from a circuit
#[derive(Debug, Clone)]
pub struct ConstraintSystem {
    /// Number of wires in the system
    pub num_wires: usize,
    /// Number of constraints
    pub num_constraints: usize,
    /// Public inputs
    pub public_inputs: Vec<String>,
    /// Private inputs
    pub private_inputs: Vec<String>,
    /// All constraints
    pub constraints: Vec<Constraint>,
    /// Signal to wire mapping
    pub signal_map: HashMap<String, usize>,
}

/// Analysis result for constraint system
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintAnalysis {
    /// System is well-constrained
    WellConstrained,
    /// System has fewer constraints than needed
    UnderConstrained,
    /// System has more constraints than needed
    OverConstrained,
    /// System has conflicting constraints
    Inconsistent,
}

/// Error types for constraint extraction
#[derive(Debug)]
pub enum ConstraintError {
    ParseError(String),
    MissingSignal(String),
    InvalidConstraint(String),
    CircularDependency(String),
}

impl std::fmt::Display for ConstraintError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConstraintError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConstraintError::MissingSignal(sig) => write!(f, "Missing signal: {}", sig),
            ConstraintError::InvalidConstraint(msg) => write!(f, "Invalid constraint: {}", msg),
            ConstraintError::CircularDependency(dep) => write!(f, "Circular dependency: {}", dep),
        }
    }
}

impl std::error::Error for ConstraintError {}

/// Extracts constraint systems from Circom circuits
pub struct ConstraintExtractor {
    /// Current circuit being processed
    circuit: Option<CircomCircuit>,
    /// Extracted constraint system
    constraint_system: Option<ConstraintSystem>,
}

impl ConstraintExtractor {
    /// Create a new constraint extractor
    pub fn new() -> Self {
        ConstraintExtractor {
            circuit: None,
            constraint_system: None,
        }
    }

    /// Extract constraints from a parsed circuit
    pub fn extract(&mut self, circuit: CircomCircuit) -> Result<&ConstraintSystem, ConstraintError> {
        self.circuit = Some(circuit);
        
        let circuit = self.circuit.as_ref().unwrap();
        
        // Build signal mapping
        let mut signal_map = HashMap::new();
        let mut wire_counter = 0;
        
        // Map all signals to wires
        for signal in &circuit.signals {
            signal_map.insert(signal.name.clone(), wire_counter);
            wire_counter += 1;
        }
        
        // Separate public and private inputs
        let mut public_inputs = Vec::new();
        let mut private_inputs = Vec::new();
        
        for signal in &circuit.signals {
            match signal.direction {
                SignalDirection::Input => {
                    if signal.tags.contains(&"public".to_string()) {
                        public_inputs.push(signal.name.clone());
                    } else {
                        private_inputs.push(signal.name.clone());
                    }
                }
                _ => {}
            }
        }
        
        // Create constraint system
        let constraint_system = ConstraintSystem {
            num_wires: wire_counter,
            num_constraints: circuit.constraints.len(),
            public_inputs,
            private_inputs,
            constraints: circuit.constraints.clone(),
            signal_map,
        };
        
        self.constraint_system = Some(constraint_system);
        Ok(self.constraint_system.as_ref().unwrap())
    }

    /// Analyze the constraint system for consistency
    pub fn analyze(&self) -> Result<ConstraintAnalysis, ConstraintError> {
        let cs = self.constraint_system.as_ref()
            .ok_or_else(|| ConstraintError::ParseError("No constraint system available".to_string()))?;
        
        // Simple heuristic: compare number of constraints to number of wires
        // This is a placeholder for more sophisticated analysis
        let wire_ratio = cs.num_constraints as f64 / cs.num_wires as f64;
        
        if wire_ratio < 0.8 {
            Ok(ConstraintAnalysis::UnderConstrained)
        } else if wire_ratio > 1.2 {
            Ok(ConstraintAnalysis::OverConstrained)
        } else {
            Ok(ConstraintAnalysis::WellConstrained)
        }
    }

    /// Get the extracted constraint system
    pub fn get_constraint_system(&self) -> Option<&ConstraintSystem> {
        self.constraint_system.as_ref()
    }

    /// Check if a signal is constrained
    pub fn is_signal_constrained(&self, signal_name: &str) -> bool {
        if let Some(cs) = &self.constraint_system {
            cs.constraints.iter().any(|c| {
                self.expression_contains_signal(&c.left, signal_name) ||
                self.expression_contains_signal(&c.right, signal_name) ||
                c.output.as_deref() == Some(signal_name)
            })
        } else {
            false
        }
    }

    /// Helper to check if an expression contains a signal
    fn expression_contains_signal(&self, expr: &Expression, signal: &str) -> bool {
        match expr {
            Expression::Signal(name) => name == signal,
            Expression::Number(_) => false,
            Expression::Addition(left, right) |
            Expression::Subtraction(left, right) |
            Expression::Multiplication(left, right) |
            Expression::SubtractionResult(left, right) => {
                self.expression_contains_signal(left, signal) ||
                self.expression_contains_signal(right, signal)
            }
        }
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
    use crate::core::parser::{CircomParser, CircomCircuit};

    #[test]
    fn test_extractor_creation() {
        let extractor = ConstraintExtractor::new();
        assert!(extractor.get_constraint_system().is_none());
    }

    #[test]
    fn test_extract_constraints() {
        let circuit = CircomCircuit {
            name: "test".to_string(),
            templates: Vec::new(),
            components: Vec::new(),
            signals: vec![
                crate::core::parser::SignalDefinition {
                    name: "a".to_string(),
                    direction: SignalDirection::Input,
                    tags: vec!["public".to_string()],
                },
                crate::core::parser::SignalDefinition {
                    name: "b".to_string(),
                    direction: SignalDirection::Output,
                    tags: vec![],
                },
            ],
            constraints: vec![
                Constraint {
                    left: Expression::Signal("a".to_string()),
                    right: Expression::Signal("b".to_string()),
                    output: Some("b".to_string()),
                }
            ],
        };

        let mut extractor = ConstraintExtractor::new();
        let cs = extractor.extract(circuit).unwrap();
        
        assert_eq!(cs.num_wires, 2);
        assert_eq!(cs.num_constraints, 1);
        assert_eq!(cs.public_inputs.len(), 1);
    }
}
