use common::loc::Loc;
use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------
// Data structures representing smart contract bugs.
//-------------------------------------------------------------------------

use std::fmt::{self, Display};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bug {
    pub name: String,
    pub description: Option<String>,
    pub loc: Loc,
    pub kind: BugKind,
    pub category: BugCategory,
    pub risk_level: RiskLevel,
    pub cwe_ids: Vec<usize>, // Related CWE: https://cwe.mitre.org/index.html
    pub swc_ids: Vec<usize>, // Related SWC: https://swcregistry.io/
    pub remediation: Option<String>,
}

// FIXME: find a better name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BugKind {
    Optimization,
    Refactoring,
    Vulnerability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    No,
    Low,
    Medium,
    High,
    Critical,
}

/// Classification of bugs by vulnerability category, aligned with the
/// SmartBugs dataset categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BugCategory {
    Reentrancy,
    Arithmetic,
    AccessControl,
    UncheckedLowLevelCalls,
    DenialOfService,
    BadRandomness,
    FrontRunning,
    TimeManipulation,
    ShortAddresses,
    CodeQuality,
    Other,
}

//-------------------------------------------------------------------------
// Implementation for BugCategory
//-------------------------------------------------------------------------

impl BugCategory {
    /// Parse from dataset annotation name (e.g., "REENTRANCY",
    /// "UNCHECKED_LL_CALLS").
    pub fn from_annotation(name: &str) -> Option<BugCategory> {
        // Normalize: trim, uppercase, strip leading/trailing whitespace
        let normalized = name.trim().to_uppercase();
        // Handle "OTHER - ..." style annotations
        let key = if normalized.starts_with("OTHER") {
            "OTHER"
        } else {
            normalized.as_str()
        };
        match key {
            "REENTRANCY" => Some(BugCategory::Reentrancy),
            "ARITHMETIC" => Some(BugCategory::Arithmetic),
            "ACCESS_CONTROL" => Some(BugCategory::AccessControl),
            "UNCHECKED_LL_CALLS" => Some(BugCategory::UncheckedLowLevelCalls),
            "DENIAL_OF_SERVICE" => Some(BugCategory::DenialOfService),
            "BAD_RANDOMNESS" => Some(BugCategory::BadRandomness),
            "FRONT_RUNNING" => Some(BugCategory::FrontRunning),
            "TIME_MANIPULATION" => Some(BugCategory::TimeManipulation),
            "SHORT_ADDRESSES" => Some(BugCategory::ShortAddresses),
            "OTHER" => Some(BugCategory::Other),
            _ => None,
        }
    }

    /// Convert to dataset annotation name.
    pub fn to_annotation(&self) -> &'static str {
        match self {
            BugCategory::Reentrancy => "REENTRANCY",
            BugCategory::Arithmetic => "ARITHMETIC",
            BugCategory::AccessControl => "ACCESS_CONTROL",
            BugCategory::UncheckedLowLevelCalls => "UNCHECKED_LL_CALLS",
            BugCategory::DenialOfService => "DENIAL_OF_SERVICE",
            BugCategory::BadRandomness => "BAD_RANDOMNESS",
            BugCategory::FrontRunning => "FRONT_RUNNING",
            BugCategory::TimeManipulation => "TIME_MANIPULATION",
            BugCategory::ShortAddresses => "SHORT_ADDRESSES",
            BugCategory::CodeQuality => "CODE_QUALITY",
            BugCategory::Other => "OTHER",
        }
    }

    /// Get a human-readable display name.
    pub fn as_str(&self) -> &'static str {
        match self {
            BugCategory::Reentrancy => "Reentrancy",
            BugCategory::Arithmetic => "Arithmetic",
            BugCategory::AccessControl => "Access Control",
            BugCategory::UncheckedLowLevelCalls => "Unchecked Low Level Calls",
            BugCategory::DenialOfService => "Denial of Service",
            BugCategory::BadRandomness => "Bad Randomness",
            BugCategory::FrontRunning => "Front Running",
            BugCategory::TimeManipulation => "Time Manipulation",
            BugCategory::ShortAddresses => "Short Addresses",
            BugCategory::CodeQuality => "Code Quality",
            BugCategory::Other => "Other",
        }
    }
}

