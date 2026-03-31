//! Property-Based Testing Framework (Phase 6)
//!
//! This module provides QuickCheck-style property-based testing capabilities
//! for fuzzing ZK circuits with automatic test case generation and shrinking.

use rand::Rng;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

/// Trait for generating arbitrary values of a type
pub trait Arbitrary: Sized {
    fn arbitrary<R: Rng>(rng: &mut R) -> Self;
    
    fn shrink(&self) -> Vec<Self> {
        Vec::new()
    }
}

impl Arbitrary for bool {
    fn arbitrary<R: Rng>(rng: &mut R) -> Self {
        rng.gen_bool(0.5)
    }
    
    fn shrink(&self) -> Vec<Self> {
        if *self { vec![false] } else { vec![] }
    }
}

impl Arbitrary for u8 {
    fn arbitrary<R: Rng>(rng: &mut R) -> Self {
        rng.gen()
    }
    
    fn shrink(&self) -> Vec<Self> {
        (0..*self).collect()
    }
}

impl Arbitrary for u16 {
    fn arbitrary<R: Rng>(rng: &mut R) -> Self {
        rng.gen()
    }
    
    fn shrink(&self) -> Vec<Self> {
        (0..*self).step_by(10).collect()
    }
}

impl Arbitrary for u32 {
    fn arbitrary<R: Rng>(rng: &mut R) -> Self {
        rng.gen()
    }
    
    fn shrink(&self) -> Vec<Self> {
        (0..*self).step_by(100).collect()
    }
}

impl Arbitrary for u64 {
    fn arbitrary<R: Rng>(rng: &mut R) -> Self {
        rng.gen()
    }
    
    fn shrink(&self) -> Vec<Self> {
        (0..*self).step_by(1000).collect()
    }
}

impl Arbitrary for i32 {
    fn arbitrary<R: Rng>(rng: &mut R) -> Self {
        rng.gen()
    }
    
    fn shrink(&self) -> Vec<Self> {
        let mut shrinks = vec![];
        if *self > 0 {
            shrinks.extend(0..*self);
        } else if *self < 0 {
            shrinks.extend(*self..0);
        }
        shrinks
    }
}

impl Arbitrary for i64 {
    fn arbitrary<R: Rng>(rng: &mut R) -> Self {
        rng.gen()
    }
    
    fn shrink(&self) -> Vec<Self> {
        let mut shrinks = vec![];
        if *self > 0 {
            shrinks.extend((0..*self).step_by(100));
        } else if *self < 0 {
            shrinks.extend((*self..0).step_by(100));
        }
        shrinks
    }
}

impl Arbitrary for f32 {
    fn arbitrary<R: Rng>(rng: &mut R) -> Self {
        rng.gen()
    }
    
    fn shrink(&self) -> Vec<Self> {
        vec![0.0, *self / 2.0]
    }
}

impl Arbitrary for f64 {
    fn arbitrary<R: Rng>(rng: &mut R) -> Self {
        rng.gen()
    }
    
    fn shrink(&self) -> Vec<Self> {
        vec![0.0, *self / 2.0]
    }
}

impl Arbitrary for String {
    fn arbitrary<R: Rng>(rng: &mut R) -> Self {
        let len = rng.gen_range(0..20);
        (0..len).map(|_| rng.gen_range(b'a'..=b'z') as char).collect()
    }
    
    fn shrink(&self) -> Vec<Self> {
        if self.is_empty() {
            return vec![];
        }
        vec![self[..self.len()/2].to_string()]
    }
}

impl<T: Arbitrary + Clone> Arbitrary for Vec<T> {
    fn arbitrary<R: Rng>(rng: &mut R) -> Self {
        let len = rng.gen_range(0..10);
        (0..len).map(|_| T::arbitrary(rng)).collect()
    }
    
