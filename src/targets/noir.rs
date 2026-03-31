//! Noir target support
//!
//! This module provides specific support for fuzzing Noir circuits.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Noir-specific fuzzer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoirFuzzerConfig {
    pub backend: NoirBackend,
    pub acir_version: String,
    pub brillig_enabled: bool,
    pub custom_functions: HashMap<String, FunctionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NoirBackend {
    Acir,
    Brillig,
    Marlin,
    Plonk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: String,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub name: String,
    pub parameter_type: String,
}

/// Noir-specific bug types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NoirBugType {
    TypeMismatch,
    VisibilityViolation,
    ConstraintSystemFailure,
    BrilligRuntimeError,
    InvalidWitness,
    ACIREncodingError,
}

impl NoirFuzzerConfig {
    pub fn new() -> Self {
        Self {
            backend: NoirBackend::Acir,
            acir_version: "0.30".to_string(),
            brillig_enabled: true,
            custom_functions: HashMap::new(),
        }
    }

    pub fn add_function(&mut self, function: FunctionInfo) {
        self.custom_functions.insert(function.name.clone(), function);
    }
}

impl Default for NoirFuzzerConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = NoirFuzzerConfig::new();
        assert_eq!(config.backend, NoirBackend::Acir);
    }
}
