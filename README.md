# ZK Circuit Fuzzer

Ultra-modern ZK Circuit Fuzzer for finding bugs in zero-knowledge circuits.

## Overview

This project provides comprehensive fuzzing capabilities for zero-knowledge circuit implementations across multiple proof systems. It includes advanced constraint analysis, genetic algorithm-based input generation, property-based testing, and distributed fuzzing support.

## Supported Frameworks

- **Circom** - Full support for .circom files
- **Noir** - Full support for .nr files
- **Halo2** - Rust-based circuit fuzzing (Phase 5)
- **Gnark** - Go-based circuit fuzzing (Phase 5)

## Features

### Core Fuzzing Engine
- Constraint-based test generation
- Genetic algorithm optimization
- Coverage-guided fuzzing
- Distributed execution support
- Real-time monitoring and metrics

### Analysis Tools
- Static constraint analysis
- Symbolic execution via Z3 SMT solver
- LLM-assisted bug classification
- Automatic crash triaging

### Advanced Constraint Solving (Phase 6)
- Z3 SMT solver integration
- Support for integer, boolean, real, and bitvector constraints
- Linear and nonlinear arithmetic
- Constraint optimization (maximize/minimize)
- Model extraction and unsat core analysis
- Backtracking and incremental solving

### Property-Based Testing (Phase 6)
- QuickCheck-style generators
- Arbitrary trait for custom types
- Property combinators
- Automatic shrinking for minimal counterexamples
- Invariant checking framework
- Statistical distribution testing

### Web Dashboard (Phase 5)
- Real-time campaign monitoring
- Bug discovery tracking
- Performance metrics visualization
- Interactive circuit analysis

## Installation

```bash
git clone https://github.com/0xgetz/zk-circuit-fuzzer.git
cd zk-circuit-fuzzer
cargo build --release
```

### Optional Features

```bash
# Include Halo2 support
cargo build --release --features halo2

# Include all Phase 5 features
cargo build --release --all-features

# Include Phase 6 features (Z3, QuickCheck, Proptest)
cargo build --release --features phase6
```

## Usage

### Basic Fuzzing

```bash
# Fuzz a Circom circuit
./target/release/zk-fuzzer fuzz --target circom --input circuit.circom

# Fuzz a Noir circuit
./target/release/zk-fuzzer fuzz --target noir --input circuit.nr

# Fuzz a Halo2 circuit (Rust)
./target/release/zk-fuzzer fuzz --target halo2 --input circuit.rs

# Fuzz a Gnark circuit (Go)
./target/release/zk-fuzzer fuzz --target gnark --input circuit.go
```

### Advanced Options

```bash
# Run with distributed fuzzing
./target/release/zk-fuzzer fuzz --target circom --input circuit.circom --distributed

# Enable web dashboard
./target/release/zk-fuzzer dashboard --port 8080

# Run analysis only
./target/release/zk-fuzzer analyze --target circom --input circuit.circom

# Run with constraint-guided fuzzing (Phase 6)
./target/release/zk-fuzzer fuzz --target circom --input circuit.circom --constraint-guided

# Run with property-based testing (Phase 6)
./target/release/zk-fuzzer fuzz --target circom --input circuit.circom --property-testing
```

## Project Structure

```
zk-circuit-fuzzer/
├── src/
│   ├── main.rs             # Application entry point
│   ├── lib.rs              # Library entry point
│   ├── genetic.rs          # Genetic algorithm core
│   ├── bug_reporter.rs     # Bug reporting system
│   ├── constraint_solver.rs # Z3 SMT constraint solver (Phase 6)
│   ├── property_testing.rs # Property-based testing framework (Phase 6)
│   ├── integration.rs      # Integration module (Phase 6)
│   ├── targets/            # Framework-specific implementations
│   │   ├── mod.rs
│   │   ├── circom.rs
│   │   ├── gnark.rs        # Phase 5
│   │   ├── halo2.rs        # Phase 5
│   │   └── noir.rs
│   ├── dashboard/          # Web UI (Phase 5)
│   │   ├── mod.rs
│   │   ├── server.rs
│   │   └── routes.rs
│   ├── analysis/           # Bug analysis and classification
│   ├── fuzzing/            # Core fuzzing engine
│   └── core/               # Shared utilities
├── targets/                # Target framework modules
│   ├── mod.rs
│   ├── circom/
│   ├── gnark.rs            # Phase 5
│   ├── halo2.rs            # Phase 5
│   └── noir/
├── static/                 # Web dashboard static files
│   ├── index.html
│   ├── style.css
│   └── app.js
├── cli/
│   └── main.rs             # CLI entry point
└── Cargo.toml
```

