//! Integration Module (Phase 6)
//!
//! This module integrates all fuzzing components: genetic algorithms,
//! constraint solving, and property-based testing into a unified fuzzer.

use rand::Rng;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::constraint_solver::{ConstraintSolver, ConstraintExpr, SolveResult};
use crate::property_testing::{PropertyTester, InvariantChecker, Arbitrary};

/// Configuration for the integrated fuzzer
#[derive(Debug, Clone)]
pub struct IntegratedConfig {
    pub ga_config: Config,
    pub use_constraints: bool,
    pub use_property_testing: bool,
    pub max_constraint_attempts: usize,
    pub property_test_count: usize,
    pub invariant_strength: f64,
}

impl Default for IntegratedConfig {
    fn default() -> Self {
        Self {
            ga_config: Config::default(),
            use_constraints: true,
            use_property_testing: true,
            max_constraint_attempts: 10,
            property_test_count: 50,
            invariant_strength: 0.8,
        }
    }
}

/// Statistics from integrated fuzzing
#[derive(Debug, Clone)]
pub struct FuzzingStatistics {
    pub total_evaluations: usize,
    pub constraint_solves: usize,
    pub constraint_satisfiable: usize,
    pub property_tests_run: usize,
    pub property_tests_passed: usize,
    pub invariants_violated: usize,
    pub bugs_found: usize,
    pub elapsed_time: Duration,
}

impl FuzzingStatistics {
    pub fn new() -> Self {
        Self {
            total_evaluations: 0,
            constraint_solves: 0,
            constraint_satisfiable: 0,
            property_tests_run: 0,
            property_tests_passed: 0,
            invariants_violated: 0,
            bugs_found: 0,
            elapsed_time: Duration::from_secs(0),
        }
    }
}

impl Default for FuzzingStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Integrated fuzzer combining GA, constraint solving, and property testing
pub struct IntegratedFuzzer {
    config: IntegratedConfig,
    stats: FuzzingStatistics,
    constraint_solver: ConstraintSolver,
    property_tester: PropertyTester,
}

impl IntegratedFuzzer {
    pub fn new(config: IntegratedConfig) -> Self {
        Self {
            config,
            stats: FuzzingStatistics::new(),
            constraint_solver: ConstraintSolver::new(),
            property_tester: PropertyTester::new()
                .with_tests(config.property_test_count),
        }
    }
    
    /// Run the integrated fuzzing campaign
    pub fn run(&mut self) -> (GeneticAlgorithm, FuzzingStatistics) {
        let start_time = Instant::now();
        
        let mut ga = GeneticAlgorithm::new(self.config.ga_config.clone());
        
        // Phase 1: Constraint-guided initialization
        if self.config.use_constraints {
            self.constraint_guided_initialization(&mut ga);
        }
        
        // Phase 2: Property-based test generation
        if self.config.use_property_testing {
            self.property_based_generation(&mut ga);
        }
        
        // Phase 3: Main fuzzing loop with constraint guidance
        for generation in 0..self.config.ga_config.max_generations {
            self.stats.total_evaluations += 1;
            
            // Evaluate population
            ga.evaluate_generation();
            
            // Apply constraint-guided mutations
            if self.config.use_constraints && generation % 5 == 0 {
                self.constraint_guided_mutation(&mut ga);
            }
            
            // Run property tests on best individuals
            if self.config.use_property_testing && generation % 10 == 0 {
                self.test_properties(&ga);
            }
            
            // Evolve next generation
            ga.evolve();
        }
        
        self.stats.elapsed_time = start_time.elapsed();
        
        (ga, self.stats.clone())
    }
    
