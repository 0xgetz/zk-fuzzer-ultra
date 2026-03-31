//! Genetic algorithm fuzzer for ZK circuits

use num_bigint::BigInt;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Individual in the genetic algorithm population
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Individual {
    pub genome: Vec<BigInt>,
    pub fitness: f64,
    pub generation: usize,
    pub metadata: IndividualMetadata,
}

/// Metadata about an individual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualMetadata {
    pub id: usize,
    pub parent_ids: Vec<usize>,
    pub mutation_count: usize,
    pub crossover_count: usize,
    pub bugs_found: Vec<BugReport>,
}

/// Bug report discovered during fuzzing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugReport {
    pub bug_type: BugType,
    pub severity: BugSeverity,
    pub description: String,
    pub input_values: HashMap<String, BigInt>,
    pub constraint_index: Option<usize>,
    pub trace_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BugType {
    UnderConstrained,
    OverConstrained,
    ConstraintViolation,
    DivisionByZero,
    Overflow,
    AbnormalTermination,
    InconsistentTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BugSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

/// Genetic algorithm fuzzer
pub struct GeneticFuzzer {
    population_size: usize,
    mutation_rate: f64,
    crossover_rate: f64,
    elitism_rate: f64,
    max_generations: usize,
    rng: ChaCha8Rng,
    population: Vec<Individual>,
    best_individual: Option<Individual>,
    generation: usize,
    bugs_found: Vec<BugReport>,
}

impl GeneticFuzzer {
    pub fn new() -> Self {
        Self {
            population_size: 100,
            mutation_rate: 0.1,
            crossover_rate: 0.7,
            elitism_rate: 0.1,
            max_generations: 1000,
            rng: ChaCha8Rng::from_entropy(),
            population: Vec::new(),
            best_individual: None,
            generation: 0,
            bugs_found: Vec::new(),
        }
    }

    /// Configure fuzzer parameters
    pub fn configure(
        &mut self,
        population_size: usize,
        mutation_rate: f64,
        crossover_rate: f64,
        max_generations: usize,
    ) {
        self.population_size = population_size;
        self.mutation_rate = mutation_rate;
        self.crossover_rate = crossover_rate;
        self.max_generations = max_generations;
    }

    /// Initialize population with random individuals
    pub fn initialize_population(&mut self, genome_length: usize) {
        self.population.clear();
        
        for i in 0..self.population_size {
            let genome = self.generate_random_genome(genome_length);
            let individual = Individual {
                genome,
                fitness: 0.0,
                generation: 0,
                metadata: IndividualMetadata {
                    id: i,
                    parent_ids: vec![],
                    mutation_count: 0,
                    crossover_count: 0,
                    bugs_found: vec![],
                },
            };
            self.population.push(individual);
        }
    }

    fn generate_random_genome(&mut self, length: usize) -> Vec<BigInt> {
        (0..length)
            .map(|_| {
                let random_bytes: [u8; 32] = self.rng.gen();
                BigInt::from_bytes_le(num_bigint::Sign::Plus, &random_bytes)
            })
            .collect()
    }

    /// Evaluate fitness of the population
    pub fn evaluate_population<F>(&mut self, fitness_fn: F) -> Result<(), String>
    where
        F: Fn(&Vec<BigInt>) -> f64 + Send + Sync,
    {
        let fitness_values: Vec<f64> = self.population
            .par_iter()
            .map(|individual| fitness_fn(&individual.genome))
            .collect();

        for (i, individual) in self.population.iter_mut().enumerate() {
            individual.fitness = fitness_values[i];
            
            if let Some(best) = &self.best_individual {
                if individual.fitness > best.fitness {
                    self.best_individual = Some(individual.clone());
                }
            } else {
                self.best_individual = Some(individual.clone());
            }
        }

        Ok(())
    }

    /// Get the best individual found
    pub fn get_best_individual(&self) -> Option<&Individual> {
        self.best_individual.as_ref()
    }

    /// Get all bugs found during fuzzing
    pub fn get_bugs_found(&self) -> &Vec<BugReport> {
        &self.bugs_found
    }

    /// Get current generation number
    pub fn get_generation(&self) -> usize {
        self.generation
    }

    /// Get population statistics
    pub fn get_population_stats(&self) -> PopulationStats {
        if self.population.is_empty() {
            return PopulationStats::default();
        }

        let fitness_values: Vec<f64> = self.population.iter().map(|i| i.fitness).collect();
        let avg_fitness = fitness_values.iter().sum::<f64>() / fitness_values.len() as f64;
        let max_fitness = fitness_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_fitness = fitness_values.iter().cloned().fold(f64::INFINITY, f64::min);

        PopulationStats {
            generation: self.generation,
            population_size: self.population_size,
            avg_fitness,
            max_fitness,
            min_fitness,
            diversity: 0.0,
        }
    }

    /// Add a bug report
    pub fn add_bug_report(&mut self, bug: BugReport) {
        self.bugs_found.push(bug);
    }
}

impl Default for GeneticFuzzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the population
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationStats {
    pub generation: usize,
    pub population_size: usize,
    pub avg_fitness: f64,
    pub max_fitness: f64,
    pub min_fitness: f64,
    pub diversity: f64,
}

impl Default for PopulationStats {
    fn default() -> Self {
        Self {
            generation: 0,
            population_size: 0,
            avg_fitness: 0.0,
            max_fitness: 0.0,
            min_fitness: 0.0,
            diversity: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzer_creation() {
        let fuzzer = GeneticFuzzer::new();
        assert_eq!(fuzzer.population_size, 100);
    }
}
