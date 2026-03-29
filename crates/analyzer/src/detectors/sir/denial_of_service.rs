//! Denial of Service Detector (SIR structural)
//!
//! Detects patterns that can lead to denial of service:
//! 1. External calls inside loops (SWC-113)
//! 2. `require(addr.send(...))` pattern (SWC-113)
//! 3. Unbounded loops over dynamic storage arrays (SWC-128)

use crate::context::AnalysisContext;
use crate::detectors::base::id::DetectorId;
use crate::detectors::{BugDetectionPass, ConfidenceLevel, DetectorResult};
use crate::passes::base::Pass;
use crate::passes::base::meta::PassLevel;
use crate::passes::base::meta::PassRepresentation;
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use frontend::solidity::ast::Loc;
use scirs::sir::dialect::EvmCallExt;
use scirs::sir::exprs::Expr;
use scirs::sir::stmts::Stmt;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{AssertStmt, ContractDecl, ForStmt, FunctionDecl, WhileStmt};
use std::any::TypeId;

/// SIR structural detector for denial of service vulnerabilities.
#[derive(Debug, Default)]
pub struct DenialOfServiceSirDetector;

impl DenialOfServiceSirDetector {
    pub fn new() -> Self {
        Self
    }
}

/// Check if a list of statements contains an external call.
fn stmts_contain_external_call(stmts: &[Stmt]) -> bool {
    for stmt in stmts {
        if stmt_contains_external_call(stmt) {
            return true;
        }
    }
    false
}

/// Check if a single statement contains an external call.
fn stmt_contains_external_call(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Expr(es) => expr_is_external_call(&es.expr),
        Stmt::If(s) => {
            stmts_contain_external_call(&s.then_body)
                || s.else_body
                    .as_ref()
                    .is_some_and(|e| stmts_contain_external_call(e))
        }
        Stmt::Block(stmts) => stmts_contain_external_call(stmts),
        Stmt::Assign(a) => expr_is_external_call(&a.rhs),
        Stmt::LocalVar(lv) => lv.init.as_ref().is_some_and(expr_is_external_call),
        Stmt::Assert(a) => expr_is_external_call(&a.cond),
        _ => false,
    }
}

/// Check if an expression is or contains an external call.
fn expr_is_external_call(expr: &Expr) -> bool {
    match expr {
        Expr::FunctionCall(call) => {
            if call.is_evm_external_call() {
                return true;
            }
            // Check args
            call.args.exprs().iter().any(|a| expr_is_external_call(a))
        }
        Expr::BinOp(bin) => expr_is_external_call(&bin.lhs) || expr_is_external_call(&bin.rhs),
        Expr::UnOp(un) => expr_is_external_call(&un.operand),
        Expr::Ternary(t) => {
            expr_is_external_call(&t.cond)
                || expr_is_external_call(&t.then_expr)
                || expr_is_external_call(&t.else_expr)
        }
        _ => false,
    }
}

/// Check if an expression is a `require(addr.send(...))` pattern.
/// This checks for assert/require whose condition is an external send/transfer
/// call.
fn is_require_send_pattern(assert: &AssertStmt) -> bool {
    expr_is_send_or_transfer(&assert.cond)
}

/// Check if an expression is a send or transfer external call.
fn expr_is_send_or_transfer(expr: &Expr) -> bool {
    match expr {
        Expr::FunctionCall(call) => {
            if let Expr::FieldAccess(fa) = &*call.callee {
                return matches!(fa.field.as_str(), "send" | "transfer");
            }
            if let Expr::Dialect(scirs::sir::DialectExpr::Evm(
                scirs::sir::dialect::evm::EvmExpr::Send(_)
                | scirs::sir::dialect::evm::EvmExpr::Transfer(_),
            )) = &*call.callee
            {
                return true;
            }
            false
        }
        _ => false,
    }
}

/// Check if a for-loop condition is bounded by a dynamic .length access.
fn is_unbounded_loop_cond(cond: &Option<Expr>) -> bool {
    if let Some(cond_expr) = cond {
        contains_length_access(cond_expr)
    } else {
        false
    }
}

/// Recursively check if an expression contains a `.length` field access.
fn contains_length_access(expr: &Expr) -> bool {
    match expr {
        Expr::FieldAccess(fa) => fa.field == "length" || contains_length_access(&fa.base),
        Expr::BinOp(bin) => contains_length_access(&bin.lhs) || contains_length_access(&bin.rhs),
        _ => false,
    }
}

impl Pass for DenialOfServiceSirDetector {
    fn name(&self) -> &'static str {
        "Denial of Service"
    }

    fn description(&self) -> &'static str {
        "Detects patterns that can lead to denial of service: external calls \
         in loops, require(send) patterns, and unbounded loops."
    }

    fn level(&self) -> PassLevel {
        PassLevel::Statement
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }
}

impl BugDetectionPass for DenialOfServiceSirDetector {
    fn detector_id(&self) -> DetectorId {
        DetectorId::DenialOfService
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        if !context.has_ir() {
            return Ok(bugs);
        }

        struct Visitor<'b> {
            detector: &'b DenialOfServiceSirDetector,
            bugs: &'b mut Vec<Bug>,
            contract_name: String,
            func_name: String,
            /// Track nesting depth inside loops for sub-pattern 1.
            in_loop: bool,
        }

