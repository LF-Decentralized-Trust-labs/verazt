//! Lowering: Solidity AST → SIR.
//!
//! This module is a self-contained pipeline that takes raw, source-faithful
//! ASTs and produces SIR.  Callers no longer invoke a separate normalize step.

pub mod lower;
pub mod normalize;

use crate::solidity::ast::SourceUnit;
use common::error::Result;
use scirs::sir::Module;

pub use lower::IrGen;

/// Lower one or more Solidity source units into SIR modules.
///
/// Internally runs all normalization passes before lowering.
pub fn lower_source_units(source_units: &[SourceUnit]) -> Result<Vec<Module>> {
    let normalized = normalize::run_passes(source_units);
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
