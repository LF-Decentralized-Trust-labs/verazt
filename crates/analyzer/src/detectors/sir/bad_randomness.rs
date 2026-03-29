//! Bad Randomness Detector (SIR structural)
//!
//! Detects use of on-chain attributes (block.timestamp, blockhash,
//! block.number, block.difficulty, block.coinbase, block.gaslimit) as
//! sources of randomness. These are predictable by miners and other
//! chain participants.

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::passes::base::Pass;
use crate::passes::base::meta::PassLevel;
use crate::passes::base::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::exprs::{BinOp, Expr};
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{BinOpExpr, ContractDecl, DialectExpr, FunctionDecl};
use std::any::TypeId;

/// SIR structural detector for bad randomness.
///
/// Flags usage of on-chain attributes (blockhash, timestamp, block.number,
/// block.difficulty, etc.) as sources of randomness.
#[derive(Debug, Default)]
pub struct BadRandomnessSirDetector;

impl BadRandomnessSirDetector {
    pub fn new() -> Self {
        Self
    }
}

/// Returns the name of a randomness-source EVM expression, or `None` if
/// the expression is not a known weak randomness source.
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

/// Returns the location from a randomness-source EVM expression.
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

/// Recursively check if an expression contains a weak randomness source.
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

/// Collect all randomness source names from an expression tree.
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

