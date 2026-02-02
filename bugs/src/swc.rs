//-------------------------------------------------------------------------
// Data structures representing SWC
//-------------------------------------------------------------------------

pub struct SWC {
    _id: String,
    _title: String,
    _description: String,
    _remediation: String,
    _cwe_ids: Vec<usize>, // CWE related to this SWC
    _references: String,
}