impl Display for BugCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

//-------------------------------------------------------------------------
// Implementation for Bug
//-------------------------------------------------------------------------

impl Bug {
    pub fn new(
        name: &str,
        description: Option<&str>,
        loc: Loc,
        kind: BugKind,
        category: BugCategory,
        risk_level: RiskLevel,
        cwe_ids: Vec<usize>,
        swc_ids: Vec<usize>,
        remediation: Option<&str>,
    ) -> Self {
        Bug {
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            loc,
            kind,
            category,
            risk_level,
            swc_ids,
            cwe_ids,
            remediation: remediation.map(|s| s.to_string()),
        }
    }

    /// Format this bug with a source code snippet.
    pub fn format_with_snippet(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("{} ({})\n\n", self.name, self.category));

        // Code snippet
        if let Some(ref file) = self.loc.file {
            if self.loc.is_valid() {
                if let Some(snippet) = common::snippet::extract_snippet(
                    file,
                    self.loc.start_line,
                    self.loc.end_line,
                    self.loc.start_col,
                    self.loc.end_col,
                    1,
                ) {
                    out.push_str(&snippet);
                } else {
                    out.push_str("<source code line not available>\n");
                }
            } else {
                out.push_str("<source code line not available>\n");
            }
        } else {
            out.push_str("<source code line not available>\n");
        }

        out.push_str(&format!(
            "\n- Description: {}\n",
            self.description.as_deref().unwrap_or("None")
        ));
        out.push_str(&format!("- Severity: {}\n", self.risk_level));
        if let Some(ref remedy) = self.remediation {
            out.push_str(&format!("- Remediation: {}\n", remedy));
        }

        let loc_str = if self.loc.start_col == 0 || self.loc.end_col == 0 {
            format!("{}", self.loc.start_line)
        } else if self.loc.start_line == self.loc.end_line {
            format!("{}:{}-{}", self.loc.start_line, self.loc.start_col, self.loc.end_col)
        } else {
            format!(
                "{}:{}-{}:{}",
                self.loc.start_line, self.loc.start_col, self.loc.end_line, self.loc.end_col
            )
        };

        if let Some(ref file) = self.loc.file {
            out.push_str(&format!("- Location: {}:{}\n", file, loc_str));
        } else {
            out.push_str(&format!("- Location: <unknown>:{}\n", loc_str));
        }

        out
    }
}

impl Display for Bug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Bug: {}", self.name)?;
        if let Some(ref desc) = self.description {
            writeln!(f, "Description: {}", desc)?;
        } else {
            writeln!(f, "Description: No description provided")?;
        }
        writeln!(f, "Kind: {}", self.kind)?;
        writeln!(f, "Category: {}", self.category)?;
        writeln!(f, "Risk Level: {}", self.risk_level)?;
        if !self.cwe_ids.is_empty() {
            writeln!(f, "Related CWE IDs: {:?}", self.cwe_ids)?;
        }
        if !self.swc_ids.is_empty() {
            writeln!(f, "Related SWC IDs: {:?}", self.swc_ids)?;
        }
        if let Some(ref remedy) = self.remediation {
            writeln!(f, "Remediation: {}", remedy)?;
        }
        Ok(())
    }
}

//-------------------------------------------------------------------------
// Implementation for BugKind
//-------------------------------------------------------------------------

impl BugKind {
    pub fn as_str(&self) -> &str {
        match self {
            BugKind::Optimization => "Optimization",
            BugKind::Refactoring => "Refactoring",
            BugKind::Vulnerability => "Vulnerability",
        }
    }
}

impl Display for BugKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

//-------------------------------------------------------------------------
// Implementation for RiskLevel
//-------------------------------------------------------------------------

impl RiskLevel {
    pub fn as_str(&self) -> &str {
        match self {
            RiskLevel::No => "Informational",
            RiskLevel::Low => "Low",
            RiskLevel::Medium => "Medium",
            RiskLevel::High => "High",
            RiskLevel::Critical => "Critical",
        }
    }
}

impl Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