## Phase 6 Completion Status

### Completed Features

- [x] **Advanced Constraint Solver with Z3 SMT**
  - Full Z3 integration via z3 crate
  - Support for multiple constraint types:
    - Integer constraints (equality, inequality, comparisons)
    - Boolean constraints (AND, OR, NOT)
    - Real number constraints
    - Bitvector operations (extraction, concatenation)
  - Arithmetic operations (add, sub, mul, div, mod)
  - Optimization objectives (maximize, minimize)
  - Model extraction and solution analysis
  - Unsat core extraction for debugging
  - Incremental solving with push/pop
  - 30-second timeout per solve

- [x] **Property-Based Testing Framework**
  - QuickCheck-style arbitrary value generation
  - Arbitrary trait implementation for common types (bool, u8-u64, i32, i64, f32, f64, String, Vec<T>)
  - Generator combinators:
    - `any<T>()` - generate arbitrary values
    - `range(low, high, gen)` - bounded generation
    - `one_of(generators)` - choice combinators
    - `vector(max_len)` / `fixed_vector(len)` - collection generators
    - `option<T>()` / `result<T, E>()` - optional/result types
  - Property testing with automatic shrinking
  - Invariant checking framework
  - Statistical distribution testing (chi-squared test for uniformity)
  - Configurable test count and shrink limits
  - Deterministic seeding for reproducibility

- [x] **Integration Module**
  - IntegratedFuzzer combining GA + constraints + property testing
  - Constraint-guided mutation operators
  - Constraint-guided individual generation
  - Invariant-preserving evolution
  - CircuitPropertyTester for input validation
  - FuzzingStatistics for comprehensive metrics
  - Solution-to-genome conversion

- [x] **Dependencies Updated**
  - Added z3 = "0.12" for SMT solving
  - Added quickcheck = "1.0" for property-based testing
  - Added proptest = "1.4" for additional property testing
  - Updated Cargo.toml with new feature flags

### New Modules

| Module | Description |
|--------|-------------|
| `constraint_solver.rs` | Z3-based constraint solving and optimization |
| `property_testing.rs` | QuickCheck-style generators and invariant checking |
| `integration.rs` | Integration of all fuzzing components |

### API Reference (Phase 6)

#### Constraint Solver

```rust
use zk_circuit_fuzzer::constraint_solver::{ConstraintSolver, ConstraintExpr, ConstraintType};

let mut solver = ConstraintSolver::new();
solver.declare_int_var("x");
solver.declare_int_var("y");

solver.add_constraint(ConstraintExpr::BinOp {
    op: ConstraintType::Equality,
    left: Box::new(ConstraintExpr::Add(vec![
        ConstraintExpr::Var("x".to_string()),
        ConstraintExpr::Var("y".to_string()),
    ])),
    right: Box::new(ConstraintExpr::IntConst(42)),
});

let result = solver.solve();
if result.sat {
    for (name, value) in &result.model {
        println!("{} = {}", name, value);
    }
}
```

#### Property Testing

```rust
use zk_circuit_fuzzer::property_testing::{PropertyTester, InvariantChecker, generators, Arbitrary};

let tester = PropertyTester::new()
    .with_tests(100)
    .with_seed(42);

let results = tester.test_property(
    |rng| u32::arbitrary(rng),
    |&x| x == u32::MAX || x < x.wrapping_add(1),
);

// Invariant checking
let checker = InvariantChecker::<u32>::new()
    .invariant("positive", |&x| x > 0)
    .invariant("bounded", |&x| x < 1000);

assert!(checker.all_hold(&42));
```

#### Integrated Fuzzing

```rust
use zk_circuit_fuzzer::integration::{IntegratedConfig, IntegratedFuzzer};

let config = IntegratedConfig {
    ga_config: Config::default(),
    use_constraints: true,
    use_property_testing: true,
    max_constraint_attempts: 10,
    property_test_count: 50,
    invariant_strength: 0.8,
};

let mut fuzzer = IntegratedFuzzer::new(config);
let (ga, stats) = fuzzer.run();
```

## Phase 5 Completion Status

### Completed Features

- [x] **Halo2 Framework Support**
  - Halo2Circuit struct with full ZKTarget trait implementation
  - Custom gate and constraint definitions
  - Support for BN254, BLS12-381 curves
  - Plonk and Groth16 backend support

