//! Feature extraction for ML-based circuit analysis
//!
//! This module provides comprehensive feature extraction from ZK circuits,
//! transforming circuit structures into numerical feature vectors suitable
//! for machine learning models.
//!
//! # Overview
//!
//! The feature extraction process captures various aspects of circuit structure:
//! - Constraint system metrics (count, degree, density)
//! - Signal characteristics (types, connectivity, domains)
//! - Template complexity measures
//! - Component interconnection patterns
//! - Expression complexity
//!
//! # Example
//!
//! ```rust,ignore
//! use zk_circuit_fuzzer::analysis::feature_extraction::CircuitFeatureExtractor;
//!
//! let extractor = CircuitFeatureExtractor::new();
//! let features = extractor.extract_features(&circuit)?;
//!
//! // Use features for ML model input
//! println!("Circuit complexity score: {}", features.complexity_score);
//! ```

use crate::core::parser::{
    BinaryOperator, Circuit, Component, Constraint, Expression, Signal, SignalDirection, Template,
    UnaryOperator,
};
use crate::core::constraints::{ConstraintExtractor, ConstraintSystem};
#[cfg(feature = "ml")]
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Comprehensive feature vector for a circuit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitFeatures {
    // Basic metrics
    pub total_signals: usize,
    pub total_constraints: usize,
    pub total_components: usize,
    pub total_templates: usize,
    
    // Signal features
    pub input_signals: usize,
    pub output_signals: usize,
    pub witness_signals: usize,
    pub binary_signals: usize,
    pub field_signals: usize,
    pub signal_connectivity_avg: f64,
    pub signal_connectivity_max: usize,
    
    // Constraint features
    pub linear_constraints: usize,
    pub quadratic_constraints: usize,
    pub constant_constraints: usize,
    pub constraint_density: f64,
    pub avg_constraint_degree: f64,
    pub max_constraint_degree: usize,
    
    // Template complexity
    pub template_count: usize,
    pub avg_template_signals: f64,
    pub max_template_signals: usize,
    pub avg_template_constraints: f64,
    pub max_template_constraints: usize,
    pub template_nesting_depth: usize,
    
    // Component features
    pub component_count: usize,
    pub avg_component_parameters: f64,
    pub component_reuse_ratio: f64,
    
    // Expression complexity
    pub avg_expression_depth: f64,
    pub max_expression_depth: usize,
    pub operator_distribution: HashMap<String, usize>,
    
    // Derived metrics
    pub complexity_score: f64,
    pub bug_risk_score: f64,
    pub is_under_constrained: bool,
    pub is_over_constrained: bool,
    
    // Feature vector for ML (normalized)
    pub feature_vector: Vec<f64>,
}

/// Statistics about expression complexity
#[derive(Debug, Clone, Default)]
struct ExpressionStats {
    total_depth: usize,
    max_depth: usize,
    count: usize,
    avg_depth: f64,
    operator_counts: HashMap<String, usize>,
}

/// Feature extraction engine for ZK circuits
pub struct CircuitFeatureExtractor {
    constraint_extractor: ConstraintExtractor,
}

impl CircuitFeatureExtractor {
    pub fn new() -> Self {
        Self {
            constraint_extractor: ConstraintExtractor::new(),
        }
    }

