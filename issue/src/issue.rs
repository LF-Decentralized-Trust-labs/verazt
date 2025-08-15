//-------------------------------------------------------------------------
// Data structures representing smart contract issues.
//-------------------------------------------------------------------------

pub struct Issue {
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub cwe_id: Option<usize>, // CWE: https://cwe.mitre.org/index.html
    pub swc_id: Option<usize>, // SWC: https://swcregistry.io/
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
        cwe_id: Option<usize>,
        swc_id: Option<usize>,
    ) -> Self {
        Issue { name, description, severity, swc_id, cwe_id }
    }
}
