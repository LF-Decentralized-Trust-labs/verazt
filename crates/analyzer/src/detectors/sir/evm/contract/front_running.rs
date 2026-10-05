//! Front Running / Transaction Order Dependence Detector
//!
//! Detects patterns vulnerable to front-running (SWC-114):
//! 1. ERC-20 `approve` functions that set an allowance without checking its old
//!    value
//! 2. Ether payments whose amount reads a state variable that another public
//!    function writes, so the amount depends on transaction order
//! 3. Payments to the caller for a value whose hash the function requires: the
//!    value is visible to others before the transaction is mined

use std::collections::BTreeSet;

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::dialect::evm::EvmExpr;
use scirs::sir::dialect::{EvmExprExt, EvmFunctionExt};
use scirs::sir::utils::visit::{self, Visit};
use scirs::sir::{
    AssignStmt, BinOp, ContractDecl, DialectExpr, Expr, FunctionDecl, IndexAccessExpr, MemberDecl,
    Module, Stmt, VarExpr,
};

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::FrontRunning,
    bug_kind: BugKind::Vulnerability,
    confidence: ConfidenceLevel::Medium,
    cwe_ids: &[362],
    description: "Detects patterns vulnerable to transaction order dependence \
         (front-running).",
    id: DetectorId::FrontRunning,
    name: "Front Running",
    recommendation: "For ERC-20 approve: use increaseAllowance/decreaseAllowance \
         instead of approve, or require the current allowance to be zero \
         before setting a new value. For state-dependent transfers and \
         rewards for secrets: use a commit-reveal scheme or mutex to prevent \
         front-running.",
    references: &[
        "https://swcregistry.io/docs/SWC-114",
        "https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729",
    ],
    risk_level: RiskLevel::Medium,
    swc_ids: &[114],
    target: Target::Evm,
};

/// Scan detector for front-running vulnerabilities.
#[derive(Debug, Default)]
pub struct FrontRunningDetector;

impl FrontRunningDetector {
    pub fn new() -> Self {
        Self
    }
}

/// A payment of Ether.
struct Payment<'a> {
    recipient: &'a Expr,
    amount: &'a Expr,
    loc: Loc,
}

