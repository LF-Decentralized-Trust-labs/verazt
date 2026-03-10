//! Typed Artifact Keys
//!
//! Defines `ArtifactKey` markers for scanner-specific data stored in
//! `AnalysisContext`. This replaces the old stringly-typed
//! `store_artifact("source_units", ...)` / `get_artifact(...)` pattern.

use analysis::context::ArtifactKey;
use frontend::solidity::ast::SourceUnit;

/// Artifact key for Solidity AST source units.
///
/// The Solidity parser produces `Vec<SourceUnit>` which is stored in the
/// analysis context for GREP-tier and AST-based detectors.
pub struct SourceUnitsArtifact;

impl ArtifactKey for SourceUnitsArtifact {
    type Value = Vec<SourceUnit>;
    const NAME: &'static str = "source_units";
}
