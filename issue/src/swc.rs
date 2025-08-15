//-------------------------------------------------------------------------
// Data structures representing SWC
//-------------------------------------------------------------------------

pub struct SWC {
    id: String,
    title: String,
    description: String,
    remediation: String,
    cwe_ids: Vec<usize>, // CWE related to this SWC
    references: String,
}
