//! Vyper AST normalization passes.

use super::source_unit::SourceUnit;

pub mod flatten_expr;
pub mod rename_defs;
pub mod rename_vars;

/// Run all normalization passes on a Vyper source unit.
pub fn run_passes(source_unit: &SourceUnit) -> SourceUnit {
    let su = rename_vars::rename_vars(source_unit);
    let su = rename_defs::rename_defs(&su);
    flatten_expr::flatten_expr(&su)
}
