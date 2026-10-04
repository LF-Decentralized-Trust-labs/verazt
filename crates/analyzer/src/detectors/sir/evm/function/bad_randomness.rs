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
use scirs::sir::{BinOpExpr, ContractDecl, DialectExpr, FunctionDecl, Module};

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

fn randomness_source_loc(evm: &EvmExpr) -> Loc {
    match evm {
        EvmExpr::Blockhash(e) => e.loc.clone(),
        EvmExpr::Timestamp(e) => e.loc.clone(),
        EvmExpr::BlockNumber(e) => e.loc.clone(),
        EvmExpr::BlockDifficulty(e) => e.loc.clone(),
        EvmExpr::BlockCoinbase(e) => e.loc.clone(),
        EvmExpr::BlockGaslimit(e) => e.loc.clone(),
        _ => Loc::new(0, 0, 0, 0),
    }
}

fn contains_randomness_source(expr: &Expr) -> bool {
    match expr {
        Expr::Dialect(DialectExpr::Evm(evm)) => randomness_source_name(evm).is_some(),
        Expr::BinOp(bin) => {
            contains_randomness_source(&bin.lhs) || contains_randomness_source(&bin.rhs)
        }
        Expr::UnOp(un) => contains_randomness_source(&un.operand),
        Expr::FunctionCall(call) => {
            contains_randomness_source(&call.callee)
                || call
                    .args
                    .exprs()
                    .iter()
                    .any(|a| contains_randomness_source(a))
        }
        Expr::TypeCast(tc) => contains_randomness_source(&tc.expr),
        Expr::IndexAccess(ia) => {
            contains_randomness_source(&ia.base)
                || ia
                    .index
                    .as_ref()
                    .is_some_and(|i| contains_randomness_source(i))
        }
        Expr::FieldAccess(fa) => contains_randomness_source(&fa.base),
        Expr::Ternary(t) => {
            contains_randomness_source(&t.cond)
                || contains_randomness_source(&t.then_expr)
                || contains_randomness_source(&t.else_expr)
        }
        Expr::Tuple(t) => t
            .elems
            .iter()
            .any(|e| e.as_ref().is_some_and(contains_randomness_source)),
        _ => false,
    }
}

fn collect_randomness_sources(expr: &Expr, sources: &mut Vec<&'static str>) {
    match expr {
        Expr::Dialect(DialectExpr::Evm(evm)) => {
            if let Some(name) = randomness_source_name(evm) {
                if !sources.contains(&name) {
                    sources.push(name);
                }
            }
        }
        Expr::BinOp(bin) => {
            collect_randomness_sources(&bin.lhs, sources);
            collect_randomness_sources(&bin.rhs, sources);
        }
        Expr::UnOp(un) => collect_randomness_sources(&un.operand, sources),
        Expr::FunctionCall(call) => {
            collect_randomness_sources(&call.callee, sources);
            for arg in call.args.exprs() {
                collect_randomness_sources(arg, sources);
            }
        }
        Expr::TypeCast(tc) => collect_randomness_sources(&tc.expr, sources),
        Expr::IndexAccess(ia) => {
            collect_randomness_sources(&ia.base, sources);
            if let Some(idx) = &ia.index {
                collect_randomness_sources(idx, sources);
            }
        }
        Expr::FieldAccess(fa) => collect_randomness_sources(&fa.base, sources),
        Expr::Ternary(t) => {
            collect_randomness_sources(&t.cond, sources);
            collect_randomness_sources(&t.then_expr, sources);
            collect_randomness_sources(&t.else_expr, sources);
        }
        Expr::Tuple(t) => {
            for elem in &t.elems {
                if let Some(e) = elem {
                    collect_randomness_sources(e, sources);
                }
            }
        }
        _ => {}
    }
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

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
            fn visit_dialect_expr(&mut self, d: &'a DialectExpr) {
                if let DialectExpr::Evm(evm) = d {
                    if let Some(_source_name) = randomness_source_name(evm) {
                        if matches!(evm, EvmExpr::Blockhash(_)) {
                            self.bugs.push(META.bug(
                                Some(&format!(
                                    "Weak randomness source: 'blockhash' used in '{}.{}'. \
                                     blockhash is predictable and should not be used \
                                     for randomness.",
                                    self.contract_name, self.func_name
                                )),
                                randomness_source_loc(evm),
                            ));
                        }
                    }
                }
            }

            fn visit_call_expr(&mut self, call: &'a scirs::sir::CallExpr) {
                if let Expr::Dialect(DialectExpr::Evm(
                    EvmExpr::Keccak256(_) | EvmExpr::Sha256(_),
                )) = &*call.callee
                {
                    for arg in call.args.exprs() {
                        if contains_randomness_source(arg) {
                            let mut sources = Vec::new();
                            collect_randomness_sources(arg, &mut sources);
                            let span = call.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0));
                            self.bugs.push(META.bug(
                                Some(&format!(
                                    "Weak randomness: {} used as input to hash \
                                     function in '{}.{}'. On-chain data is \
                                     predictable by miners.",
                                    sources.join(", "),
                                    self.contract_name,
                                    self.func_name
                                )),
                                span,
                            ));
                        }
                    }
                }

                match &*call.callee {
                    Expr::Dialect(DialectExpr::Evm(EvmExpr::Keccak256(k))) => {
                        if contains_randomness_source(&k.expr) {
                            let mut sources = Vec::new();
                            collect_randomness_sources(&k.expr, &mut sources);
                            self.bugs.push(META.bug(
                                Some(&format!(
                                    "Weak randomness: {} used as input to keccak256 \
                                     in '{}.{}'. On-chain data is predictable by miners.",
                                    sources.join(", "),
                                    self.contract_name,
                                    self.func_name
                                )),
                                k.loc.clone(),
                            ));
                        }
                    }
                    Expr::Dialect(DialectExpr::Evm(EvmExpr::Sha256(s))) => {
                        if contains_randomness_source(&s.expr) {
                            let mut sources = Vec::new();
                            collect_randomness_sources(&s.expr, &mut sources);
                            self.bugs.push(META.bug(
                                Some(&format!(
                                    "Weak randomness: {} used as input to sha256 \
                                     in '{}.{}'. On-chain data is predictable by miners.",
                                    sources.join(", "),
                                    self.contract_name,
                                    self.func_name
                                )),
                                s.loc.clone(),
                            ));
                        }
                    }
                    _ => {}
                }

                visit::default::visit_call_expr(self, call);
            }

            fn visit_binop_expr(&mut self, expr: &'a BinOpExpr) {
                if expr.op == BinOp::Mod && contains_randomness_source(&expr.lhs) {
                    let mut sources = Vec::new();
                    collect_randomness_sources(&expr.lhs, &mut sources);
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
        assert_eq!(detector.meta().risk_level,RiskLevel::High);
    }
}
