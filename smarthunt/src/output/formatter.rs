//! Output formatter trait.

use bugs::bug::Bug;
use crate::engine::context::AnalysisContext;
use std::time::Duration;

/// Analysis report containing all findings.
pub struct AnalysisReport {
    /// All bugs found
    pub bugs: Vec<Bug>,
    
    /// Source files analyzed
    pub files_analyzed: Vec<String>,
    
    /// Analysis duration
    pub duration: Duration,
    
    /// SmartHunt version
    pub version: String,
    
    /// Analysis timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Statistics
    pub stats: AnalysisStats,
}

/// Analysis statistics.
#[derive(Debug, Default)]
pub struct AnalysisStats {
    /// Total number of contracts analyzed
    pub contracts: usize,
    
    /// Total number of functions analyzed
    pub functions: usize,
    
    /// Number of detectors run
    pub detectors_run: usize,
    
    /// Bugs by severity
    pub bugs_by_severity: BugsBySeverity,
}

/// Bug counts by severity.
#[derive(Debug, Default)]
pub struct BugsBySeverity {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub info: usize,
}

impl AnalysisReport {
    /// Create a new analysis report.
    pub fn new(
        bugs: Vec<Bug>,
        files_analyzed: Vec<String>,
        duration: Duration,
    ) -> Self {
        let mut stats = AnalysisStats::default();
        
        // Count bugs by severity
        for bug in &bugs {
            match bug.risk_level {
                bugs::bug::RiskLevel::Critical => stats.bugs_by_severity.critical += 1,
                bugs::bug::RiskLevel::High => stats.bugs_by_severity.high += 1,
                bugs::bug::RiskLevel::Medium => stats.bugs_by_severity.medium += 1,
                bugs::bug::RiskLevel::Low => stats.bugs_by_severity.low += 1,
                bugs::bug::RiskLevel::No => stats.bugs_by_severity.info += 1,
            }
        }
        
        Self {
            bugs,
            files_analyzed,
            duration,
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: chrono::Utc::now(),
            stats,
        }
    }

    /// Get total bug count.
    pub fn total_bugs(&self) -> usize {
        self.bugs.len()
    }

    /// Check if there are any bugs.
    pub fn has_bugs(&self) -> bool {
        !self.bugs.is_empty()
    }

    /// Check if there are high severity bugs.
    pub fn has_high_severity(&self) -> bool {
        self.stats.bugs_by_severity.critical > 0 || self.stats.bugs_by_severity.high > 0
    }
}

/// Trait for output formatters.
pub trait OutputFormatter {
    /// Format the analysis report.
    fn format(&self, report: &AnalysisReport) -> String;
    
    /// Get the file extension for this format.
    fn extension(&self) -> &'static str;
    
    /// Get the content type for this format.
    fn content_type(&self) -> &'static str;
}

/// Format a location for display.
pub fn format_location(bug: &Bug) -> String {
    format!(
        "<unknown>:{}:{}",
        bug.loc.start_line,
        bug.loc.start_col,
    )
}
