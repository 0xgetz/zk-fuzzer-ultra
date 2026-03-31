// Targets module for ZK Circuit Fuzzer
// Provides abstractions for different ZK proof systems

pub mod circom;
pub mod gnark;
pub mod halo2;
pub mod noir;

use std::path::Path;
use std::fmt::Debug;

/// Trait for ZK circuit targets
pub trait ZKTarget: Debug + Send + Sync {
    /// Parse circuit from file or source
    fn parse<P: AsRef<Path>>(path: P) -> Result<Self, String>
    where
        Self: Sized;
    
    /// Get circuit name
    fn name(&self) -> &str;
    
    /// Get number of public inputs
    fn public_input_count(&self) -> usize;
    
    /// Get number of private inputs
    fn private_input_count(&self) -> usize;
    
    /// Get number of constraints
    fn constraint_count(&self) -> usize;
    
    /// Validate input against circuit constraints
    fn validate_input(&self, input: &[u8]) -> Result<bool, String>;
    
    /// Generate random valid input
    fn generate_random_input(&self) -> Vec<u8>;
    
    /// Get circuit metadata
    fn metadata(&self) -> CircuitMetadata;
}

/// Circuit metadata information
#[derive(Debug, Clone)]
pub struct CircuitMetadata {
    pub name: String,
    pub target_type: String,
    pub public_inputs: usize,
    pub private_inputs: usize,
    pub constraints: usize,
    pub wires: usize,
    pub gates: usize,
    pub version: String,
}

/// Target type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetType {
    Circom,
    Gnark,
    Halo2,
    Noir,
}

impl TargetType {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "circom" => Some(TargetType::Circom),
            "go" | "gnark" => Some(TargetType::Gnark),
            "rs" | "halo2" => Some(TargetType::Halo2),
            "nr" => Some(TargetType::Noir),
            _ => None,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            TargetType::Circom => "circom",
            TargetType::Gnark => "gnark",
            TargetType::Halo2 => "halo2",
            TargetType::Noir => "noir",
        }
    }
}

/// Factory for creating ZK targets
pub struct TargetFactory;

impl TargetFactory {
    /// Create a target from file path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Box<dyn ZKTarget>, String> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| "No file extension found")?;
        
        match TargetType::from_extension(extension) {
            Some(TargetType::Circom) => {
                let circuit = circom::CircomCircuit::parse(path)?;
                Ok(Box::new(circuit))
            }
            Some(TargetType::Gnark) => {
                let circuit = gnark::GnarkCircuit::parse(path)?;
                Ok(Box::new(circuit))
            }
            Some(TargetType::Halo2) => {
                let circuit = halo2::Halo2Circuit::parse(path)?;
                Ok(Box::new(circuit))
            }
            Some(TargetType::Noir) => {
                let circuit = noir::NoirCircuit::parse(path)?;
                Ok(Box::new(circuit))
            }
            None => Err(format!("Unsupported target type: {}", extension)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_target_type_from_extension() {
        assert_eq!(TargetType::from_extension("circom"), Some(TargetType::Circom));
        assert_eq!(TargetType::from_extension("go"), Some(TargetType::Gnark));
        assert_eq!(TargetType::from_extension("rs"), Some(TargetType::Halo2));
        assert_eq!(TargetType::from_extension("nr"), Some(TargetType::Noir));
        assert_eq!(TargetType::from_extension("txt"), None);
    }
    
    #[test]
    fn test_target_type_as_str() {
        assert_eq!(TargetType::Circom.as_str(), "circom");
        assert_eq!(TargetType::Gnark.as_str(), "gnark");
        assert_eq!(TargetType::Halo2.as_str(), "halo2");
        assert_eq!(TargetType::Noir.as_str(), "noir");
    }
}