- [x] **Gnark Framework Support**
  - GnarkCircuit struct with full ZKTarget trait implementation
  - Witness configuration and constraint management
  - Support for multiple curves (BN254, BLS12-381, etc.)
  - Backend support: Groth16, Plonk, Marlin

- [x] **Web Dashboard**
  - Axum-based HTTP server
  - RESTful API endpoints for campaigns and bugs
  - Real-time metrics and monitoring
  - Responsive HTML/CSS/JS frontend
  - CORS support for cross-origin requests

- [x] **Dependencies Updated**
  - Added axum, tokio, tower-http for web server
  - Added chrono for timestamp handling
  - Added halo2_proofs (optional feature)
  - Updated Cargo.toml with feature flags

### API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/health` | GET | Health check |
| `/api/metrics` | GET | Dashboard metrics |
| `/api/campaigns` | GET | List all campaigns |
| `/api/campaigns/:id` | GET | Get campaign details |
| `/api/bugs` | GET | List discovered bugs |
| `/api/bugs/:id` | GET | Get bug details |

## Development

### Running Tests

```bash
cargo test
```

### Building Documentation

```bash
cargo doc --open
```

### Running Examples

```bash
# Run the integrated demo
cargo run --release
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Acknowledgments

- The ZK community for excellent proof system implementations
- Contributors to the Halo2 and Gnark projects
- Z3 developers for the SMT solver
- QuickCheck and Proptest developers for property-based testing inspiration
- Early testers and bug reporters

---

**Version:** 0.1.0  
**Last Updated:** March 31, 2026  
**Phase 5 Status:** Complete  
**Phase 6 Status:** Complete

## Phase 7: ML-Based Fitness Functions

### Overview

Phase 7 introduces machine learning-based fitness functions that use neural networks to predict the fitness of test inputs, enabling more intelligent and efficient fuzzing. The ML models learn from historical fuzzing data to prioritize promising test cases and guide the genetic algorithm toward more productive regions of the search space.

### Key Features

- **Neural Network Fitness Predictor**: Multi-layer perceptron (MLP) that learns to predict fitness scores
- **Comprehensive Feature Extraction**: 48 features combining circuit structure and input characteristics
- **Online Learning**: Continuous model improvement during fuzzing campaigns
- **Hybrid Fitness**: Weighted combination of ML predictions and traditional heuristics
- **Training Pipeline**: Complete ML pipeline with data collection, preprocessing, training, and evaluation

### New Modules

| Module | Description |
|--------|-------------|
| `feature_extraction.rs` | Circuit and input feature extraction for ML models |
| `ml_fitness.rs` | Neural network models for fitness prediction |
| `ml_training.rs` | Training pipeline with evaluation metrics |

### Feature Extraction

The framework extracts 48 features from circuits and test inputs:

**Circuit Features (23)**:
- Structural: num_constraints, num_variables, num_gates, constraint_density
- Constraint types: arithmetic, boolean, range, equality, custom
- Graph metrics: depth, fanout, cyclomatic complexity
- Signal properties: bitwidth, interdependencies
- Temporal: execution path length, coverage potential, loop depth
- Historical: bug discovery rate, constraint violations, improvement rate

**Input Features (15)**:
- Statistics: mean, std_dev, min, max, entropy
- Distribution: skewness, kurtosis, zero_ratio, one_ratio
- Patterns: repetition_count, pattern_complexity, bit_transition_rate
- Constraint-related: satisfaction_score, boundary_proximity, diversity_score

**Interaction Features (10)**:
- Cross-products between key circuit and input features

### ML Model Architecture

The default neural network architecture:
```
Input (48) -> Dense(64, ReLU) -> Dense(32, ReLU) -> Dense(16, ReLU) -> Dense(1, Sigmoid)
```

The model outputs a fitness prediction in [0, 1] range.

### Usage

#### Basic ML Fitness Prediction

```rust
use zk_circuit_fuzzer::ml_fitness::{MLFitnessPredictor, MLFitnessConfig};
use zk_circuit_fuzzer::feature_extraction::{FeatureExtractor, CircuitFeatures, CombinedFeatures};

// Create predictor
let mut predictor = MLFitnessPredictor::new(CombinedFeatures::TOTAL_FEATURES, None);

// Extract features
let mut extractor = FeatureExtractor::new();
extractor.set_circuit_features(circuit_features);
let combined = extractor.extract(&genome);
let features = combined.to_array();

// Predict fitness
let predicted_fitness = predictor.predict(&features);

