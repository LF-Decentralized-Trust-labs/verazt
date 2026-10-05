//! Denial of Service Detector
//!
//! Detects patterns that can lead to denial of service:
//! 1. External calls inside loops (SWC-113)
//! 2. `require(addr.send(...))` pattern (SWC-113)
//! 3. Unbounded loops over dynamic storage arrays (SWC-128)
//! 4. Loops that grow a storage array (SWC-128)

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::dialect::EvmExprExt;
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::exprs::Expr;
use scirs::sir::stmts::Stmt;
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{
    AssignStmt, AugAssignStmt, CallExpr, ContractDecl, DialectExpr, ForStmt, FunctionDecl, Module,
    WhileStmt,
};

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::DenialOfService,
    bug_kind: BugKind::Vulnerability,
    confidence: ConfidenceLevel::Medium,
    cwe_ids: &[400],
    description: "Detects patterns that can lead to denial of service: external calls \
         in loops, require(send) patterns, and unbounded loops.",
    id: DetectorId::DenialOfService,
    name: "Denial of Service",
    recommendation: "Avoid external calls inside loops. Use the pull-over-push pattern: \
         let recipients withdraw funds themselves instead of pushing in a loop. \
         Bound loop iterations to a known safe limit.",
    references: &[
        "https://swcregistry.io/docs/SWC-113",
        "https://swcregistry.io/docs/SWC-128",
        "https://consensys.github.io/smart-contract-best-practices/attacks/denial-of-service/",
    ],
    risk_level: RiskLevel::High,
    swc_ids: &[113, 128],
    target: Target::Evm,
};

/// Scan detector for denial of service vulnerabilities.
#[derive(Debug, Default)]
pub struct DenialOfServiceDetector;

impl DenialOfServiceDetector {
    pub fn new() -> Self {
        Self
    }
}

/// What a loop body does that can deny service.
#[derive(Default)]
struct LoopBody<'s> {
    storage_vars: &'s [String],
    /// Whether the body makes an external call whose failure reverts the
    /// transaction: a `transfer`, or a required `send` or call. A failed
    /// call whose result is ignored lets the loop go on.
    reverting_call: bool,
    /// Whether the body grows a storage array.
    grows_storage_array: bool,
}

impl LoopBody<'_> {
    fn note_growth_of(&mut self, array: &Expr) {
        if ContractDecl::expr_references_storage(array, self.storage_vars) {
            self.grows_storage_array = true;
        }
    }
}

impl<'a> Visit<'a> for LoopBody<'_> {
    fn visit_expr(&mut self, expr: &'a Expr) {
        if is_transfer(expr) {
            self.reverting_call = true;
        }
        visit::default::visit_expr(self, expr);
    }

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        if stmt
            .required_condition()
            .is_some_and(|cond| cond.is_evm_external_call())
        {
            self.reverting_call = true;
        }
        visit::default::visit_stmt(self, stmt);
    }

    // `array.push(x)`
    fn visit_call_expr(&mut self, call: &'a CallExpr) {
        if let Expr::FieldAccess(fa) = &*call.callee
            && fa.field == "push"
        {
            self.note_growth_of(&fa.base);
        }
        visit::default::visit_call_expr(self, call);
    }

    // `array.length = n`, before Solidity 0.6
    fn visit_assign_stmt(&mut self, stmt: &'a AssignStmt) {
        if let Expr::FieldAccess(fa) = &stmt.lhs
            && fa.field == "length"
        {
            self.note_growth_of(&fa.base);
        }
        visit::default::visit_assign_stmt(self, stmt);
    }

    // `array.length += n`, before Solidity 0.6
    fn visit_aug_assign_stmt(&mut self, stmt: &'a AugAssignStmt) {
        if let Expr::FieldAccess(fa) = &stmt.lhs
            && fa.field == "length"
        {
            self.note_growth_of(&fa.base);
        }
        visit::default::visit_aug_assign_stmt(self, stmt);
    }
}

/// The denial of service risk of a loop with condition `cond` and `body`,
/// as a message about `contract_name.func_name`, if any.
fn loop_risk(
    cond: Option<&Expr>,
    body: &[Stmt],
    storage_vars: &[String],
    contract_name: &str,
    func_name: &str,
) -> Option<String> {
    let mut loop_body = LoopBody { storage_vars, ..LoopBody::default() };
    loop_body.visit_stmts(body);
    if loop_body.reverting_call {
        Some(format!(
            "External call inside loop in '{contract_name}.{func_name}'. A single \
             failed call can revert the entire transaction."
        ))
    } else if cond.is_some_and(|c| reads_storage_array_length(c, storage_vars)) {
        Some(format!(
            "Unbounded loop in '{contract_name}.{func_name}': loop bound depends on \
             dynamic array length, which could exceed the block gas limit."
        ))
    } else if loop_body.grows_storage_array {
        Some(format!(
            "Loop in '{contract_name}.{func_name}' grows a storage array: every loop \
             over the array then costs more gas, until it exceeds the block gas limit."
        ))
    } else {
        None
    }
}