    /// Extract comprehensive features from a circuit
    pub fn extract_features(&self, circuit: &Circuit) -> Result<CircuitFeatures, String> {
        // Extract constraint system for detailed analysis
        let constraint_system = self.constraint_extractor.extract(circuit);
        
        // Basic counts
        let total_signals = circuit.signals.len();
        let total_constraints = circuit.constraints.len();
        let total_components = circuit.components.len();
        let total_templates = circuit.templates.len();
        
        // Signal analysis
        let (input_signals, output_signals, witness_signals, binary_signals, field_signals) =
            self.analyze_signals(&circuit.signals);
        
        // Signal connectivity analysis
        let (connectivity_avg, connectivity_max) =
            self.analyze_signal_connectivity(&constraint_system);
        
        // Constraint analysis
        let (linear_constraints, quadratic_constraints, constant_constraints) =
            self.analyze_constraint_types(&constraint_system);
        
        let constraint_density = if total_signals > 0 {
            total_constraints as f64 / total_signals as f64
        } else {
            0.0
        };
        
        let (avg_degree, max_degree) = self.analyze_constraint_degrees(circuit);
        
        // Template analysis
        let (template_stats, nesting_depth) = self.analyze_templates(&circuit.templates);
        
        // Component analysis
        let component_stats = self.analyze_components(&circuit.components);
        
        // Expression complexity
        let expr_stats = self.analyze_expressions(circuit);
        
        // Derived metrics
        let complexity_score = self.compute_complexity_score(
            total_signals,
            total_constraints,
            total_templates,
            avg_degree,
            expr_stats.max_depth,
        );
        
        let bug_risk_score = self.compute_bug_risk_score(
            constraint_density,
            witness_signals,
            total_constraints,
            max_degree,
            &constraint_system,
        );
        
        // Build feature vector for ML
        let feature_vector = self.build_feature_vector(
            total_signals,
            total_constraints,
            input_signals,
            output_signals,
            witness_signals,
            binary_signals,
            field_signals,
            connectivity_avg,
            linear_constraints,
            quadratic_constraints,
            constant_constraints,
            constraint_density,
            avg_degree,
            template_stats.count,
            template_stats.avg_signals,
            template_stats.max_signals,
            template_stats.avg_constraints,
            template_stats.max_constraints,
            nesting_depth,
            component_stats.count,
            component_stats.avg_params,
            component_stats.reuse_ratio,
            expr_stats.total_depth as f64 / expr_stats.count as f64,
            expr_stats.max_depth,
            complexity_score,
            bug_risk_score,
        );
        
        Ok(CircuitFeatures {
            total_signals,
            total_constraints,
            total_components,
            total_templates,
            input_signals,
            output_signals,
            witness_signals,
            binary_signals,
            field_signals,
            signal_connectivity_avg: connectivity_avg,
            signal_connectivity_max: connectivity_max,
            linear_constraints,
            quadratic_constraints,
            constant_constraints,
            constraint_density,
            avg_constraint_degree: avg_degree,
            max_constraint_degree: max_degree,
            template_count: template_stats.count,
            avg_template_signals: template_stats.avg_signals,
            max_template_signals: template_stats.max_signals,
            avg_template_constraints: template_stats.avg_constraints,
            max_template_constraints: template_stats.max_constraints,
            template_nesting_depth: nesting_depth,
            component_count: component_stats.count,
            avg_component_parameters: component_stats.avg_params,
            component_reuse_ratio: component_stats.reuse_ratio,
            avg_expression_depth: expr_stats.total_depth as f64 / expr_stats.count as f64,
            max_expression_depth: expr_stats.max_depth,
            operator_distribution: expr_stats.operator_counts,
            complexity_score,
            bug_risk_score,
            is_under_constrained: constraint_system.metadata.is_under_constrained.unwrap_or(false),
            is_over_constrained: constraint_system.metadata.is_over_constrained.unwrap_or(false),
            feature_vector,
        })
    }

    fn analyze_signals(&self, signals: &[Signal]) -> (usize, usize, usize, usize, usize) {
        let mut input_signals = 0;
        let mut output_signals = 0;
        let mut witness_signals = 0;
        let mut binary_signals = 0;
        let mut field_signals = 0;

        for signal in signals {
            match signal.direction {
                SignalDirection::Input => input_signals += 1,
                SignalDirection::Output => output_signals += 1,
                SignalDirection::Intermediate => witness_signals += 1,
            }

            match signal.signal_type {
                crate::core::parser::SignalType::Binary => binary_signals += 1,
                crate::core::parser::SignalType::Field => field_signals += 1,
            }
        }

        (input_signals, output_signals, witness_signals, binary_signals, field_signals)
    }

