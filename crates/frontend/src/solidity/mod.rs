// Hack to print log when running unit test

// Hack to print log when running unit test
pub mod ast;

pub use mlir::sir;

pub mod irgen;
pub mod parser;

use common::error::Result;

/// Extract the pragma version string from a `.sol` file.
///
/// Returns the combined pragma constraint (e.g. `"^0.8.25"`) or `None` if
/// absent.
pub fn extract_pragma(file: &str) -> Result<Option<String>> {
    let versions = ast::utils::version::find_pragma_solidity_versions(file)?;
    if versions.is_empty() {
        Ok(None)
    } else {
        Ok(Some(versions.join(", ")))
    }
}

/// Query binaries.soliditylang.org and return installable solc versions
/// satisfying `pragma`. Versions are sorted newest-first.
pub fn find_installable_versions(pragma: &str) -> Result<Vec<node_semver::Version>> {
    ast::utils::version::find_installable_solc_versions(pragma)
}

/// Install a specific solc version via `solc-select`.
pub fn install_version(ver: &node_semver::Version) -> Result<()> {
    parser::configure_solc_compiler(ver)
}
