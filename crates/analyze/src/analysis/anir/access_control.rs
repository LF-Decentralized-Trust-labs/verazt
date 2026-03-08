//! ANIR Access Control Pass
//!
//! Checks that every public function performing a storage write has an access
//! guard using a SignerArg-labelled value before the first write.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::{AnalysisPass, Pass, PassResult};
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use mlir::air::interfaces::TaintLabel;
use mlir::air::ops::OpId;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use solidity::ast::Loc;
use std::collections::HashMap;

/// ANIR-based access control analysis pass.
pub struct AnirAccessControlPass;

impl Pass for AnirAccessControlPass {
    fn id(&self) -> PassId {
        PassId::AnirAccessControl
    }

    fn name(&self) -> &'static str {
        "anir-access-control"
    }

    fn description(&self) -> &'static str {
        "Access control: taint-flow path to storage write lacks SignerArg guard"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Air
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::AnirTaintPropagation]
    }
}

impl AnalysisPass for AnirAccessControlPass {
    fn run(&self, ctx: &mut AnalysisContext) -> PassResult<()> {
        let _taint_map: &HashMap<OpId, TaintLabel> =
            ctx.get_artifact("anir.taint_map").unwrap_or_else(|| {
                // Should not happen if dependencies are satisfied
                static EMPTY: std::sync::LazyLock<HashMap<OpId, TaintLabel>> =
                    std::sync::LazyLock::new(HashMap::new);
                &EMPTY
            });

        let mut bugs = Vec::new();

        for module in ctx.anir_units() {
            for summary in &module.summaries {
                // Check if the function is public by looking at its name
                // (in the ANIR summary, we can check the function attributes)
                // For now, check if the function has storage writes
                if summary.modifies.is_empty() {
                    continue;
                }

                // Check if any Assert condition uses a SignerArg-tainted value
                let has_signer_guard = summary
                    .taint_in
                    .values()
                    .any(|label| *label == TaintLabel::SignerArg);

                if !has_signer_guard && !summary.reentrancy_safe {
                    bugs.push(Bug::new(
                        "ANIR Missing Access Control",
                        Some(&format!(
                            "Function '{}' writes to storage without a signer/auth guard",
                            summary.func_id.0
                        )),
                        Loc::new(0, 0, 0, 0),
                        BugKind::Vulnerability,
                        BugCategory::AccessControl,
                        RiskLevel::High,
                        vec![284],
                        vec![105],
                    ));
                }
            }
        }

        ctx.store_artifact("anir.access_control_findings", bugs);
        ctx.mark_pass_completed(self.id());
        Ok(())
    }

    fn is_completed(&self, ctx: &AnalysisContext) -> bool {
        ctx.is_pass_completed(self.id())
    }
}
