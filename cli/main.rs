//! Command-line interface for ZK Circuit Fuzzer

use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

/// ZK Circuit Fuzzer - Ultra-modern fuzzer for zero-knowledge circuits
#[derive(Parser, Debug)]
#[command(name = "zk-fuzzer")]
#[command(about = "A comprehensive fuzzer for finding bugs in ZK circuits")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Timeout in seconds for operations
    #[arg(long, global = true, default_value = "300")]
    pub timeout: u64,

    /// Number of parallel workers
    #[arg(short, long, global = true, default_value = "4")]
    pub workers: usize,

    /// Output directory for results
    #[arg(short, long, global = true)]
    pub output: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run fuzzing campaign on a circuit
    Fuzz {
        /// Path to the circuit file
        #[arg(short, long)]
        target: PathBuf,

        /// Fuzzing strategy (random, genetic, coverage-guided)
        #[arg(short, long, default_value = "genetic")]
        strategy: String,

        /// Number of test cases to generate
        #[arg(short, long, default_value = "1000")]
        iterations: usize,

        /// Target language (circom, noir)
        #[arg(short, long, default_value = "circom")]
        language: String,
    },

    /// Perform static analysis on a circuit
    Analyze {
        /// Path to the circuit file
        #[arg(short, long)]
        target: PathBuf,

        /// Analysis type (constraint-check, under-constrained, over-constrained, all)
        #[arg(short, long, default_value = "all")]
        analysis_type: String,

        /// Target language (circom, noir)
        #[arg(short, long, default_value = "circom")]
        language: String,
    },

    /// Generate bug report from analysis results
    Report {
        /// Path to the results directory
        #[arg(short, long)]
        results_dir: PathBuf,

        /// Report format (text, json, markdown)
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// Include detailed trace information
        #[arg(long, default_value = "false")]
        include_traces: bool,
    },
}

/// Run the CLI
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("{}", "Verbose mode enabled".cyan());
        println!("  Timeout: {}s", cli.timeout);
        println!("  Workers: {}", cli.workers);
        if let Some(ref output) = cli.output {
            println!("  Output: {}", output.display());
        }
    }

    match cli.command {
        Commands::Fuzz { target, strategy, iterations, language } => {
            println!("{}", "Starting fuzzing campaign...".green().bold());
            println!("  Target: {}", target.display());
            println!("  Strategy: {}", strategy);
            println!("  Iterations: {}", iterations);
            println!("  Language: {}", language);
            println!("  Timeout: {}s", cli.timeout);
            println!("  Workers: {}", cli.workers);
            
            if let Some(ref output_dir) = cli.output {
                println!("  Output directory: {}", output_dir.display());
            }
            
            // Placeholder for actual fuzzing logic
            println!("{}", "Fuzzing engine not yet implemented".yellow());
        }

        Commands::Analyze { target, analysis_type, language } => {
            println!("{}", "Analyzing circuit...".green().bold());
            println!("  Target: {}", target.display());
            println!("  Analysis type: {}", analysis_type);
            println!("  Language: {}", language);
            
            // Placeholder for actual analysis logic
            println!("{}", "Analysis engine not yet implemented".yellow());
        }

        Commands::Report { results_dir, format, include_traces } => {
            println!("{}", "Generating bug report...".green().bold());
            println!("  Results directory: {}", results_dir.display());
            println!("  Format: {}", format);
            println!("  Include traces: {}", include_traces);
            
            if let Some(ref output_dir) = cli.output {
                println!("  Output: {}", output_dir.display());
            }
            
            // Placeholder for actual report generation logic
            println!("{}", "Report generator not yet implemented".yellow());
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_fuzz_command() {
        let cli = Cli::parse_from(vec![
            "zk-fuzzer",
            "fuzz",
            "--target", "circuit.circom",
            "--strategy", "genetic",
            "--iterations", "500",
        ]);
        match cli.command {
            Commands::Fuzz { target, strategy, iterations, language } => {
                assert_eq!(target, PathBuf::from("circuit.circom"));
                assert_eq!(strategy, "genetic");
                assert_eq!(iterations, 500);
                assert_eq!(language, "circom");
            }
            _ => panic!("Expected Fuzz command"),
        }
    }

    #[test]
    fn test_cli_analyze_command() {
        let cli = Cli::parse_from(vec![
            "zk-fuzzer",
            "analyze",
            "--target", "circuit.circom",
            "--analysis-type", "all",
        ]);
        match cli.command {
            Commands::Analyze { target, analysis_type, language } => {
                assert_eq!(target, PathBuf::from("circuit.circom"));
                assert_eq!(analysis_type, "all");
                assert_eq!(language, "circom");
            }
            _ => panic!("Expected Analyze command"),
        }
    }

    #[test]
    fn test_cli_report_command() {
        let cli = Cli::parse_from(vec![
            "zk-fuzzer",
            "report",
            "--results-dir", "results/",
            "--format", "json",
        ]);
        match cli.command {
            Commands::Report { results_dir, format, include_traces } => {
                assert_eq!(results_dir, PathBuf::from("results/"));
                assert_eq!(format, "json");
                assert!(!include_traces);
            }
            _ => panic!("Expected Report command"),
        }
    }

    #[test]
    fn test_global_flags() {
        let cli = Cli::parse_from(vec![
            "zk-fuzzer",
            "--timeout", "600",
            "--workers", "8",
            "--verbose",
            "fuzz",
            "--target", "circuit.circom",
        ]);
        assert_eq!(cli.timeout, 600);
        assert_eq!(cli.workers, 8);
        assert!(cli.verbose);
    }
}
