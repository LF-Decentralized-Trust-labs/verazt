//! Lowering: Solidity AST → SIR.
//!
//! This module is a self-contained pipeline that takes raw, source-faithful
//! ASTs and produces SIR.  Callers no longer invoke a separate normalize step.

pub mod lower;

pub mod eliminate_imports;
pub mod eliminate_modifiers;
pub mod eliminate_named_args;
pub mod eliminate_using;
pub mod flatten_expr;
pub mod flatten_names;
pub mod merge_pragmas;
pub mod rename_callees;
pub mod rename_contracts;
pub mod rename_defs;
pub mod rename_vars;
pub mod resolve_inheritance;
pub mod strip_specifiers;
pub mod substitution;
pub mod unroll_tuples;

#[macro_use]
pub mod utils;

pub use eliminate_imports::eliminate_import;
pub use eliminate_using::eliminate_using_directives;
pub use flatten_expr::flatten_expr;
pub use flatten_names::flatten_name;
pub use merge_pragmas::merge_pragmas;
pub use rename_callees::rename_callees;
pub use rename_contracts::rename_contracts;
pub use rename_defs::rename_defs;
pub use rename_vars::rename_vars;
pub use unroll_tuples::unroll_unary_tuple;

use crate::solidity::ast::SourceUnit;
use common::error::Result;
use scirs::sir::Module;

pub use lower::Lowerer;

/// Lower one or more Solidity source units into SIR modules.
///
/// Internally runs all normalization passes before lowering.
pub fn lower_source_units(source_units: &[SourceUnit]) -> Result<Vec<Module>> {
    let normalized = lower::run_passes(source_units);
    normalized
        .iter()
        .map(|su| lower::lower_source_unit(su))
        .collect()
}

/// Lower a **single** (already-normalized) source unit into a SIR module.
///
/// Prefer [`lower_source_units`] for the full pipeline.  This entry point
/// is kept for callers that have already normalized the AST externally.
pub fn lower_source_unit(source_unit: &SourceUnit) -> Result<Module> {
    lower::lower_source_unit(source_unit)
}
