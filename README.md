# ZK Circuit Fuzzer

Ultra-modern ZK Circuit Fuzzer for finding bugs in zero-knowledge circuits.

## Overview

This project provides comprehensive fuzzing capabilities for zero-knowledge circuit implementations across multiple proof systems. It includes advanced constraint analysis, genetic algorithm-based input generation, and distributed fuzzing support.

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
```

## Project Structure

```
zk-circuit-fuzzer/
├── src/
│   ├── lib.rs              # Library entry point
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
- Early testers and bug reporters

---

**Version:** 0.1.0  
**Last Updated:** March 31, 2026  
**Phase 5 Status:** Complete