        impl<'a, 'b> Visit<'a> for Visitor<'b> {
            fn visit_contract_decl(&mut self, contract: &'a ContractDecl) {
                self.contract_name = contract.name.clone();
                visit::default::visit_contract_decl(self, contract);
            }

            fn visit_function_decl(&mut self, func: &'a FunctionDecl) {
                self.func_name = func.name.clone();
                self.in_loop = false;
                visit::default::visit_function_decl(self, func);
            }

            fn visit_for_stmt(&mut self, stmt: &'a ForStmt) {
                let was_in_loop = self.in_loop;

                // Sub-pattern 1: External call inside loop
                if stmts_contain_external_call(&stmt.body) {
                    self.bugs.push(Bug::new(
                        self.detector.name(),
                        Some(&format!(
                            "External call inside loop in '{}.{}'. A single \
                             failed call can revert the entire transaction.",
                            self.contract_name, self.func_name
                        )),
                        stmt.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                        self.detector.bug_kind(),
                        self.detector.bug_category(),
                        self.detector.risk_level(),
                        self.detector.cwe_ids(),
                        self.detector.swc_ids(),
                        Some(self.detector.recommendation()),
                    ));
                }

                // Sub-pattern 3: Unbounded loop over dynamic storage .length
                if is_unbounded_loop_cond(&stmt.cond) {
                    self.bugs.push(Bug::new(
                        self.detector.name(),
                        Some(&format!(
                            "Unbounded loop in '{}.{}': loop bound depends on \
                             dynamic array length, which could exceed the block \
                             gas limit.",
                            self.contract_name, self.func_name
                        )),
                        stmt.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                        self.detector.bug_kind(),
                        self.detector.bug_category(),
                        self.detector.risk_level(),
                        self.detector.cwe_ids(),
                        self.detector.swc_ids(),
                        Some(self.detector.recommendation()),
                    ));
                }

                self.in_loop = true;
                visit::default::visit_for_stmt(self, stmt);
                self.in_loop = was_in_loop;
            }

            fn visit_while_stmt(&mut self, stmt: &'a WhileStmt) {
                let was_in_loop = self.in_loop;

                // Sub-pattern 1: External call inside while loop
                if stmts_contain_external_call(&stmt.body) {
                    self.bugs.push(Bug::new(
                        self.detector.name(),
                        Some(&format!(
                            "External call inside loop in '{}.{}'. A single \
                             failed call can revert the entire transaction.",
                            self.contract_name, self.func_name
                        )),
                        stmt.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                        self.detector.bug_kind(),
                        self.detector.bug_category(),
                        self.detector.risk_level(),
                        self.detector.cwe_ids(),
                        self.detector.swc_ids(),
                        Some(self.detector.recommendation()),
                    ));
                }

                self.in_loop = true;
                visit::default::visit_while_stmt(self, stmt);
                self.in_loop = was_in_loop;
            }

            fn visit_assert_stmt(&mut self, stmt: &'a AssertStmt) {
                // Sub-pattern 2: require(addr.send(...))
                if is_require_send_pattern(stmt) {
                    self.bugs.push(Bug::new(
                        self.detector.name(),
                        Some(&format!(
                            "require(send/transfer) in '{}.{}': a single \
                             failed send reverts the entire transaction, \
                             enabling DoS by a malicious recipient.",
                            self.contract_name, self.func_name
                        )),
                        stmt.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                        self.detector.bug_kind(),
                        self.detector.bug_category(),
                        self.detector.risk_level(),
                        self.detector.cwe_ids(),
                        self.detector.swc_ids(),
                        Some(self.detector.recommendation()),
                    ));
                }

                visit::default::visit_assert_stmt(self, stmt);
            }
        }

        let mut visitor = Visitor {
            detector: self,
            bugs: &mut bugs,
            contract_name: String::new(),
            func_name: String::new(),
            in_loop: false,
        };
        visitor.visit_modules(context.ir_units());

        Ok(bugs)
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::DenialOfService
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![400] // CWE-400: Uncontrolled Resource Consumption
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![113, 128] // SWC-113: DoS with Failed Call, SWC-128: DoS with Block Gas Limit
    }

    fn recommendation(&self) -> &'static str {
        "Avoid external calls inside loops. Use the pull-over-push pattern: \
         let recipients withdraw funds themselves instead of pushing in a loop. \
         Bound loop iterations to a known safe limit."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-113",
            "https://swcregistry.io/docs/SWC-128",
            "https://consensys.github.io/smart-contract-best-practices/attacks/denial-of-service/",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_denial_of_service_sir_detector() {
        let detector = DenialOfServiceSirDetector::new();
        assert_eq!(detector.detector_id(), DetectorId::DenialOfService);
        assert_eq!(detector.swc_ids(), vec![113, 128]);
        assert_eq!(detector.cwe_ids(), vec![400]);
        assert_eq!(detector.risk_level(), RiskLevel::High);
        assert_eq!(detector.bug_category(), BugCategory::DenialOfService);
    }
}