// Record actual fitness for online learning
predictor.record_sample(features, actual_fitness);
```

#### Training Pipeline

```rust
use zk_circuit_fuzzer::ml_training::{TrainingPipeline, TrainingConfig};

let mut pipeline = TrainingPipeline::new(Some(TrainingConfig {
    epochs: 100,
    learning_rate: 0.001,
    batch_size: 32,
    validation_split: 0.2,
    early_stopping_patience: 10,
    ..Default::default()
}));

// Add training samples
pipeline.add_sample(&genome, circuit_features, fitness, metadata);

// Build dataset and train
let dataset = pipeline.build_dataset();
let result = pipeline.train(&dataset);

// Evaluate
let (train, val) = dataset.split(0.2, 42);
let metrics = pipeline.evaluate(result.model, &val);
```

#### Hybrid Fitness Function

```rust
use zk_circuit_fuzzer::ml_fitness::HybridFitnessFunction;

// Create hybrid with 70% ML weight
let mut hybrid = HybridFitnessFunction::new(
    CombinedFeatures::TOTAL_FEATURES,
    0.7, // ML weight
    None,
);

// Evaluate
let hybrid_fitness = hybrid.evaluate(&features, heuristic_fitness);
```

### Training Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| epochs | 100 | Number of training epochs |
| learning_rate | 0.001 | Optimizer learning rate |
| batch_size | 32 | Mini-batch size |
| validation_split | 0.2 | Validation data ratio |
| early_stopping_patience | 10 | Patience for early stopping |
| min_improvement | 1e-4 | Minimum loss improvement |
| replay_buffer_size | 10000 | Online learning buffer size |
| update_frequency | 50 | Online update frequency |

### Evaluation Metrics

The training pipeline reports:
- **MSE** (Mean Squared Error)
- **MAE** (Mean Absolute Error)
- **RMSE** (Root Mean Squared Error)
- **R-squared** (Coefficient of determination)
- **Max Error** (Worst prediction error)
- **MAPE** (Mean Absolute Percentage Error)

### Integration with Genetic Algorithm

The ML fitness predictor can be integrated with the genetic algorithm for intelligent fitness evaluation:

```rust
use zk_circuit_fuzzer::genetic::{GeneticAlgorithm, Config, Individual};
use zk_circuit_fuzzer::ml_fitness::MLFitnessPredictor;

let mut ga = GeneticAlgorithm::new(Config::default());
let mut predictor = MLFitnessPredictor::new(CombinedFeatures::TOTAL_FEATURES, None);

for generation in 0..max_generations {
    for individual in ga.population_mut().individuals.iter_mut() {
        // Extract features
        let features = extract_features(&individual.genome, circuit_info);
        
        // Predict fitness
        let predicted = predictor.predict(&features);
        
        // Evaluate actual fitness
        let actual = evaluate_fitness(&individual);
        
        // Use hybrid or predicted fitness
        individual.fitness = predicted; // or hybrid combination
        
        // Record for online learning
        predictor.record_sample(features, actual);
    }
    
    ga.evolve();
}
```

### Dependencies

Phase 7 adds the following ML dependencies:

```toml
[dependencies]
ndarray = "0.15"
ndarray-rand = "0.14"
linfa = "0.7"
linfa-nn = "0.7"
linfa-clustering = "0.7"
linfa-trees = "0.7"
linfa-linear = "0.7"
linfa-preprocessing = "0.7"
```

### Build with Phase 7

```bash
# Build with ML features
cargo build --release --features phase7

# Build with all features
cargo build --release --all-features
```

### Performance Considerations

- **Training Time**: Neural networks require initial training on historical data (typically minutes)
- **Inference Speed**: Prediction is fast (~microseconds per sample)
- **Memory**: Replay buffer stores up to 10,000 samples by default
- **Online Learning**: Periodic updates add minimal overhead to fuzzing

### Best Practices

1. **Warm-up Phase**: Collect initial data with heuristic fitness before enabling ML
2. **Feature Normalization**: Always normalize features for better training
3. **Early Stopping**: Use validation loss to prevent overfitting
4. **Hybrid Approach**: Start with high heuristic weight, gradually increase ML weight
5. **Periodic Retraining**: Retrain on accumulated data for long-running campaigns

### Future Enhancements

- Support for more advanced architectures (CNNs, Transformers)
- Transfer learning across different circuit types
- Multi-objective fitness prediction
- Uncertainty estimation for exploration-exploitation balance
- Distributed training for large-scale campaigns

---

**Phase 7 Status:** Complete
**Last Updated:** March 31, 2026