//-------------------------------------------------------------------------
// Tests
//-------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bug_category_from_annotation() {
        assert_eq!(BugCategory::from_annotation("REENTRANCY"), Some(BugCategory::Reentrancy));
        assert_eq!(
            BugCategory::from_annotation("ACCESS_CONTROL"),
            Some(BugCategory::AccessControl)
        );
        assert_eq!(
            BugCategory::from_annotation("UNCHECKED_LL_CALLS"),
            Some(BugCategory::UncheckedLowLevelCalls)
        );
        assert_eq!(BugCategory::from_annotation("ARITHMETIC"), Some(BugCategory::Arithmetic));
        assert_eq!(
            BugCategory::from_annotation("BAD_RANDOMNESS"),
            Some(BugCategory::BadRandomness)
        );
        assert_eq!(
            BugCategory::from_annotation("DENIAL_OF_SERVICE"),
            Some(BugCategory::DenialOfService)
        );
        assert_eq!(BugCategory::from_annotation("FRONT_RUNNING"), Some(BugCategory::FrontRunning));
        assert_eq!(
            BugCategory::from_annotation("TIME_MANIPULATION"),
            Some(BugCategory::TimeManipulation)
        );
        assert_eq!(
            BugCategory::from_annotation("SHORT_ADDRESSES"),
            Some(BugCategory::ShortAddresses)
        );
        assert_eq!(BugCategory::from_annotation("OTHER"), Some(BugCategory::Other));
        assert_eq!(
            BugCategory::from_annotation("OTHER - uninitialized storage"),
            Some(BugCategory::Other)
        );
        assert_eq!(BugCategory::from_annotation("UNKNOWN"), None);
    }

    #[test]
    fn test_bug_category_round_trip() {
        let categories = [
            BugCategory::Reentrancy,
            BugCategory::Arithmetic,
            BugCategory::AccessControl,
            BugCategory::UncheckedLowLevelCalls,
            BugCategory::DenialOfService,
            BugCategory::BadRandomness,
            BugCategory::FrontRunning,
            BugCategory::TimeManipulation,
            BugCategory::ShortAddresses,
            BugCategory::Other,
        ];
        for cat in &categories {
            let annotation = cat.to_annotation();
            let parsed = BugCategory::from_annotation(annotation);
            assert_eq!(parsed, Some(*cat), "Round-trip failed for {:?}", cat);
        }
    }

    #[test]
    fn test_bug_category_serde() {
        let cat = BugCategory::Reentrancy;
        let json = serde_json::to_string(&cat).unwrap();
        let parsed: BugCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, cat);
    }

    #[test]
    fn test_bug_serde() {
        let bug = Bug::new(
            "Test Bug",
            Some("A test bug"),
            Loc::new(1, 1, 1, 10),
            BugKind::Vulnerability,
            BugCategory::Reentrancy,
            RiskLevel::High,
            vec![841],
            vec![107],
            Some("Follow the Checks-Effects-Interactions pattern."),
        );
        let json = serde_json::to_string(&bug).unwrap();
        let parsed: Bug = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Test Bug");
        assert_eq!(parsed.category, BugCategory::Reentrancy);
        assert_eq!(parsed.risk_level, RiskLevel::High);
    }

    #[test]
    fn test_risk_level_display() {
        assert_eq!(RiskLevel::No.as_str(), "Informational");
        assert_eq!(RiskLevel::Low.as_str(), "Low");
        assert_eq!(RiskLevel::Medium.as_str(), "Medium");
        assert_eq!(RiskLevel::High.as_str(), "High");
        assert_eq!(RiskLevel::Critical.as_str(), "Critical");
    }

    #[test]
    fn test_bug_kind_display() {
        assert_eq!(BugKind::Optimization.as_str(), "Optimization");
        assert_eq!(BugKind::Refactoring.as_str(), "Refactoring");
        assert_eq!(BugKind::Vulnerability.as_str(), "Vulnerability");
    }

    #[test]
    fn test_bug_category_display() {
        assert_eq!(format!("{}", BugCategory::Reentrancy), "Reentrancy");
        assert_eq!(
            format!("{}", BugCategory::UncheckedLowLevelCalls),
            "Unchecked Low Level Calls"
        );
    }
}