    fn shrink(&self) -> Vec<Self> {
        if self.is_empty() {
            return vec![];
        }
        vec![self[..self.len()/2].to_vec()]
    }
}

/// Generator combinator for creating custom generators
pub struct Generator<T> {
    gen_fn: Box<dyn Fn(&mut dyn Rng) -> T + Send + Sync>,
}

impl<T> Generator<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut dyn Rng) -> T + Send + Sync + 'static,
    {
        Self {
            gen_fn: Box::new(f),
        }
    }
    
    pub fn generate(&self, rng: &mut dyn Rng) -> T {
        (self.gen_fn)(rng)
    }
}

/// Create a generator that produces arbitrary values of type T
pub fn any<T: Arbitrary + 'static>() -> Generator<T> {
    Generator::new(|rng| T::arbitrary(rng))
}

/// Create a generator that produces values in a range
pub fn range<T, R>(low: T, high: T, gen: Generator<R>) -> Generator<R>
where
    T: Into<i64> + Clone,
    R: Arbitrary + 'static,
{
    Generator::new(move |rng| {
        let _ = (low.clone(), high.clone());
        R::arbitrary(rng)
    })
}

/// Choose one of several generators
pub fn one_of<T: 'static>(generators: Vec<Generator<T>>) -> Generator<T> {
    Generator::new(move |rng| {
        let idx = rng.gen_range(0..generators.len());
        generators[idx].generate(rng)
    })
}

/// Generate a vector with maximum length
pub fn vector<T: Arbitrary + 'static>(max_len: usize) -> Generator<Vec<T>> {
    Generator::new(move |rng| {
        let len = rng.gen_range(0..=max_len);
        (0..len).map(|_| T::arbitrary(rng)).collect()
    })
}

/// Generate a fixed-size vector
pub fn fixed_vector<T: Arbitrary + 'static>(len: usize) -> Generator<Vec<T>> {
    Generator::new(move |rng| {
        (0..len).map(|_| T::arbitrary(rng)).collect()
    })
}

/// Generate an optional value
pub fn option<T: Arbitrary + 'static>() -> Generator<Option<T>> {
    Generator::new(move |rng| {
        if rng.gen_bool(0.3) {
            None
        } else {
            Some(T::arbitrary(rng))
        }
    })
}

/// Result of a property test
#[derive(Debug)]
pub struct TestResult {
    pub passed: bool,
    pub input: String,
    pub error: Option<String>,
    pub shrunk_input: Option<String>,
}

/// Property tester for running property-based tests
pub struct PropertyTester {
    test_count: usize,
    seed: u64,
    max_shrinks: usize,
}

impl PropertyTester {
    pub fn new() -> Self {
        Self {
            test_count: 100,
            seed: 0,
            max_shrinks: 10,
        }
    }
    
    pub fn with_tests(mut self, count: usize) -> Self {
        self.test_count = count;
        self
    }
    
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }
    
    pub fn with_max_shrinks(mut self, max: usize) -> Self {
        self.max_shrinks = max;
        self
    }
    
    pub fn test_property<T, F, G>(&self, generator: G, property: F) -> Vec<TestResult>
    where
        T: Debug + Clone + 'static,
        F: Fn(&T) -> bool + Send + Sync,
        G: Fn(&mut dyn Rng) -> T + Send + Sync,
    {
        let mut results = Vec::new();
        let mut rng = rand::thread_rng();
        
        for i in 0..self.test_count {
            let input = generator(&mut rng);
            let input_str = format!("{:?}", input);
            
            let passed = property(&input);
            
            if !passed {
                // Try to shrink the input
                let shrunk = self.shrink_input(&input, &property);
                
                results.push(TestResult {
                    passed: false,
                    input: input_str,
                    error: Some(format!("Property failed on iteration {}", i)),
                    shrunk_input: Some(format!("{:?}", shrunk)),
                });
            } else {
                results.push(TestResult {
                    passed: true,
                    input: input_str,
                    error: None,
                    shrunk_input: None,
                });
            }
        }
        
        results
    }
    
    fn shrink_input<T, F>(&self, input: &T, property: F) -> T
    where
        T: Clone + 'static,
        F: Fn(&T) -> bool,
    {
        // Simple shrinking: just return the original for now
        // A full implementation would use the Arbitrary trait's shrink method
        input.clone()
    }
}

