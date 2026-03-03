//! Tool invocation runners.
//!
//! Supports two modes:
//! - `BinaryRunner`: Run a local binary
//! - `DockerRunner`: Run inside a Docker container

use bugs::bug::BugCategory;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// How to invoke the analysis tool.
#[derive(Debug, Clone)]
pub enum ToolConfig {
    /// Run a local binary.
    Binary { path: PathBuf },
    /// Run inside a Docker container.
    Docker {
        image: String,
        timeout_secs: Option<u64>,
        memory_limit: Option<String>,
        cpus: Option<f64>,
    },
}

/// Result of running the tool on a single file.
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// Path to the analyzed file.
    pub file_path: PathBuf,
    /// Detected bugs.
    pub findings: Vec<DetectedBug>,
    /// Whether the tool execution succeeded.
    pub success: bool,
    /// Standard error output.
    pub stderr: String,
}

/// A bug detected by the analysis tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedBug {
    /// Bug name/title.
    pub name: String,
    /// Bug category.
    pub category: BugCategory,
    /// Start line of the bug.
    pub start_line: usize,
    /// Severity level.
    pub severity: String,
}

/// Trait for tool invocation backends.
pub trait ToolRunner {
    /// Run the tool on a single .sol file.
    fn run(&self, sol_file: &Path) -> Result<ToolResult, String>;
}

/// Runs a local binary.
pub struct BinaryRunner {
    pub binary_path: PathBuf,
}

impl BinaryRunner {
    pub fn new(binary_path: PathBuf) -> Self {
        Self { binary_path }
    }

    /// Parse JSON output from smarthunt into DetectedBugs.
    fn parse_json_output(stdout: &str, file_path: &Path) -> Result<Vec<DetectedBug>, String> {
        let json: serde_json::Value =
            serde_json::from_str(stdout).map_err(|e| format!("JSON parse error: {}", e))?;

        let mut bugs = Vec::new();
        if let Some(findings) = json.get("findings").and_then(|f| f.as_array()) {
            for finding in findings {
                let name = finding
                    .get("title")
                    .and_then(|t| t.as_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let category_str = finding
                    .get("category")
                    .and_then(|c| c.as_str())
                    .unwrap_or("Other");

                let category = category_from_json(category_str);

                let start_line = finding
                    .get("location")
                    .and_then(|l| l.get("start_line"))
                    .and_then(|l| l.as_u64())
                    .unwrap_or(0) as usize;

                let severity = finding
                    .get("severity")
                    .and_then(|s| s.as_str())
                    .unwrap_or("Unknown")
                    .to_string();

                bugs.push(DetectedBug { name, category, start_line, severity });
            }
        }

        let _ = file_path; // used for context in error messages
        Ok(bugs)
    }
}

impl ToolRunner for BinaryRunner {
    fn run(&self, sol_file: &Path) -> Result<ToolResult, String> {
        let sol_file = sol_file
            .canonicalize()
            .map_err(|e| format!("Failed to resolve path {}: {}", sol_file.display(), e))?;

        let output = Command::new(&self.binary_path)
            .args(["analyze", "--format", "json"])
            .arg(&sol_file)
            .output()
            .map_err(|e| format!("Failed to run binary: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() && stdout.is_empty() {
            return Ok(ToolResult {
                file_path: sol_file,
                findings: vec![],
                success: false,
                stderr,
            });
        }

        let findings = if stdout.trim().is_empty() {
            vec![]
        } else {
            BinaryRunner::parse_json_output(&stdout, &sol_file).unwrap_or_else(|e| {
                eprintln!("Warning: Failed to parse output for {}: {}", sol_file.display(), e);
                vec![]
            })
        };

        Ok(ToolResult { file_path: sol_file, findings, success: true, stderr })
    }
}

/// Runs the tool inside a Docker container.
pub struct DockerRunner {
    pub image: String,
    pub timeout_secs: Option<u64>,
    pub memory_limit: Option<String>,
    pub cpus: Option<f64>,
}

impl DockerRunner {
    pub fn new(
        image: String,
        timeout_secs: Option<u64>,
        memory_limit: Option<String>,
        cpus: Option<f64>,
    ) -> Self {
        Self { image, timeout_secs, memory_limit, cpus }
    }
}

impl ToolRunner for DockerRunner {
    fn run(&self, sol_file: &Path) -> Result<ToolResult, String> {
        let sol_file = sol_file
            .canonicalize()
            .map_err(|e| format!("Failed to resolve path {}: {}", sol_file.display(), e))?;

        // Create a temp dir for output
        let output_dir =
            TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;
        let output_path = output_dir.path();

        let mut cmd = Command::new("docker");
        cmd.args(["run", "--rm"]);

        // Mount the sol file
        cmd.arg("-v")
            .arg(format!("{}:/contract.sol:ro", sol_file.display()));

        // Mount the output directory
        cmd.arg("-v")
            .arg(format!("{}:/output", output_path.display()));

        // Resource limits
        if let Some(mem) = &self.memory_limit {
            cmd.arg("--memory").arg(mem);
        }
        if let Some(cpus) = self.cpus {
            cmd.arg("--cpus").arg(cpus.to_string());
        }
        if let Some(timeout) = self.timeout_secs {
            cmd.arg("--stop-timeout").arg(timeout.to_string());
        }

        // Image and command
        cmd.arg(&self.image);
        cmd.args([
            "analyze",
            "--format",
            "json",
            "--output",
            "/output/result.json",
            "/contract.sol",
        ]);

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to run docker: {}", e))?;

        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Ok(ToolResult {
                file_path: sol_file,
                findings: vec![],
                success: false,
                stderr,
            });
        }