/// Returns true if `expr` sends Ether with `send` or `transfer` to another
/// account than the caller. A caller whose own payment fails can only block
/// itself.
fn sends_to_another_account(expr: &Expr) -> bool {
    let recipient = match expr {
        Expr::FunctionCall(call) => match &*call.callee {
            Expr::FieldAccess(fa) if matches!(fa.field.as_str(), "send" | "transfer") => &*fa.base,
            _ => return false,
        },
        Expr::Dialect(DialectExpr::Evm(EvmExpr::Send(e))) => &*e.target,
        Expr::Dialect(DialectExpr::Evm(EvmExpr::Transfer(e))) => &*e.target,
        _ => return false,
    };
    !recipient.is_evm_caller()
}

/// Returns true if `expr` sends Ether with `transfer`, which reverts on
/// failure.
fn is_transfer(expr: &Expr) -> bool {
    match expr {
        Expr::FunctionCall(call) => {
            matches!(&*call.callee, Expr::FieldAccess(fa) if fa.field == "transfer")
        }
        Expr::Dialect(DialectExpr::Evm(EvmExpr::Transfer(_))) => true,
        _ => false,
    }
}

/// Returns true if `expr` reads the length of a storage array, which other
/// callers may grow. A caller passing a long array only spends its own gas.
fn reads_storage_array_length(expr: &Expr, storage_vars: &[String]) -> bool {
    match expr {
        Expr::FieldAccess(fa) if fa.field == "length" => {
            ContractDecl::expr_references_storage(&fa.base, storage_vars)
        }
        Expr::BinOp(bin) => {
            reads_storage_array_length(&bin.lhs, storage_vars)
                || reads_storage_array_length(&bin.rhs, storage_vars)
        }
        _ => false,
    }
}

impl ScanDetector for DenialOfServiceDetector {
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
            storage_vars: Vec<String>,
            contract_name: String,
            func_name: String,
            in_loop: bool,
        }

        impl Visitor<'_> {
            /// Report the loop at `span` if it risks denial of service, then
            /// visit its body.
            fn check_loop(&mut self, cond: Option<&Expr>, body: &[Stmt], span: Option<&Loc>) {
                let risk = loop_risk(
                    cond,
                    body,
                    &self.storage_vars,
                    &self.contract_name,
                    &self.func_name,
                );
                if let Some(message) = risk {
                    let loc = span.cloned().unwrap_or_else(|| Loc::new(0, 0, 0, 0));
                    self.bugs.push(META.bug(Some(&message), loc));
                }
            }

            /// Report a required `send` or `transfer` at `span`. In a loop,
            /// the external call already makes the loop a finding.
            fn check_required(&mut self, cond: &Expr, span: Option<&Loc>) {
                if !self.in_loop && sends_to_another_account(cond) {
                    self.bugs.push(META.bug(
                        Some(&format!(
                            "require(send/transfer) in '{}.{}': a single \
                             failed send reverts the entire transaction, \
                             enabling DoS by a malicious recipient.",
                            self.contract_name, self.func_name
                        )),
                        span.cloned().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                    ));
                }
            }
        }

        impl<'a> Visit<'a> for Visitor<'_> {
            fn visit_for_stmt(&mut self, stmt: &'a ForStmt) {
                self.check_loop(stmt.cond.as_ref(), &stmt.body, stmt.span.as_ref());
                let was_in_loop = self.in_loop;
                self.in_loop = true;
                visit::default::visit_for_stmt(self, stmt);
                self.in_loop = was_in_loop;
            }

            fn visit_while_stmt(&mut self, stmt: &'a WhileStmt) {
                self.check_loop(None, &stmt.body, stmt.span.as_ref());
                let was_in_loop = self.in_loop;
                self.in_loop = true;
                visit::default::visit_while_stmt(self, stmt);
                self.in_loop = was_in_loop;
            }

            fn visit_stmt(&mut self, stmt: &'a Stmt) {
                if let Some(cond) = stmt.required_condition() {
                    self.check_required(cond, stmt.span());
                }
                visit::default::visit_stmt(self, stmt);
            }
        }

        let mut visitor = Visitor {
            bugs: &mut bugs,
            storage_vars: contract.storage_names(),
            contract_name: contract.name.clone(),
            func_name: func.name.clone(),
            in_loop: false,
        };
        visitor.visit_function_decl(func);

        bugs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_denial_of_service_detector() {
        let detector = DenialOfServiceDetector::new();
        assert_eq!(detector.meta().id, DetectorId::DenialOfService);
        assert_eq!(detector.meta().risk_level, RiskLevel::High);
    }
}