/// The payment that `expr` makes, if any: `send`, `transfer`, or a call with
/// a value.
fn payment(expr: &Expr) -> Option<Payment<'_>> {
    match expr {
        Expr::Dialect(DialectExpr::Evm(EvmExpr::Transfer(e))) => {
            Some(Payment { recipient: &e.target, amount: &e.amount, loc: e.loc.clone() })
        }
        Expr::Dialect(DialectExpr::Evm(EvmExpr::Send(e))) => {
            Some(Payment { recipient: &e.target, amount: &e.value, loc: e.loc.clone() })
        }
        Expr::Dialect(DialectExpr::Evm(EvmExpr::LowLevelCall(e))) => {
            Some(Payment { recipient: &e.target, amount: e.value.as_deref()?, loc: e.loc.clone() })
        }
        Expr::FunctionCall(call) => {
            let loc = call.span.clone().unwrap_or_default();
            match (&*call.callee, call.args.exprs().as_slice()) {
                // `addr.transfer(amount)`, `addr.send(amount)`
                (Expr::FieldAccess(fa), [amount])
                    if matches!(fa.field.as_str(), "transfer" | "send") =>
                {
                    Some(Payment { recipient: &fa.base, amount, loc })
                }
                // `addr.call.value(amount)(data)`, before Solidity 0.6
                (Expr::FunctionCall(with_value), _) => {
                    match (&*with_value.callee, with_value.args.exprs().as_slice()) {
                        (Expr::FieldAccess(value), [amount]) if value.field == "value" => {
                            match &*value.base {
                                Expr::FieldAccess(call) if call.field == "call" => {
                                    Some(Payment { recipient: &call.base, amount, loc })
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Collects the payments in the visited code.
#[derive(Default)]
struct Payments<'a> {
    found: Vec<Payment<'a>>,
}

impl<'a> Visit<'a> for Payments<'a> {
    fn visit_expr(&mut self, expr: &'a Expr) {
        if let Some(found) = payment(expr) {
            self.found.push(found);
        }
        visit::default::visit_expr(self, expr);
    }
}

/// Collects the conditions that the visited statements require.
#[derive(Default)]
struct RequiredConditions<'a> {
    found: Vec<(&'a Expr, Option<&'a Loc>)>,
}

impl<'a> Visit<'a> for RequiredConditions<'a> {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        if let Some(cond) = stmt.required_condition() {
            self.found.push((cond, stmt.span()));
        }
        visit::default::visit_stmt(self, stmt);
    }
}

/// Collects the names of the variables that the visited code reads. With
/// `whole_only`, an indexed variable is left out: `balances[msg.sender]` is
/// an entry of one user, not state shared by all.
#[derive(Default)]
struct VarReads {
    whole_only: bool,
    names: BTreeSet<String>,
}

impl<'a> Visit<'a> for VarReads {
    fn visit_var_expr(&mut self, var: &'a VarExpr) {
        self.names.insert(var.name.clone());
    }

    fn visit_index_access_expr(&mut self, expr: &'a IndexAccessExpr) {
        if !self.whole_only {
            visit::default::visit_index_access_expr(self, expr);
        }
    }
}

/// The names of the variables that `expr` reads.
fn var_reads(expr: &Expr, whole_only: bool) -> BTreeSet<String> {
    let mut reads = VarReads { whole_only, ..VarReads::default() };
    reads.visit_expr(expr);
    reads.names
}

/// Collects the storage variables of which the visited code assigns an
/// element, as `allowed[owner][spender] = value` does.
struct ElementWrites<'s> {
    storage_vars: &'s [String],
    roots: BTreeSet<String>,
}

impl<'a> Visit<'a> for ElementWrites<'_> {
    fn visit_assign_stmt(&mut self, stmt: &'a AssignStmt) {
        if let Expr::IndexAccess(_) = &stmt.lhs
            && let Some(root) = ContractDecl::storage_root(&stmt.lhs, self.storage_vars)
        {
            self.roots.insert(root.clone());
        }
        visit::default::visit_assign_stmt(self, stmt);
    }
}

/// Returns true if `func` is a constructor: `constructor`, or, before
/// Solidity 0.5, a function named after its contract.
fn is_constructor(func: &FunctionDecl, contract: &ContractDecl) -> bool {
    func.name == "constructor" || func.name == contract.name
}

/// Returns true if `cond` compares a hash of one of `params` with a value.
fn compares_hash_of(cond: &Expr, params: &BTreeSet<String>) -> bool {
    let Expr::BinOp(cmp) = cond else {
        return false;
    };
    let hashes_param = |side: &Expr| match side {
        Expr::Dialect(DialectExpr::Evm(
            EvmExpr::Keccak256(_) | EvmExpr::Sha256(_) | EvmExpr::Ripemd160(_),
        )) => !var_reads(side, false).is_disjoint(params),
        _ => false,
    };
    matches!(cmp.op, BinOp::Eq | BinOp::Ne) && (hashes_param(&cmp.lhs) || hashes_param(&cmp.rhs))
}

impl FrontRunningDetector {
    /// An ERC-20 `approve` that overwrites an allowance without requiring
    /// anything of its old value.
    fn check_approve(
        &self,
        func: &FunctionDecl,
        body: &[Stmt],
        contract: &ContractDecl,
        storage_vars: &[String],
    ) -> Option<Bug> {
        if func.name != "approve" || func.params.len() != 2 {
            return None;
        }
        let mut writes = ElementWrites { storage_vars, roots: BTreeSet::new() };
        writes.visit_stmts(body);
        let mut conditions = RequiredConditions::default();
        conditions.visit_stmts(body);
        let checks_old_value = conditions
            .found
            .iter()
            .any(|(cond, _)| !var_reads(cond, false).is_disjoint(&writes.roots));
        if writes.roots.is_empty() || checks_old_value {
            return None;
        }
        Some(META.bug(
            Some(&format!(
                "ERC-20 approve race condition in '{}.approve': \
                 allowance is set directly without checking the \
                 old value. An attacker can front-run the approval \
                 and spend both the old and new allowance.",
                contract.name
            )),
            func.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
        ))
    }