    /// Initialize population using constraint solving
    fn constraint_guided_initialization(&mut self, ga: &mut GeneticAlgorithm) {
        // Declare variables for constraint solving
        self.constraint_solver.declare_int_var("input1");
        self.constraint_solver.declare_int_var("input2");
        
        for _ in 0..self.config.max_constraint_attempts {
            self.stats.constraint_solves += 1;
            
            // Add random constraints
            let constraint = self.generate_random_constraint();
            self.constraint_solver.add_constraint(constraint);
            
            let result = self.constraint_solver.solve();
            
            if result.sat {
                self.stats.constraint_satisfiable += 1;
                
                // Convert solution to genome
                let genome = self.solution_to_genome(&result.model);
                ga.population.push(genome);
            }
        }
    }
    
    /// Generate random constraints for initialization
    fn generate_random_constraint(&self) -> ConstraintExpr {
        let mut rng = rand::thread_rng();
        
        // Simple random constraint: x + y == constant
        let constant = rng.gen_range(0..100);
        
        ConstraintExpr::BinOp {
            op: crate::constraint_solver::ConstraintType::Equality,
            left: Box::new(ConstraintExpr::Add(vec![
                ConstraintExpr::Var("input1".to_string()),
                ConstraintExpr::Var("input2".to_string()),
            ])),
            right: Box::new(ConstraintExpr::IntConst(constant as i64)),
        }
    }
    
    /// Convert constraint solver solution to genome
    fn solution_to_genome(&self, model: &HashMap<String, i64>) -> Genome {
        let mut rng = rand::thread_rng();
        
        // Extract values from model, defaulting to random if not found
        let input1 = model.get("input1").copied().unwrap_or_else(|| rng.gen::<i64>());
        let input2 = model.get("input2").copied().unwrap_or_else(|| rng.gen::<i64>());
        
        Genome {
            genes: vec![input1 as u64, input2 as u64],
            fitness: 0.0,
        }
    }
    
    /// Apply constraint-guided mutations
    fn constraint_guided_mutation(&mut self, ga: &mut GeneticAlgorithm) {
        for individual in &mut ga.population {
            if rand::thread_rng().gen_bool(0.3) {
                // Try to find a valid mutation using constraint solving
                self.constraint_solver.clear();
                self.constraint_solver.declare_int_var("x");
                
                // Add constraint that new value should be close to current
                let current = individual.genes[0] as i64;
                self.constraint_solver.add_constraint(ConstraintExpr::BinOp {
                    op: crate::constraint_solver::ConstraintType::LessThanOrEqual,
                    left: Box::new(ConstraintExpr::Sub(
                        Box::new(ConstraintExpr::Var("x".to_string())),
                        Box::new(ConstraintExpr::IntConst(current - 10)),
                    )),
                    right: Box::new(ConstraintExpr::IntConst(10)),
                });
                
                self.stats.constraint_solves += 1;
                let result = self.constraint_solver.solve();
                
                if result.sat {
                    self.stats.constraint_satisfiable += 1;
                    if let Some(&new_val) = result.model.get("x") {
                        individual.genes[0] = new_val as u64;
                    }
                }
            }
        }
    }
    
    /// Generate test cases using property-based testing
    fn property_based_generation(&mut self, ga: &mut GeneticAlgorithm) {
        let results = self.property_tester.test_property(
            |rng| u64::arbitrary(rng),
            |&x| x == u64::MAX || x < x.wrapping_add(1),
        );
        
        self.stats.property_tests_run += results.len();
        self.stats.property_tests_passed += results.iter().filter(|r| r.passed).count();
        
        // Add interesting test cases to population
        for result in &results {
            if !result.passed {
                // Failed property test - potential bug
                self.stats.bugs_found += 1;
                
                // Parse the input and add to population
                if let Ok(val) = result.input.parse::<u64>() {
                    ga.population.push(Genome {
                        genes: vec![val],
                        fitness: 0.0,
                    });
                }
            }
        }
    }
    
    /// Test properties on current population
    fn test_properties(&mut self, ga: &GeneticAlgorithm) {
        // Create invariant checker for circuit properties
        let checker = InvariantChecker::<Genome>::new()
            .invariant("non_negative", |g| g.genes.iter().all(|&x| x as i64 >= 0))
            .invariant("bounded", |g| g.genes.iter().all(|&x| x < 1_000_000));
        
        for individual in &ga.population {
            let violations = checker.check(individual);
            if !violations.is_empty() {
                self.stats.invariants_violated += 1;
            }
        }
    }
}

