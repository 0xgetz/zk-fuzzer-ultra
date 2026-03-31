//! Input generation strategies for ZK circuit fuzzing

use num_bigint::BigInt;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Input generation strategy types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GenerationStrategy {
    Random,
    Boundary,
    Structured,
    PropertyBased,
    Focused,
    Adaptive,
}

/// Input generator for ZK circuits
pub struct InputGenerator {
    rng: ChaCha8Rng,
    strategy: GenerationStrategy,
    field_modulus: Option<BigInt>,
    generation_count: usize,
}

/// Generated input set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedInput {
    pub values: HashMap<String, BigInt>,
    pub strategy_used: GenerationStrategy,
    pub metadata: InputMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMetadata {
    pub generation_id: usize,
    pub timestamp: u64,
    pub seed: u64,
    pub notes: String,
}

impl InputGenerator {
    pub fn new() -> Self {
        Self {
            rng: ChaCha8Rng::from_entropy(),
            strategy: GenerationStrategy::Random,
            field_modulus: None,
            generation_count: 0,
        }
    }

    /// Set the generation strategy
    pub fn set_strategy(&mut self, strategy: GenerationStrategy) {
        self.strategy = strategy;
    }

    /// Set the field modulus for ZK circuits
    pub fn set_field_modulus(&mut self, modulus: BigInt) {
        self.field_modulus = Some(modulus);
    }

    /// Generate a single input set
    pub fn generate_input(&mut self, signal_names: &[&str]) -> GeneratedInput {
        let mut values = HashMap::new();
        let seed = self.rng.gen::<u64>();

        for signal_name in signal_names {
            let value = self.generate_random_value();
            values.insert(signal_name.to_string(), value);
        }

        self.generation_count += 1;

        GeneratedInput {
            values,
            strategy_used: self.strategy.clone(),
            metadata: InputMetadata {
                generation_id: self.generation_count,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                seed,
                notes: format!("Generated using {:?}", self.strategy),
            },
        }
    }

    /// Generate multiple input sets
    pub fn generate_batch(&mut self, signal_names: &[&str], count: usize) -> Vec<GeneratedInput> {
        (0..count).map(|_| self.generate_input(signal_names)).collect()
    }

    fn generate_random_value(&mut self) -> BigInt {
        let random_bytes: [u8; 32] = self.rng.gen();
        let value = BigInt::from_bytes_le(num_bigint::Sign::Plus, &random_bytes);
        
        if let Some(modulus) = &self.field_modulus {
            value % modulus
        } else {
            value
        }
    }

    /// Get generation statistics
    pub fn get_stats(&self) -> GenerationStats {
        GenerationStats {
            total_generated: self.generation_count,
            strategy: self.strategy.clone(),
        }
    }
}

impl Default for InputGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about input generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationStats {
    pub total_generated: usize,
    pub strategy: GenerationStrategy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_creation() {
        let generator = InputGenerator::new();
        assert_eq!(generator.strategy, GenerationStrategy::Random);
    }

    #[test]
    fn test_generate_input() {
        let mut generator = InputGenerator::new();
        let signal_names = ["input_a", "input_b", "input_c"];
        let input = generator.generate_input(&signal_names);
        assert_eq!(input.values.len(), 3);
    }
}
