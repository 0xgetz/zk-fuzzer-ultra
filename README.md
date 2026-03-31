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
