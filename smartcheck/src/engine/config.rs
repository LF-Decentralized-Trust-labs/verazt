//-------------------------------------------------------------------------
// Data structures Representing analysis configuration
//-------------------------------------------------------------------------

#[derive(Clone)]
pub struct Config {
    pub enable_logging: bool,
    // Flags to control various analysis checks
    pub check_bug_reentrancy: bool,
}
