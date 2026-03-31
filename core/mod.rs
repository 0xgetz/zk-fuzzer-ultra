//! Core module for ZK Circuit Fuzzer
//!
//! This module contains the fundamental components for parsing, constraint extraction,
//! trace-constraint consistency testing, and witness emulation.

pub mod parser;
pub mod constraints;
pub mod tcct;
pub mod emulator;

// Re-export main types for convenience
pub use parser::CircomParser;
pub use constraints::ConstraintExtractor;
pub use tcct::TCCTEngine;
pub use emulator::WitnessEmulator;
