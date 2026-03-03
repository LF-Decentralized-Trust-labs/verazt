//! SmartBench - Benchmark smarthunt against the SmartBugs-curated dataset.
//!
//! This crate provides tools for:
//! - Parsing ground-truth annotations from Solidity files
//! - Running analysis tools (via local binary or Docker image)
//! - Comparing detections against ground truth
//! - Generating accuracy reports

pub mod annotation;
pub mod matcher;
pub mod report;
pub mod runner;

pub use annotation::{AnnotatedBug, parse_annotations, scan_dataset};
pub use matcher::{MatchResult, MatchedBug, match_file};
pub use report::{BenchmarkReport, CategoryReport, FileReport, generate_report, print_report};
pub use runner::{
    BinaryRunner, DetectedBug, DockerRunner, ToolConfig, ToolResult, ToolRunner, create_runner,
    run_batch,
};
