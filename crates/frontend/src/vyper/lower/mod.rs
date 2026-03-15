//! Lowering: Vyper AST → SIR.
//!
//! This module is a self-contained pipeline that takes a raw, source-faithful
//! AST and produces SIR.  Callers no longer invoke a separate normalize step.

pub mod lower;
pub mod normalize;

use crate::vyper::ast::source_unit::SourceUnit;
use common::error::Result;
use scirs::sir::Module;

pub use lower::IrGen;

/// Lower a Vyper source unit into a SIR module.
///
/// Internally runs all normalization passes before lowering.
pub fn lower_source_unit_normalized(source_unit: &SourceUnit) -> Result<Module> {
    let normalized = normalize::run_passes(source_unit);
    lower::lower_source_unit(&normalized)
}

/// Lower a **single** (already-normalized) source unit into a SIR module.
///
/// Prefer [`lower_source_unit_normalized`] for the full pipeline.
pub fn lower_source_unit(source_unit: &SourceUnit) -> Result<Module> {
    lower::lower_source_unit(source_unit)
}
