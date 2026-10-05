//! Bad Randomness Detector
//!
//! Detects use of on-chain attributes (block.timestamp, blockhash,
//! block.number, block.difficulty, block.coinbase, block.gaslimit) as
//! sources of randomness.

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::exprs::{BinOp, Expr};
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{BinOpExpr, CallExpr, ContractDecl, DialectExpr, FunctionDecl, Module};

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::BadRandomness,
    bug_kind: BugKind::Vulnerability,
    confidence: ConfidenceLevel::Medium,
    cwe_ids: &[330],
    description: "Detects use of on-chain attributes as sources of randomness.",
    id: DetectorId::BadRandomness,
    name: "Bad Randomness",
    recommendation: "Do not use on-chain data (blockhash, block.timestamp, block.number, \
         block.difficulty) as a source of randomness. Use Chainlink VRF or \
         a commit-reveal scheme instead.",
    references: &[
        "https://swcregistry.io/docs/SWC-120",
        "https://docs.chain.link/vrf/v2/introduction",
    ],
    risk_level: RiskLevel::High,
    swc_ids: &[120],
    target: Target::Evm,
};

/// Scan detector for bad randomness.
#[derive(Debug, Default)]
pub struct BadRandomnessDetector;

impl BadRandomnessDetector {
    pub fn new() -> Self {
        Self
    }
}

fn randomness_source_name(evm: &EvmExpr) -> Option<&'static str> {
    match evm {
        EvmExpr::Blockhash(_) => Some("blockhash"),
        EvmExpr::Timestamp(_) => Some("block.timestamp"),
        EvmExpr::BlockNumber(_) => Some("block.number"),
        EvmExpr::BlockDifficulty(_) => Some("block.difficulty/prevrandao"),
        EvmExpr::BlockCoinbase(_) => Some("block.coinbase"),
        EvmExpr::BlockGaslimit(_) => Some("block.gaslimit"),
        _ => None,
    }
}

/// Collects the randomness sources in the visited expressions, without
/// duplicates.
#[derive(Default)]
struct SourceCollector {
    sources: Vec<&'static str>,
}

impl<'a> Visit<'a> for SourceCollector {
    fn visit_dialect_expr(&mut self, d: &'a DialectExpr) {
        if let DialectExpr::Evm(evm) = d
            && let Some(name) = randomness_source_name(evm)
            && !self.sources.contains(&name)
        {
            self.sources.push(name);
        }
        visit::default::visit_dialect_expr(self, d);
    }
}

/// The randomness sources in `exprs`.
fn randomness_sources<'a>(exprs: impl IntoIterator<Item = &'a Expr>) -> Vec<&'static str> {
    let mut collector = SourceCollector::default();
    for expr in exprs {
        collector.visit_expr(expr);
    }
    collector.sources
}

impl ScanDetector for BadRandomnessDetector {
    fn meta(&self) -> &'static DetectorMeta {
        &META
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Function
    }

    fn check_function(
        &self,
        func: &FunctionDecl,
        contract: &ContractDecl,
        _module: &Module,
    ) -> Vec<Bug> {
        let mut bugs = Vec::new();

        struct Visitor<'b> {
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
        }

        impl Visitor<'_> {
            /// Report the randomness sources hashed by `hash_name`, if any.
            fn check_hash<'a>(
                &mut self,
                hash_name: &str,
                input: impl IntoIterator<Item = &'a Expr>,
                loc: Loc,
            ) {
                let sources = randomness_sources(input);
                if !sources.is_empty() {
                    self.bugs.push(META.bug(
                        Some(&format!(
                            "Weak randomness: {} used as input to {} in '{}.{}'. \
                             On-chain data is predictable by miners.",
                            sources.join(", "),
                            hash_name,
                            self.contract_name,
                            self.func_name
                        )),
                        loc,
                    ));
                }
            }
        }

        impl<'a> Visit<'a> for Visitor<'_> {
            fn visit_dialect_expr(&mut self, d: &'a DialectExpr) {
                match d {
                    DialectExpr::Evm(EvmExpr::Blockhash(e)) => {
                        self.bugs.push(META.bug(
                            Some(&format!(
                                "Weak randomness source: 'blockhash' used in '{}.{}'. \
                                 blockhash is predictable and should not be used \
                                 for randomness.",
                                self.contract_name, self.func_name
                            )),
                            e.loc.clone(),
                        ));
                    }
                    DialectExpr::Evm(EvmExpr::Keccak256(e)) => {
                        self.check_hash("keccak256", [&*e.expr], e.loc.clone())
                    }
                    DialectExpr::Evm(EvmExpr::Sha256(e)) => {
                        self.check_hash("sha256", [&*e.expr], e.loc.clone())
                    }
                    _ => {}
                }
                visit::default::visit_dialect_expr(self, d);
            }

            // Vyper lowers its hash builtins to calls of `keccak256` and
            // `sha256` rather than to EVM dialect expressions.
            fn visit_call_expr(&mut self, call: &'a CallExpr) {
                if let Expr::Var(callee) = &*call.callee
                    && matches!(callee.name.as_str(), "keccak256" | "sha256")
                {
                    let loc = call.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0));
                    self.check_hash(&callee.name, call.args.exprs(), loc);
                }
                visit::default::visit_call_expr(self, call);
            }

            fn visit_binop_expr(&mut self, expr: &'a BinOpExpr) {
                if expr.op == BinOp::Mod {
                    let sources = randomness_sources([&*expr.lhs]);
                    if !sources.is_empty() {
                        self.bugs.push(META.bug(
                            Some(&format!(
                                "Weak randomness: {} used with modulo operator in \
                                 '{}.{}'. On-chain data is predictable by miners.",
                                sources.join(", "),
                                self.contract_name,
                                self.func_name
                            )),
                            expr.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                        ));
                    }
                }
                visit::default::visit_binop_expr(self, expr);
            }
        }

        let mut visitor = Visitor {
            bugs: &mut bugs,
            contract_name: contract.name.clone(),
            func_name: func.name.clone(),
        };
        visitor.visit_function_decl(func);

        bugs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bad_randomness_detector() {
        let detector = BadRandomnessDetector::new();
        assert_eq!(detector.meta().id, DetectorId::BadRandomness);
        assert_eq!(detector.meta().risk_level, RiskLevel::High);
    }
}
