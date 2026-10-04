//! Static description of a detector: its identity, the classification of
//! its findings, and the guidance attached to them.

use super::id::DetectorId;
use super::traits::ConfidenceLevel;
use crate::context::InputLanguage;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;

// ═══════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════

/// Everything about a detector that does not depend on the analyzed code.
#[derive(Debug)]
pub struct DetectorMeta {
    pub bug_category: BugCategory,
    pub bug_kind: BugKind,
    pub confidence: ConfidenceLevel,
    /// Associated CWE (Common Weakness Enumeration) IDs.
    pub cwe_ids: &'static [usize],
    /// Short description of what the detector checks.
    pub description: &'static str,
    pub id: DetectorId,
    /// Human-readable name, also the title of each finding.
    pub name: &'static str,
    /// How to fix a finding.
    pub recommendation: &'static str,
    /// Documentation links.
    pub references: &'static [&'static str],
    pub risk_level: RiskLevel,
    /// Associated SWC (Smart Contract Weakness Classification) IDs.
    pub swc_ids: &'static [usize],
    pub target: Target,
}

/// The platform a detector applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    /// EVM-based languages (Solidity, Vyper).
    Evm,
    /// Move-based languages (Sui, Aptos).
    Move,
    /// Solana programs.
    Solana,
}

// ═══════════════════════════════════════════════════════════════════
// DetectorMeta Implementations
// ═══════════════════════════════════════════════════════════════════

impl DetectorMeta {
    /// A finding of this detector at `loc`.
    pub fn bug(&self, description: Option<&str>, loc: Loc) -> Bug {
        Bug::new(
            self.name,
            description,
            loc,
            self.bug_kind.clone(),
            self.bug_category,
            self.risk_level,
            self.cwe_ids.to_vec(),
            self.swc_ids.to_vec(),
            Some(self.recommendation),
        )
    }
}

// ═══════════════════════════════════════════════════════════════════
// Target Implementations
// ═══════════════════════════════════════════════════════════════════

impl Target {
    /// The platform programs written in `language` run on.
    pub fn of(language: InputLanguage) -> Self {
        match language {
            InputLanguage::MoveAptos | InputLanguage::MoveSui => Target::Move,
            InputLanguage::Solana => Target::Solana,
            InputLanguage::Solidity | InputLanguage::Vyper => Target::Evm,
        }
    }
}
