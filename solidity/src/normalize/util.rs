//! Module providing helper functions for normlization.

use color_eyre::Result;

/// Helper function to configure unit test.
pub fn configure_unit_test_env() -> Result<()> {
    color_eyre::install()
}