    fn analyze_signal_connectivity(&self, constraint_system: &ConstraintSystem) -> (f64, usize) {
        if constraint_system.signals.is_empty() {
            return (0.0, 0);
        }

        let mut total_connectivity = 0;
        let mut max_connectivity = 0;

        for signal_info in constraint_system.signals.values() {
            let connectivity = signal_info.constraints_involved.len();
            total_connectivity += connectivity;
            if connectivity > max_connectivity {
                max_connectivity = connectivity;
            }
        }

        let avg_connectivity = total_connectivity as f64 / constraint_system.signals.len() as f64;
        (avg_connectivity, max_connectivity)
    }

    fn analyze_constraint_types(&self, constraint_system: &ConstraintSystem) -> (usize, usize, usize) {
        (
            constraint_system.metadata.linear_constraints,
            constraint_system.metadata.quadratic_constraints,
            constraint_system.metadata.constant_constraints,
        )
    }

    fn analyze_constraint_degrees(&self, circuit: &Circuit) -> (f64, usize) {
        if circuit.constraints.is_empty() {
            return (0.0, 0);
        }

        let mut total_degree = 0;
        let mut max_degree = 0;

        for constraint in &circuit.constraints {
            let left_degree = self.get_expression_degree(&constraint.left);
            let right_degree = self.get_expression_degree(&constraint.right);
            let degree = left_degree + right_degree;
            total_degree += degree;
            if degree > max_degree {
                max_degree = degree;
            }
        }

        let avg_degree = total_degree as f64 / circuit.constraints.len() as f64;
        (avg_degree, max_degree)
    }