    /// Payments in `func` whose amount reads state that another public
    /// function of `functions` writes.
    fn check_order_dependence(
        &self,
        func: &FunctionDecl,
        payments: &[Payment],
        functions: &[(&FunctionDecl, &[Stmt])],
        contract: &ContractDecl,
        storage_vars: &[String],
    ) -> Vec<Bug> {
        let writer_of = |var: &String| {
            functions.iter().find(|(other, body)| {
                other.name != func.name
                    && ContractDecl::has_storage_write(body, std::slice::from_ref(var))
            })
        };
        payments
            .iter()
            .filter_map(|paid| {
                let (var, (writer, _)) = var_reads(paid.amount, true)
                    .into_iter()
                    .filter(|var| storage_vars.contains(var))
                    .find_map(|var| Some((var.clone(), writer_of(&var)?)))?;
                Some(META.bug(
                    Some(&format!(
                        "Transaction order dependence in '{}.{}': the Ether \
                         amount depends on '{}', which '{}' can change in a \
                         transaction mined first.",
                        contract.name, func.name, var, writer.name
                    )),
                    paid.loc.clone(),
                ))
            })
            .collect()
    }

    /// Required checks of a hash of a parameter in `func`, when `func` pays
    /// the caller: whoever sees the transaction can copy the parameter.
    fn check_revealed_secret(
        &self,
        func: &FunctionDecl,
        body: &[Stmt],
        payments: &[Payment],
        contract: &ContractDecl,
    ) -> Vec<Bug> {
        if !payments.iter().any(|paid| paid.recipient.is_evm_caller()) {
            return vec![];
        }
        let params: BTreeSet<String> = func.params.iter().map(|p| p.name.clone()).collect();
        let mut conditions = RequiredConditions::default();
        conditions.visit_stmts(body);
        conditions
            .found
            .into_iter()
            .filter(|(cond, _)| compares_hash_of(cond, &params))
            .map(|(_, span)| {
                META.bug(
                    Some(&format!(
                        "Front-runnable secret in '{}.{}': the caller is paid \
                         for a value whose hash matches, and the value is \
                         visible to others before the transaction is mined.",
                        contract.name, func.name
                    )),
                    span.cloned().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                )
            })
            .collect()
    }
}

impl ScanDetector for FrontRunningDetector {
    fn meta(&self) -> &'static DetectorMeta {
        &META
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Contract
    }

    fn check_contract(&self, contract: &ContractDecl, _module: &Module) -> Vec<Bug> {
        let storage_vars = contract.storage_names();
        let functions: Vec<(&FunctionDecl, &[Stmt])> = contract
            .members
            .iter()
            .filter_map(|m| match m {
                MemberDecl::Function(func)
                    if func.is_public() && !is_constructor(func, contract) =>
                {
                    Some((func, func.body.as_deref()?))
                }
                _ => None,
            })
            .collect();

        let mut bugs = Vec::new();
        for (func, body) in &functions {
            let mut payments = Payments::default();
            payments.visit_stmts(body);
            bugs.extend(self.check_approve(func, body, contract, &storage_vars));
            bugs.extend(self.check_order_dependence(
                func,
                &payments.found,
                &functions,
                contract,
                &storage_vars,
            ));
            bugs.extend(self.check_revealed_secret(func, body, &payments.found, contract));
        }
        bugs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_front_running_detector() {
        let detector = FrontRunningDetector::new();
        assert_eq!(detector.meta().id, DetectorId::FrontRunning);
        assert_eq!(detector.meta().risk_level, RiskLevel::Medium);
    }
}
