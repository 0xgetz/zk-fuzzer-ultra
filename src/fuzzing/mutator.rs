//! Input mutation strategies for fuzzing

use num_bigint::BigInt;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Mutation strategy types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MutationStrategy {
    BitFlip,
    ByteFlip,
    Arithmetic,
    InterestingValues,
    DictionaryBased,
    Crossover,
}

/// Mutator for generating variant inputs
pub struct Mutator {
    rng: ChaCha8Rng,
    strategies: Vec<MutationStrategy>,
    interesting_values: Vec<BigInt>,
    mutation_count: usize,
}

/// Result of a mutation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationResult {
    pub original: Vec<BigInt>,
    pub mutated: Vec<BigInt>,
    pub strategy_used: MutationStrategy,
    pub genes_modified: Vec<usize>,
    pub success: bool,
}

impl Mutator {
    pub fn new() -> Self {
        Self {
            rng: ChaCha8Rng::from_entropy(),
            strategies: vec![
                MutationStrategy::BitFlip,
                MutationStrategy::ByteFlip,
                MutationStrategy::Arithmetic,
                MutationStrategy::InterestingValues,
            ],
            interesting_values: Self::generate_interesting_values(),
            mutation_count: 0,
        }
    }

    fn generate_interesting_values() -> Vec<BigInt> {
        vec![
            BigInt::from(0),
            BigInt::from(1),
            BigInt::from(-1),
            BigInt::from(i32::MIN),
            BigInt::from(i32::MAX),
            BigInt::from(i64::MIN),
            BigInt::from(i64::MAX),
        ]
    }

    /// Mutate an input vector using a random strategy
    pub fn mutate(&mut self, input: &[BigInt]) -> MutationResult {
        if input.is_empty() {
            return MutationResult {
                original: input.to_vec(),
                mutated: vec![],
                strategy_used: MutationStrategy::BitFlip,
                genes_modified: vec![],
                success: false,
            };
        }

        let strategy = self.strategies[self.rng.gen_range(0..self.strategies.len())].clone();
        let mutated = match strategy {
            MutationStrategy::BitFlip => self.bit_flip_mutation(input),
            MutationStrategy::ByteFlip => self.byte_flip_mutation(input),
            MutationStrategy::Arithmetic => self.arithmetic_mutation(input),
            MutationStrategy::InterestingValues => self.interesting_values_mutation(input),
            MutationStrategy::DictionaryBased => input.to_vec(),
            MutationStrategy::Crossover => input.to_vec(),
        };

        let genes_modified = self.identify_modified_genes(input, &mutated);
        self.mutation_count += 1;

        MutationResult {
            original: input.to_vec(),
            mutated,
            strategy_used: strategy,
            genes_modified,
            success: true,
        }
    }

    fn bit_flip_mutation(&mut self, input: &[BigInt]) -> Vec<BigInt> {
        let mut result = input.to_vec();
        let idx = self.rng.gen_range(0..result.len());
        
        let mut bytes = result[idx].to_bytes_le().1;
        if !bytes.is_empty() {
            let byte_idx = self.rng.gen_range(0..bytes.len());
            let bit_idx = self.rng.gen_range(0..8);
            bytes[byte_idx] ^= 1 << bit_idx;
            result[idx] = BigInt::from_bytes_le(num_bigint::Sign::Plus, &bytes);
        }
        
        result
    }

    fn byte_flip_mutation(&mut self, input: &[BigInt]) -> Vec<BigInt> {
        let mut result = input.to_vec();
        let idx = self.rng.gen_range(0..result.len());
        
        let mut bytes = result[idx].to_bytes_le().1;
        if !bytes.is_empty() {
            let byte_idx = self.rng.gen_range(0..bytes.len());
            bytes[byte_idx] = !bytes[byte_idx];
            result[idx] = BigInt::from_bytes_le(num_bigint::Sign::Plus, &bytes);
        }
        
        result
    }

    fn arithmetic_mutation(&mut self, input: &[BigInt]) -> Vec<BigInt> {
        let mut result = input.to_vec();
        let idx = self.rng.gen_range(0..result.len());
        
        let operation = self.rng.gen_range(0..4);
        match operation {
            0 => result[idx] = &result[idx] + 1,
            1 => result[idx] = &result[idx] - 1,
            2 => result[idx] = &result[idx] * 2,
            3 => result[idx] = &result[idx] / 2,
            _ => {}
        }
        
        result
    }

    fn interesting_values_mutation(&mut self, input: &[BigInt]) -> Vec<BigInt> {
        let mut result = input.to_vec();
        let idx = self.rng.gen_range(0..result.len());
        let interesting_idx = self.rng.gen_range(0..self.interesting_values.len());
        result[idx] = self.interesting_values[interesting_idx].clone();
        result
    }

    fn identify_modified_genes(&self, original: &[BigInt], mutated: &[BigInt]) -> Vec<usize> {
        let max_len = std::cmp::max(original.len(), mutated.len());
        let mut modified = Vec::new();
        
        for i in 0..max_len {
            let orig_val = original.get(i);
            let mut_val = mutated.get(i);
            
            if orig_val != mut_val {
                modified.push(i);
            }
        }
        
        modified
    }

    /// Get mutation statistics
    pub fn get_stats(&self) -> MutationStats {
        MutationStats {
            total_mutations: self.mutation_count,
            strategies_available: self.strategies.len(),
        }
    }
}

impl Default for Mutator {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about mutations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationStats {
    pub total_mutations: usize,
    pub strategies_available: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutator_creation() {
        let mutator = Mutator::new();
        assert_eq!(mutator.strategies.len(), 4);
    }

    #[test]
    fn test_mutation() {
        let mut mutator = Mutator::new();
        let input = vec![BigInt::from(42), BigInt::from(100)];
        let result = mutator.mutate(&input);
        assert!(result.success);
    }
}
