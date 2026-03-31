//! ZK Circuit Fuzzer - Ultra-modern fuzzer for zero-knowledge circuits
//!
//! This library provides comprehensive fuzzing capabilities for ZK circuits,
//! including constraint analysis, genetic algorithms, symbolic execution,
//! and LLM-guided mutation strategies.
//!
//! # Overview
//!
//! The fuzzer is organized into several key modules:
//!
//! - **core**: Fundamental circuit processing (parsing, constraint extraction, TCCT, emulation)
//! - **fuzzing**: Fuzzing strategies (genetic algorithms, mutation, input generation, coverage)
//! - **analysis**: Advanced analysis techniques (symbolic execution, LLM analysis, static analysis)
//! - **targets**: Circuit framework support (Circom, Noir)
//! - **cli**: Command-line interface
//!
//! # Example
//!
//! ```rust,ignore
//! use zk_circuit_fuzzer::core::parser::CircomParser;
//! use zk_circuit_fuzzer::fuzzing::genetic::GeneticFuzzer;
//!
//! let parser = CircomParser::new();
//! let circuit = parser.parse_file("circuit.circom")?;
//!
//! let mut fuzzer = GeneticFuzzer::new(circuit);
//! let bugs = fuzzer.run(1000)?;
//! ```

// Module declarations
pub mod core;
pub mod fuzzing;
pub mod analysis;
pub mod targets;
pub mod cli;

// Error types
/// Main error type for the ZK Circuit Fuzzer
#[derive(Debug)]
pub enum FuzzerError {
    /// Parser error
    Parser(String),
    /// Constraint extraction error
    ConstraintError(String),
    /// TCCT engine error
    TCCTError(String),
    /// Emulation error
    EmulationError(String),
    /// Fuzzing error
    FuzzingError(String),
    /// Analysis error
    AnalysisError(String),
    /// I/O error
    IoError(std::io::Error),
    /// Generic error
    Generic(String),
}

impl std::fmt::Display for FuzzerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FuzzerError::Parser(msg) => write!(f, "Parser error: {}", msg),
            FuzzerError::ConstraintError(msg) => write!(f, "Constraint error: {}", msg),
            FuzzerError::TCCTError(msg) => write!(f, "TCCT error: {}", msg),
            FuzzerError::EmulationError(msg) => write!(f, "Emulation error: {}", msg),
            FuzzerError::FuzzingError(msg) => write!(f, "Fuzzing error: {}", msg),
            FuzzerError::AnalysisError(msg) => write!(f, "Analysis error: {}", msg),
            FuzzerError::IoError(e) => write!(f, "I/O error: {}", e),
            FuzzerError::Generic(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for FuzzerError {}

impl From<std::io::Error> for FuzzerError {
    fn from(err: std::io::Error) -> Self {
        FuzzerError::IoError(err)
    }
}

/// Result type alias for fuzzer operations
pub type FuzzerResult<T> = Result<T, FuzzerError>;

// Re-export commonly used types for convenience
pub use analysis::llm::LLMAnalyzer;
pub use analysis::static_analysis::{StaticAnalyzer, StaticAnalysisResult};
pub use analysis::symbolic::SymbolicAnalyzer;
pub use core::constraints::{ConstraintExtractor, ConstraintSystem};
pub use core::emulator::{WitnessAssignment, WitnessEmulator};
pub use core::parser::{Circuit, CircomParser, ParserError};
pub use core::tcct::{ExecutionTrace, TCCTEngine, TCCTResult};
pub use fuzzing::coverage::CoverageTracker;
pub use fuzzing::genetic::{BugReport, GeneticFuzzer, Individual};
pub use fuzzing::input_gen::{GeneratedInput, InputGenerator};
pub use fuzzing::mutator::Mutator;
pub use targets::circom::CircomFuzzerConfig;
pub use targets::noir::NoirFuzzerConfig;
