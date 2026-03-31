//! Circom target support
//!
//! This module provides specific support for fuzzing Circom circuits.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Circom-specific fuzzer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircomFuzzerConfig {
    pub prime_field: String,
    pub version: CircomVersion,
    pub include_paths: Vec<String>,
    pub custom_templates: HashMap<String, TemplateInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CircomVersion {
    V1,
    V2,
    V2_1,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    pub name: String,
    pub parameters: Vec<ParameterInfo>,
    pub input_count: usize,
    pub output_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub name: String,
    pub parameter_type: String,
}

/// Circom-specific bug types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CircomBugType {
    SignalNotAssigned,
    MultipleAssignments,
    ComponentNotInstantiated,
    TemplateParameterMismatch,
    CircularSignalDependency,
    QuadraticConstraintViolation,
    InvalidFieldElement,
}

impl CircomFuzzerConfig {
    pub fn new() -> Self {
        Self {
            prime_field: "21888242871839275222246405745257275088548364400416034343698204186575808495617".to_string(), // BN256 prime
            version: CircomVersion::V2,
            include_paths: vec![],
            custom_templates: HashMap::new(),
        }
    }

    pub fn add_template(&mut self, template: TemplateInfo) {
        self.custom_templates.insert(template.name.clone(), template);
    }
}

impl Default for CircomFuzzerConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = CircomFuzzerConfig::new();
        assert_eq!(config.version, CircomVersion::V2);
    }
}