/// Simple genome representation
#[derive(Debug, Clone)]
pub struct Genome {
    pub genes: Vec<u64>,
    pub fitness: f64,
}

/// Simple genetic algorithm configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub population_size: usize,
    pub max_generations: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub elitism_count: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            population_size: 100,
            max_generations: 1000,
            mutation_rate: 0.1,
            crossover_rate: 0.7,
            elitism_count: 2,
        }
    }
}

/// Simple genetic algorithm
pub struct GeneticAlgorithm {
    pub population: Vec<Genome>,
    config: Config,
}

impl GeneticAlgorithm {
    pub fn new(config: Config) -> Self {
        let mut rng = rand::thread_rng();
        let population = (0..config.population_size)
            .map(|_| Genome {
                genes: (0..2).map(|_| rng.gen::<u64>()).collect(),
                fitness: 0.0,
            })
            .collect();
        
        Self { population, config }
    }
    
    pub fn evaluate_generation(&mut self) {
        let mut rng = rand::thread_rng();
        for individual in &mut self.population {
            // Simple fitness: sum of genes (for demo purposes)
            individual.fitness = individual.genes.iter().map(|&g| g as f64).sum();
            
            // Add some randomness to fitness
            individual.fitness += rng.gen_range(-0.1..0.1);
        }
    }
    
    pub fn evolve(&mut self) {
        // Sort by fitness (descending)
        self.population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
        
        // Elitism: keep best individuals
        let elites: Vec<Genome> = self.population[..self.config.elitism_count].to_vec();
        
        // Create new population
        let mut new_population = elites;
        
        while new_population.len() < self.config.population_size {
            // Selection
            let parent1 = self.tournament_select();
            let parent2 = self.tournament_select();
            
            // Crossover
            let mut child = if rand::thread_rng().gen_bool(self.config.crossover_rate) {
                self.crossover(&parent1, &parent2)
            } else {
                parent1.clone()
            };
            
            // Mutation
            self.mutate(&mut child);
            
            new_population.push(child);
        }
        
        self.population = new_population;
    }
    
    fn tournament_select(&self) -> Genome {
        let mut rng = rand::thread_rng();
        let tournament_size = 3;
        let mut best = &self.population[rng.gen_range(0..self.population.len())];
        
        for _ in 1..tournament_size {
            let contestant = &self.population[rng.gen_range(0..self.population.len())];
            if contestant.fitness > best.fitness {
                best = contestant;
            }
        }
        
        best.clone()
    }
    
    fn crossover(&self, parent1: &Genome, parent2: &Genome) -> Genome {
        let mut rng = rand::thread_rng();
        let point = rng.gen_range(0..parent1.genes.len());
        
        let mut genes = parent1.genes[..point].to_vec();
        genes.extend_from_slice(&parent2.genes[point..]);
        
        Genome {
            genes,
            fitness: 0.0,
        }
    }
    
    fn mutate(&self, genome: &mut Genome) {
        let mut rng = rand::thread_rng();
        
        for gene in &mut genome.genes {
            if rng.gen_bool(self.config.mutation_rate) {
                *gene = rng.gen();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrated_fuzzer() {
        let config = IntegratedConfig {
            ga_config: Config {
                population_size: 10,
                max_generations: 5,
                mutation_rate: 0.1,
                crossover_rate: 0.7,
                elitism_count: 1,
            },
            use_constraints: true,
            use_property_testing: true,
            max_constraint_attempts: 3,
            property_test_count: 5,
            invariant_strength: 0.8,
        };
        
        let mut fuzzer = IntegratedFuzzer::new(config);
        let (ga, stats) = fuzzer.run();
        
        assert!(!ga.population.is_empty());
        assert!(stats.total_evaluations > 0);
    }
}