        // Read the result.json from the temp directory
        let result_file = output_path.join("result.json");
        let findings = if result_file.exists() {
            let content = std::fs::read_to_string(&result_file)
                .map_err(|e| format!("Failed to read result: {}", e))?;
            BinaryRunner::parse_json_output(&content, &sol_file).unwrap_or_else(|e| {
                eprintln!(
                    "Warning: Failed to parse Docker output for {}: {}",
                    sol_file.display(),
                    e
                );
                vec![]
            })
        } else {
            vec![]
        };

        Ok(ToolResult { file_path: sol_file, findings, success: true, stderr })
    }
}

/// Create the appropriate runner from a ToolConfig.
pub fn create_runner(config: &ToolConfig) -> Box<dyn ToolRunner> {
    match config {
        ToolConfig::Binary { path } => Box::new(BinaryRunner::new(path.clone())),
        ToolConfig::Docker { image, timeout_secs, memory_limit, cpus } => {
            Box::new(DockerRunner::new(image.clone(), *timeout_secs, memory_limit.clone(), *cpus))
        }
    }
}

/// Run the tool on all .sol files in a directory.
pub fn run_batch(runner: &dyn ToolRunner, dataset_dir: &Path) -> Vec<ToolResult> {
    use crate::annotation::collect_sol_files;

    let files = collect_sol_files(dataset_dir);
    let mut results = Vec::new();

    for file in &files {
        match runner.run(file) {
            Ok(result) => results.push(result),
            Err(e) => {
                results.push(ToolResult {
                    file_path: file.clone(),
                    findings: vec![],
                    success: false,
                    stderr: e,
                });
            }
        }
    }

    results
}

/// Convert a category string from JSON output to BugCategory.
fn category_from_json(category_str: &str) -> BugCategory {
    match category_str {
        "Reentrancy" => BugCategory::Reentrancy,
        "Arithmetic" => BugCategory::Arithmetic,
        "Access Control" => BugCategory::AccessControl,
        "Unchecked Low Level Calls" => BugCategory::UncheckedLowLevelCalls,
        "Denial of Service" => BugCategory::DenialOfService,
        "Bad Randomness" => BugCategory::BadRandomness,
        "Front Running" => BugCategory::FrontRunning,
        "Time Manipulation" => BugCategory::TimeManipulation,
        "Short Addresses" => BugCategory::ShortAddresses,
        "Code Quality" => BugCategory::CodeQuality,
        _ => BugCategory::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_from_json() {
        assert_eq!(category_from_json("Reentrancy"), BugCategory::Reentrancy);
        assert_eq!(category_from_json("Access Control"), BugCategory::AccessControl);
        assert_eq!(
            category_from_json("Unchecked Low Level Calls"),
            BugCategory::UncheckedLowLevelCalls
        );
        assert_eq!(category_from_json("Time Manipulation"), BugCategory::TimeManipulation);
        assert_eq!(category_from_json("Code Quality"), BugCategory::CodeQuality);
        assert_eq!(category_from_json("Unknown"), BugCategory::Other);
    }

    #[test]
    fn test_parse_json_output() {
        let json = r#"{
            "version": "0.0.1",
            "timestamp": "2024-01-01T00:00:00Z",
            "duration_ms": 100,
            "files_analyzed": ["test.sol"],
            "summary": {"total": 1, "critical": 0, "high": 1, "medium": 0, "low": 0, "info": 0},
            "findings": [
                {
                    "id": "vulnerability",
                    "title": "Reentrancy (DFA)",
                    "description": "Potential reentrancy",
                    "severity": "Critical",
                    "category": "Reentrancy",
                    "location": {"file": null, "start_line": 19, "end_line": 19, "start_column": 1, "end_column": 50},
                    "swc_id": "SWC-107",
                    "cwe_id": "CWE-841",
                    "confidence": "high"
                }
            ]
        }"#;

        let bugs = BinaryRunner::parse_json_output(json, Path::new("test.sol")).unwrap();
        assert_eq!(bugs.len(), 1);
        assert_eq!(bugs[0].name, "Reentrancy (DFA)");
        assert_eq!(bugs[0].category, BugCategory::Reentrancy);
        assert_eq!(bugs[0].start_line, 19);
        assert_eq!(bugs[0].severity, "Critical");
    }

    #[test]
    fn test_parse_json_output_empty() {
        let json = r#"{
            "version": "0.0.1",
            "timestamp": "2024-01-01T00:00:00Z",
            "duration_ms": 0,
            "files_analyzed": [],
            "summary": {"total": 0, "critical": 0, "high": 0, "medium": 0, "low": 0, "info": 0},
            "findings": []
        }"#;

        let bugs = BinaryRunner::parse_json_output(json, Path::new("test.sol")).unwrap();
        assert!(bugs.is_empty());
    }

    #[test]
    fn test_create_runner_binary() {
        let config = ToolConfig::Binary { path: PathBuf::from("/usr/bin/smarthunt") };
        let _runner = create_runner(&config);
    }

    #[test]
    fn test_create_runner_docker() {
        let config = ToolConfig::Docker {
            image: "smarthunt:latest".to_string(),
            timeout_secs: Some(300),
            memory_limit: Some("4g".to_string()),
            cpus: Some(2.0),
        };
        let _runner = create_runner(&config);
    }
}