impl Pass for BadRandomnessSirDetector {
    fn name(&self) -> &'static str {
        "Bad Randomness"
    }

    fn description(&self) -> &'static str {
        "Detects use of on-chain attributes as sources of randomness."
    }

    fn level(&self) -> PassLevel {
        PassLevel::Expression
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl BugDetectionPass for BadRandomnessSirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::BadRandomness
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        struct Visitor<'b> {
            detector: &'b BadRandomnessSirDetector,
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
        }

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
            fn visit_contract_decl(&mut self, contract: &'a ContractDecl) {
                self.contract_name = contract.name.clone();
                visit::default::visit_contract_decl(self, contract);
            }

            fn visit_function_decl(&mut self, func: &'a FunctionDecl) {
                self.func_name = func.name.clone();
                visit::default::visit_function_decl(self, func);
            }

            fn visit_dialect_expr(&mut self, d: &'a DialectExpr) {
                // Pattern 1: Direct usage of weak randomness sources
                // inside hash functions (detected by checking the parent
                // context in visit_call_expr and visit_binop_expr below).
                // Here we just flag any direct usage of blockhash as it is
                // the most common bad-randomness source.
                if let DialectExpr::Evm(evm) = d {
                    if let Some(source_name) = randomness_source_name(evm) {
                        // Only flag blockhash directly — other block
                        // attributes are flagged when used in hash or
                        // modulo context.
                        if matches!(evm, EvmExpr::Blockhash(_)) {
                            self.bugs.push(Bug::new(
                                self.detector.name(),
                                Some(&format!(
                                    "Weak randomness source: '{}' used in '{}.{}'. \
                                     blockhash is predictable and should not be used \
                                     for randomness.",
                                    source_name, self.contract_name, self.func_name
                                )),
                                randomness_source_loc(evm),
                                self.detector.bug_kind(),
                                self.detector.bug_category(),
                                self.detector.risk_level(),
                                self.detector.cwe_ids(),
                                self.detector.swc_ids(),
                                Some(self.detector.recommendation()),
                            ));
                        }
                    }
                }
            }

            fn visit_call_expr(&mut self, call: &'a scirs::sir::CallExpr) {
                // Pattern 2: Randomness source used as argument to hash
                // functions (keccak256, sha256).
                if let Expr::Dialect(DialectExpr::Evm(
                    EvmExpr::Keccak256(_) | EvmExpr::Sha256(_),
                )) = &*call.callee
                {
                    for arg in call.args.exprs() {
                        if contains_randomness_source(arg) {
                            let mut sources = Vec::new();
                            collect_randomness_sources(arg, &mut sources);
                            let span = call.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0));
                            self.bugs.push(Bug::new(
                                self.detector.name(),
                                Some(&format!(
                                    "Weak randomness: {} used as input to hash \
                                     function in '{}.{}'. On-chain data is \
                                     predictable by miners.",
                                    sources.join(", "),
                                    self.contract_name,
                                    self.func_name
                                )),
                                span,
                                self.detector.bug_kind(),
                                self.detector.bug_category(),
                                self.detector.risk_level(),
                                self.detector.cwe_ids(),
                                self.detector.swc_ids(),
                                Some(self.detector.recommendation()),
                            ));
                        }
                    }
                }

                // Also check EVM hash dialect expressions directly
                match &*call.callee {
                    Expr::Dialect(DialectExpr::Evm(EvmExpr::Keccak256(k))) => {
                        if contains_randomness_source(&k.expr) {
                            let mut sources = Vec::new();
                            collect_randomness_sources(&k.expr, &mut sources);
                            self.bugs.push(Bug::new(
                                self.detector.name(),
                                Some(&format!(
                                    "Weak randomness: {} used as input to keccak256 \
                                     in '{}.{}'. On-chain data is predictable by miners.",
                                    sources.join(", "),
                                    self.contract_name,
                                    self.func_name
                                )),
                                k.loc.clone(),
                                self.detector.bug_kind(),
                                self.detector.bug_category(),
                                self.detector.risk_level(),
                                self.detector.cwe_ids(),
                                self.detector.swc_ids(),
                                Some(self.detector.recommendation()),
                            ));
                        }
                    }
                    Expr::Dialect(DialectExpr::Evm(EvmExpr::Sha256(s))) => {
                        if contains_randomness_source(&s.expr) {
                            let mut sources = Vec::new();
                            collect_randomness_sources(&s.expr, &mut sources);
                            self.bugs.push(Bug::new(
                                self.detector.name(),
                                Some(&format!(
                                    "Weak randomness: {} used as input to sha256 \
                                     in '{}.{}'. On-chain data is predictable by miners.",
                                    sources.join(", "),
                                    self.contract_name,
                                    self.func_name
                                )),
                                s.loc.clone(),
                                self.detector.bug_kind(),
                                self.detector.bug_category(),
                                self.detector.risk_level(),
                                self.detector.cwe_ids(),
                                self.detector.swc_ids(),
                                Some(self.detector.recommendation()),
                            ));
                        }
                    }
                    _ => {}
                }

                visit::default::visit_call_expr(self, call);
            }

            fn visit_binop_expr(&mut self, expr: &'a BinOpExpr) {
                // Pattern 3: Randomness source used as operand of modulo
                if expr.op == BinOp::Mod && contains_randomness_source(&expr.lhs) {
                    let mut sources = Vec::new();
                    collect_randomness_sources(&expr.lhs, &mut sources);
                    self.bugs.push(Bug::new(
                        self.detector.name(),
                        Some(&format!(
                            "Weak randomness: {} used with modulo operator in \
                             '{}.{}'. On-chain data is predictable by miners.",
                            sources.join(", "),
                            self.contract_name,
                            self.func_name
                        )),
                        expr.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                        self.detector.bug_kind(),
                        self.detector.bug_category(),
                        self.detector.risk_level(),
                        self.detector.cwe_ids(),
                        self.detector.swc_ids(),
                        Some(self.detector.recommendation()),
                    ));
                }

                visit::default::visit_binop_expr(self, expr);
            }
        }

        let mut visitor = Visitor {
            detector: self,
            bugs: &mut bugs,
            contract_name: String::new(),
            func_name: String::new(),
        };
        visitor.visit_modules(context.ir_units());

        Ok(bugs)
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::BadRandomness
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![330] // CWE-330: Use of Insufficiently Random Values
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![120] // SWC-120: Weak Sources of Randomness from Chain Attributes
    }

    fn recommendation(&self) -> &'static str {
        "Do not use on-chain data (blockhash, block.timestamp, block.number, \
         block.difficulty) as a source of randomness. Use Chainlink VRF or \
         a commit-reveal scheme instead."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-120",
            "https://docs.chain.link/vrf/v2/introduction",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bad_randomness_sir_detector() {
        let detector = BadRandomnessSirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::BadRandomness);
        assert_eq!(detector.swc_ids(), vec![120]);
        assert_eq!(detector.cwe_ids(), vec![330]);
        assert_eq!(detector.risk_level(), RiskLevel::High);
        assert_eq!(detector.bug_category(), BugCategory::BadRandomness);
    }
}