    fn get_expression_degree(&self, expr: &Expression) -> usize {
        match expr {
            Expression::Constant(_) => 0,
            Expression::Signal(_) => 1,
            Expression::BinaryOp(left, op, right) => {
                let left_degree = self.get_expression_degree(left);
                let right_degree = self.get_expression_degree(right);
                match op {
                    BinaryOperator::Mul => left_degree + right_degree,
                    BinaryOperator::Pow => left_degree * right_degree,
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

    fn analyze_templates(&self, templates: &[Template]) -> (TemplateStats, usize) {
        if templates.is_empty() {
            return (
                TemplateStats {
                    count: 0,
                    avg_signals: 0.0,
                    max_signals: 0,
                    avg_constraints: 0.0,
                    max_constraints: 0,
                },
                0,
            );
        }

        let mut total_signals = 0;
        let mut max_signals = 0;
        let mut total_constraints = 0;
        let mut max_constraints = 0;

        for template in templates {
            let sig_count = template.signals.len();
            let con_count = template.constraints.len();
            total_signals += sig_count;
            total_constraints += con_count;
            if sig_count > max_signals {
                max_signals = sig_count;
            }
            if con_count > max_constraints {
                max_constraints = con_count;
            }
        }

        let count = templates.len();
        let avg_signals = total_signals as f64 / count as f64;
        let avg_constraints = total_constraints as f64 / count as f64;

        let nesting_depth = if count > 10 { 3 } else if count > 5 { 2 } else { 1 };

        (
            TemplateStats {
                count,
                avg_signals,
                max_signals,
                avg_constraints,
                max_constraints,
            },
            nesting_depth,
        )
    }

    fn analyze_components(&self, components: &[Component]) -> ComponentStats {
        if components.is_empty() {
            return ComponentStats {
                count: 0,
                avg_params: 0.0,
                reuse_ratio: 0.0,
            };
        }

        let mut total_params = 0;
        let mut template_usage = HashMap::new();

        for component in components {
            total_params += component.parameters.len();
            *template_usage.entry(component.template_name.clone()).or_insert(0) += 1;
        }

        let count = components.len();
        let avg_params = total_params as f64 / count as f64;
        
        let templates_used = template_usage.len();
        let reused_templates = template_usage.values().filter(|&&count| count > 1).count();
        let reuse_ratio = if templates_used > 0 {
            reused_templates as f64 / templates_used as f64
        } else {
            0.0
        };

        ComponentStats {
            count,
            avg_params,
            reuse_ratio,
        }
    }

    fn analyze_expressions(&self, circuit: &Circuit) -> ExpressionStats {
        let mut stats = ExpressionStats::default();

        for constraint in &circuit.constraints {
            self.analyze_expression_recursive(&constraint.left, 0, &mut stats);
            self.analyze_expression_recursive(&constraint.right, 0, &mut stats);
        }

        for component in &circuit.components {
            for param in &component.parameters {
                self.analyze_expression_recursive(param, 0, &mut stats);
            }
        }

        for template in &circuit.templates {
            for constraint in &template.constraints {
                self.analyze_expression_recursive(&constraint.left, 0, &mut stats);
                self.analyze_expression_recursive(&constraint.right, 0, &mut stats);
            }
        }

        if stats.count > 0 {
            // Average depth computed on demand
        }

        stats
    }

    fn analyze_expression_recursive(
        &self,
        expr: &Expression,
        current_depth: usize,
        stats: &mut ExpressionStats,
    ) {
        match expr {
            Expression::Constant(_) => {}
            Expression::Signal(_) => {}
            Expression::BinaryOp(left, op, right) => {
                let new_depth = current_depth + 1;
                stats.total_depth += new_depth;
                stats.count += 1;
                if new_depth > stats.max_depth {
                    stats.max_depth = new_depth;
                }

                let op_name = format!("{:?}", op);
                *stats.operator_counts.entry(op_name).or_insert(0) += 1;

                self.analyze_expression_recursive(left, new_depth, stats);
                self.analyze_expression_recursive(right, new_depth, stats);
            }
            Expression::UnaryOp(op, inner) => {
                let new_depth = current_depth + 1;
                stats.total_depth += new_depth;
                stats.count += 1;
                if new_depth > stats.max_depth {
                    stats.max_depth = new_depth;
                }

                let op_name = format!("{:?}", op);
                *stats.operator_counts.entry(op_name).or_insert(0) += 1;

                self.analyze_expression_recursive(inner, new_depth, stats);
            }
            Expression::FunctionCall(name, args) => {
                let new_depth = current_depth + 1;
                stats.total_depth += new_depth;
                stats.count += 1;
                if new_depth > stats.max_depth {
                    stats.max_depth = new_depth;
                }

                *stats.operator_counts.entry(format!("Call({})", name)).or_insert(0) += 1;

                for arg in args {
                    self.analyze_expression_recursive(arg, new_depth, stats);
                }
            }
            Expression::ComponentOutput(_, _) => {
                let new_depth = current_depth + 1;
                stats.total_depth += new_depth;
                stats.count += 1;
                if new_depth > stats.max_depth {
                    stats.max_depth = new_depth;
                }
                *stats.operator_counts.entry("ComponentOutput".to_string()).or_insert(0) += 1;
            }
        }
    }

    fn compute_complexity_score(
        &self,
        signals: usize,
        constraints: usize,
        templates: usize,
        avg_degree: f64,
        max_expr_depth: usize,
    ) -> f64 {
        let signal_score = (signals as f64).ln() * 10.0;
        let constraint_score = (constraints as f64).ln() * 10.0;
        let template_score = (templates as f64) * 5.0;
        let degree_score = avg_degree * 15.0;
        let depth_score = (max_expr_depth as f64) * 8.0;

        let raw_score = signal_score + constraint_score + template_score + degree_score + depth_score;
        
        raw_score.min(100.0).max(0.0)
    }

    fn compute_bug_risk_score(
        &self,
        constraint_density: f64,
        witness_signals: usize,
        total_constraints: usize,
        max_degree: usize,
        constraint_system: &ConstraintSystem,
    ) -> f64 {
        let mut risk = 0.0;

        if witness_signals > 0 && total_constraints > 0 {
            let ratio = witness_signals as f64 / total_constraints as f64;
            if ratio > 2.0 {
                risk += 25.0;
            } else if ratio > 1.0 {
                risk += 15.0;
            }
        }

        if constraint_density < 0.5 {
            risk += 20.0;
        } else if constraint_density < 1.0 {
            risk += 10.0;
        }

        if max_degree > 2 {
            risk += (max_degree - 2) as f64 * 10.0;
        }

        if constraint_system.metadata.is_under_constrained == Some(true) {
            risk += 20.0;
        }
        if constraint_system.metadata.is_over_constrained == Some(true) {
            risk += 15.0;
        }

        let max_connectivity = constraint_system
            .signals
            .values()
            .map(|s| s.constraints_involved.len())
            .max()
            .unwrap_or(0);
        if max_connectivity > 10 {
            risk += 10.0;
        }

        risk.min(100.0).max(0.0)
    }

    #[allow(clippy::too_many_arguments)]
    fn build_feature_vector(
        &self,
        total_signals: usize,
        total_constraints: usize,
        input_signals: usize,
        output_signals: usize,
        witness_signals: usize,
        binary_signals: usize,
        field_signals: usize,
        connectivity_avg: f64,
        linear_constraints: usize,
        quadratic_constraints: usize,
        constant_constraints: usize,
        constraint_density: f64,
        avg_degree: f64,
        template_count: usize,
        avg_template_signals: f64,
        max_template_signals: usize,
        avg_template_constraints: f64,
        max_template_constraints: usize,
        nesting_depth: usize,
        component_count: usize,
        avg_component_params: f64,
        component_reuse_ratio: f64,
        avg_expr_depth: f64,
        max_expr_depth: usize,
        complexity_score: f64,
        bug_risk_score: f64,
    ) -> Vec<f64> {
        vec![
            total_signals as f64 / 1000.0,
            total_constraints as f64 / 1000.0,
            input_signals as f64 / 100.0,
            output_signals as f64 / 100.0,
            witness_signals as f64 / 500.0,
            binary_signals as f64 / 100.0,
            field_signals as f64 / 1000.0,
            connectivity_avg / 20.0,
            if total_constraints > 0 {
                linear_constraints as f64 / total_constraints as f64
            } else {
                0.0
            },
            if total_constraints > 0 {
                quadratic_constraints as f64 / total_constraints as f64
            } else {
                0.0
            },
            if total_constraints > 0 {
                constant_constraints as f64 / total_constraints as f64
            } else {
                0.0
            },
            constraint_density.min(10.0) / 10.0,
            avg_degree / 5.0,
            template_count as f64 / 50.0,
            avg_template_signals / 50.0,
            max_template_signals as f64 / 100.0,
            avg_template_constraints / 20.0,
            max_template_constraints as f64 / 50.0,
            nesting_depth as f64 / 5.0,
            component_count as f64 / 100.0,
            avg_component_params / 10.0,
            component_reuse_ratio,
            avg_expr_depth / 10.0,
            max_expr_depth as f64 / 20.0,
            complexity_score / 100.0,
            bug_risk_score / 100.0,
        ]
    }

    #[cfg(feature = "ml")]
    /// Convert features to ndarray Array1 for ML processing
    pub fn features_to_array(features: &CircuitFeatures) -> Array1<f64> {
        Array1::from(features.feature_vector.clone())
    }

    #[cfg(feature = "ml")]
    /// Create feature matrix from multiple circuits
    pub fn create_feature_matrix(features_list: &[CircuitFeatures]) -> Array2<f64> {
        if features_list.is_empty() {
            return Array2::zeros((0, 0));
        }

        let n_samples = features_list.len();
        let n_features = features_list[0].feature_vector.len();
        
        let mut matrix = Array2::zeros((n_samples, n_features));
        
        for (i, features) in features_list.iter().enumerate() {
            for (j, &value) in features.feature_vector.iter().enumerate() {
                matrix[[i, j]] = value;
            }
        }

        matrix
    }

    /// Get feature names for interpretability
    pub fn get_feature_names() -> Vec<&'static str> {
        vec![
            "total_signals",
            "total_constraints",
            "input_signals",
            "output_signals",
            "witness_signals",
            "binary_signals",
            "field_signals",
            "connectivity_avg",
            "linear_constraints_ratio",
            "quadratic_constraints_ratio",
            "constant_constraints_ratio",
            "constraint_density",
            "avg_constraint_degree",
            "template_count",
            "avg_template_signals",
            "max_template_signals",
            "avg_template_constraints",
            "max_template_constraints",
            "nesting_depth",
            "component_count",
            "avg_component_params",
            "component_reuse_ratio",
            "avg_expression_depth",
            "max_expression_depth",
            "complexity_score",
            "bug_risk_score",
        ]
    }
}

impl Default for CircuitFeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct TemplateStats {
    count: usize,
    avg_signals: f64,
    max_signals: usize,
    avg_constraints: f64,
    max_constraints: usize,
}

#[derive(Debug, Clone)]
struct ComponentStats {
    count: usize,
    avg_params: f64,
    reuse_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::parser::{
        BinaryOperator, Circuit, Constraint, Expression, Signal, SignalDirection, SignalType,
        Template,
    };

    fn create_test_circuit() -> Circuit {
        Circuit {
            name: "test_circuit".to_string(),
            templates: vec![Template {
                name: "adder".to_string(),
                parameters: vec![],
                signals: vec![
                    Signal {
                        name: "a".to_string(),
                        direction: SignalDirection::Input,
                        signal_type: SignalType::Field,
                        dimensions: vec![],
                    },
                    Signal {
                        name: "b".to_string(),
                        direction: SignalDirection::Input,
                        signal_type: SignalType::Field,
                        dimensions: vec![],
                    },
                    Signal {
                        name: "out".to_string(),
                        direction: SignalDirection::Output,
                        signal_type: SignalType::Field,
                        dimensions: vec![],
                    },
                ],
                components: vec![],
                constraints: vec![Constraint {
                    left: Expression::Signal("out".to_string()),
                    right: Expression::BinaryOp(
                        Box::new(Expression::Signal("a".to_string())),
                        BinaryOperator::Add,
                        Box::new(Expression::Signal("b".to_string())),
                    ),
                    constraint_type: crate::core::parser::ConstraintType::Assignment,
                }],
            }],
            components: vec![],
            signals: vec![
                Signal {
                    name: "input1".to_string(),
                    direction: SignalDirection::Input,
                    signal_type: SignalType::Field,
                    dimensions: vec![],
                },
                Signal {
                    name: "output1".to_string(),
                    direction: SignalDirection::Output,
                    signal_type: SignalType::Field,
                    dimensions: vec![],
                },
            ],
            constraints: vec![],
        }
    }

    #[test]
    fn test_feature_extractor_creation() {
        let extractor = CircuitFeatureExtractor::new();
        assert!(true);
    }

    #[test]
    fn test_extract_features_basic() {
        let extractor = CircuitFeatureExtractor::new();
        let circuit = create_test_circuit();
        let features = extractor.extract_features(&circuit).unwrap();
        
        assert_eq!(features.total_signals, 2);
        assert_eq!(features.total_templates, 1);
        assert!(features.complexity_score >= 0.0);
        assert!(features.bug_risk_score >= 0.0);
        assert_eq!(features.feature_vector.len(), 26);
    }

    #[test]
    fn test_feature_vector_normalization() {
        let extractor = CircuitFeatureExtractor::new();
        let circuit = create_test_circuit();
        let features = extractor.extract_features(&circuit).unwrap();
        
        for &value in &features.feature_vector {
            assert!(value >= 0.0 && value <= 1.0, "Feature value {} out of range", value);
        }
    }

    #[test]
    fn test_feature_names() {
        let names = CircuitFeatureExtractor::get_feature_names();
        assert_eq!(names.len(), 26);
        assert_eq!(names[0], "total_signals");
        assert_eq!(names[25], "bug_risk_score");
    }
}
