//-------------------------------------------------------------------------
// Data structures representing smart contract issues.
//-------------------------------------------------------------------------

use std::fmt::{self, Display};

pub struct Issue {
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub cwe_ids: Vec<usize>, // Related CWE: https://cwe.mitre.org/index.html
    pub swc_ids: Vec<usize>, // Related SWC: https://swcregistry.io/
}

pub enum Severity {
    Informational,
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
        name: String,
        description: String,
        severity: Severity,
        cwe_ids: Vec<usize>,
        swc_ids: Vec<usize>,
    ) -> Self {
        Issue { name, description, severity, swc_ids, cwe_ids }
    }
}

impl Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Issue: {}", self.name)?;
        writeln!(f, "Description: {}", self.description)?;
        writeln!(f, "Severity: {}", self.severity)?;
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
// Implementation for Severity
//-------------------------------------------------------------------------

impl Severity {}

impl Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Informational => write!(f, "Informational"),
            Severity::Low => write!(f, "Low"),
            Severity::Medium => write!(f, "Medium"),
            Severity::High => write!(f, "High"),
            Severity::Critical => write!(f, "Critical"),
        }
    }
}