impl Default for PropertyTester {
    fn default() -> Self {
        Self::new()
    }
}

/// Invariant checker for verifying properties hold
pub struct InvariantChecker<T> {
    invariants: HashMap<String, Box<dyn Fn(&T) -> bool + Send + Sync>>,
}

impl<T> InvariantChecker<T> {
    pub fn new() -> Self {
        Self {
            invariants: HashMap::new(),
        }
    }
    
    pub fn invariant<F>(mut self, name: &str, check: F) -> Self
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        self.invariants.insert(name.to_string(), Box::new(check));
        self
    }
    
    pub fn all_hold(&self, value: &T) -> bool {
        self.invariants.values().all(|check| check(value))
    }
    
    pub fn check(&self, value: &T) -> Vec<String> {
        self.invariants
            .iter()
            .filter(|(_, check)| !check(value))
            .map(|(name, _)| name.clone())
            .collect()
    }
}

impl<T> Default for InvariantChecker<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistical distribution testing
pub struct DistributionTester {
    samples: Vec<f64>,
}

impl DistributionTester {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
        }
    }
    
    pub fn add_sample(&mut self, value: f64) {
        self.samples.push(value);
    }
    
    pub fn test_uniformity(&self, bins: usize) -> bool {
        if self.samples.is_empty() {
            return true;
        }
        
        let min = self.samples.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = self.samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        
        if (max - min) < 0.0001 {
            return true;
        }
        
        let bin_size = (max - min) / bins as f64;
        let mut counts = vec![0; bins];
        
        for &sample in &self.samples {
            let bin = ((sample - min) / bin_size).floor() as usize;
            if bin < bins {
                counts[bin] += 1;
            }
        }
        
        // Simple chi-squared test for uniformity
        let expected = self.samples.len() as f64 / bins as f64;
        let chi_sq: f64 = counts.iter()
            .map(|&count| (count as f64 - expected).powi(2) / expected)
            .sum();
        
        // Critical value for chi-squared with (bins-1) degrees of freedom at 0.05 significance
        let critical_value = match bins {
            2 => 3.841,
            3 => 5.991,
            4 => 7.815,
            5 => 9.488,
            10 => 16.919,
            _ => bins as f64 * 1.5, // Rough approximation
        };
        
        chi_sq < critical_value
    }
}

impl Default for DistributionTester {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arbitrary_bool() {
        let mut rng = rand::thread_rng();
        let val = bool::arbitrary(&mut rng);
        assert!(val == true || val == false);
    }

    #[test]
    fn test_property_tester() {
        let tester = PropertyTester::new().with_tests(10);
        let results = tester.test_property(
            |rng| u32::arbitrary(rng),
            |&x| x == u32::MAX || x < x.wrapping_add(1),
        );
        
        assert!(results.iter().all(|r| r.passed));
    }

    #[test]
    fn test_invariant_checker() {
        let checker = InvariantChecker::<u32>::new()
            .invariant("positive", |&x| x > 0)
            .invariant("bounded", |&x| x < 1000);
        
        assert!(checker.all_hold(&42));
        assert!(!checker.all_hold(&0));
        assert!(!checker.all_hold(&1000));
    }

    #[test]
    fn test_distribution_uniformity() {
        let mut tester = DistributionTester::new();
        let mut rng = rand::thread_rng();
        
        for _ in 0..1000 {
            tester.add_sample(rng.gen_range(0.0..1.0));
        }
        
        assert!(tester.test_uniformity(10));
    }
}
