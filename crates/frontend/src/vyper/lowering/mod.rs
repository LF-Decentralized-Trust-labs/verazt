//! Lowering: Vyper AST → SIR.
//!
//! This module is a self-contained pipeline that takes a raw, source-faithful
//! AST and produces SIR.  Callers no longer invoke a separate normalize step.

pub mod lower;

pub mod flatten_expr;
pub mod rename_defs;
pub mod rename_vars;

pub use flatten_expr::flatten_expr;
pub use rename_defs::rename_defs;
pub use rename_vars::rename_vars;

use crate::vyper::ast::source_unit::SourceUnit;
use common::error::Result;
use scirs::sir::Module;

pub use lower::Lowerer;

/// Lower a Vyper source unit into a SIR module.
///
/// Internally runs all normalization passes before lowering.
pub fn lower_source_unit_normalized(source_unit: &SourceUnit) -> Result<Module> {
    let normalized = lower::run_passes(source_unit);
    lower::lower_source_unit(&normalized)
}

/// Lower a **single** (already-normalized) source unit into a SIR module.
///
/// Prefer [`lower_source_unit_normalized`] for the full pipeline.
pub fn lower_source_unit(source_unit: &SourceUnit) -> Result<Module> {
    lower::lower_source_unit(source_unit)
}
