use meta::Loc;

//-------------------------------------------------------------------------
// Data structures representing smart contract issues.
//-------------------------------------------------------------------------

use std::fmt::{self, Display};

pub struct Issue {
    pub name: String,
    pub description: Option<String>,
    pub loc: Loc,
    pub kind: IssueKind,
    pub risk_level: RiskLevel,
    pub cwe_ids: Vec<usize>, // Related CWE: https://cwe.mitre.org/index.html
    pub swc_ids: Vec<usize>, // Related SWC: https://swcregistry.io/
}

// FIXME: find a better name
pub enum IssueKind {
    Optimization,
    Refactoring,
    Vulnerability,
}

pub enum RiskLevel {
    No,
    Low,
    Medium,
    High,
    Critical,
}

//-------------------------------------------------------------------------
// Implementation for Issue
//-------------------------------------------------------------------------

impl Issue {
    pub fn new(
        name: &str,
        description: Option<&str>,
        loc: Loc,
        kind: IssueKind,
        risk_level: RiskLevel,
        cwe_ids: Vec<usize>,
        swc_ids: Vec<usize>,
    ) -> Self {
        Issue {
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            loc,
            kind,
            risk_level,
            swc_ids,
            cwe_ids,
        }
    }
}

impl Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Issue: {}", self.name)?;
        if let Some(ref desc) = self.description {
            writeln!(f, "Description: {}", desc)?;
        } else {
            writeln!(f, "Description: No description provided")?;
        }
        writeln!(f, "Kind: {}", self.kind)?;
        writeln!(f, "Risk Level: {}", self.risk_level)?;
        if !self.cwe_ids.is_empty() {
            writeln!(f, "Related CWE IDs: {:?}", self.cwe_ids)?;
        }
        if !self.swc_ids.is_empty() {
            writeln!(f, "Related SWC IDs: {:?}", self.swc_ids)?;
        }
        Ok(())
    }
}

//-------------------------------------------------------------------------
// Implementation for IssueKind
//-------------------------------------------------------------------------

impl IssueKind {
    pub fn as_str(&self) -> &str {
        match self {
            IssueKind::Optimization => "Optimization",
            IssueKind::Refactoring => "Refactoring",
            IssueKind::Vulnerability => "Vulnerability",
        }
    }
}

impl Display for IssueKind {
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
