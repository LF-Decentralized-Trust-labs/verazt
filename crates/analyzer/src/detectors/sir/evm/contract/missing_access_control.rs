//! Missing Access Control Detector
//!
//! Detects public functions that let any caller write an authorization
//! variable: a state variable that the contract compares with `msg.sender`
//! or `tx.origin` elsewhere, such as `owner`.

use std::collections::HashSet;

use crate::detectors::sir::detector::{DetectionLevel, ScanDetector};
use crate::detectors::{ConfidenceLevel, DetectorId, DetectorMeta, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::dialect::evm::{EvmMemberDecl, EvmModifierDef};
use scirs::sir::dialect::{EvmExprExt, EvmFunctionExt};
use scirs::sir::utils::visit::{Visit, default};
use scirs::sir::{
    AssignStmt, AugAssignStmt, BinOp, BinOpExpr, ContractDecl, DialectMemberDecl, Expr,
    FunctionDecl, MemberDecl, Module, Stmt,
};

const META: DetectorMeta = DetectorMeta {
    bug_category: BugCategory::AccessControl,
    bug_kind: BugKind::Vulnerability,
    confidence: ConfidenceLevel::Medium,
    cwe_ids: &[284],
    description: "Detects public functions that let any caller modify an authorization \
         variable, such as the contract owner",
    id: DetectorId::MissingAccessControl,
    name: "Missing Access Control",
    recommendation: "Add access control modifiers (e.g., `onlyOwner` or OpenZeppelin's \
         `AccessControl` with role-based checks) to functions that modify \
         sensitive state. Use `Ownable2Step` for ownership to prevent \
         accidental transfers.",
    references: &[
        "https://swcregistry.io/docs/SWC-105",
        "https://swcregistry.io/docs/SWC-106",
    ],
    risk_level: RiskLevel::High,
    swc_ids: &[105, 106],
    target: Target::Evm,
};

/// Scan detector for public functions that write authorization variables
/// without checking the caller.
#[derive(Debug, Default)]
pub struct MissingAccessControlDetector;

impl MissingAccessControlDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ScanDetector for MissingAccessControlDetector {
    fn meta(&self) -> &'static DetectorMeta {
        &META
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Contract
    }

    fn check_contract(&self, contract: &ContractDecl, _module: &Module) -> Vec<Bug> {
        let storage_vars = contract.storage_names();
        let functions: Vec<&FunctionDecl> = contract
            .members
            .iter()
            .filter_map(|m| match m {
                MemberDecl::Function(func) => Some(func),
                _ => None,
            })
            .collect();
        let modifiers: Vec<&EvmModifierDef> = contract
            .members
            .iter()
            .filter_map(|m| match m {
                MemberDecl::Dialect(DialectMemberDecl::Evm(EvmMemberDecl::ModifierDef(m))) => {
                    Some(m)
                }
                _ => None,
            })
            .collect();

        // Authorization variables are the ones compared with the caller
        // anywhere in the contract.
        let mut checks = CallerChecks::new(&storage_vars);
        for modifier in &modifiers {
            checks.visit_stmts(&modifier.body);
        }
        for func in &functions {
            if let Some(body) = &func.body {
                checks.visit_stmts(body);
            }
        }
        let auth_vars = checks.compared_vars;
        if auth_vars.is_empty() {
            return vec![];
        }

        functions
            .into_iter()
            .filter(|func| func.is_public() && !is_constructor(func, contract))
            .filter(|func| !is_guarded(func, &modifiers, &storage_vars))
            .filter_map(|func| {
                let written = written_auth_var(func, &storage_vars, &auth_vars)?;
                Some(META.bug(
                    Some(&format!(
                        "Function '{}' in '{}' lets any caller modify the \
                         authorization variable '{}'.",
                        func.name, contract.name, written,
                    )),
                    func.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                ))
            })
            .collect()
    }
}

/// Returns true if `func` is a constructor: `constructor`, or, before
/// Solidity 0.5, a function named after its contract.
fn is_constructor(func: &FunctionDecl, contract: &ContractDecl) -> bool {
    func.name == "constructor" || func.name == contract.name
}

/// Returns true if `func` checks the caller, in its body or in one of its
/// modifiers. A modifier defined outside the contract, as by a parent
/// contract, counts as a check.
fn is_guarded(func: &FunctionDecl, modifiers: &[&EvmModifierDef], storage_vars: &[String]) -> bool {
    let checks_caller = |stmts: &[Stmt]| {
        let mut checks = CallerChecks::new(storage_vars);
        checks.visit_stmts(stmts);
        checks.checks_caller
    };
    func.body.as_deref().is_some_and(checks_caller)
        || func.modifier_invocs.iter().any(|invoc| {
            modifiers
                .iter()
                .find(|m| m.name == invoc.name)
                .is_none_or(|m| checks_caller(&m.body))
        })
}

/// The first authorization variable that `func` writes, if any.
fn written_auth_var(
    func: &FunctionDecl,
    storage_vars: &[String],
    auth_vars: &HashSet<String>,
) -> Option<String> {
    let mut writes = StorageWrites { storage_vars, written: vec![] };
    writes.visit_stmts(func.body.as_deref()?);
    writes.written.into_iter().find(|var| auth_vars.contains(var))
}

/// Collects the equality comparisons with the caller in the visited code.
struct CallerChecks<'s> {
    storage_vars: &'s [String],
    /// Whether the code compares anything with the caller.
    checks_caller: bool,
    /// Storage variables compared, as a whole, with the caller. An element
    /// compared with the caller (`channels[id].party == msg.sender`) guards
    /// that element only, so its variable is left out.
    compared_vars: HashSet<String>,
}

impl<'s> CallerChecks<'s> {
    fn new(storage_vars: &'s [String]) -> Self {
        Self { storage_vars, checks_caller: false, compared_vars: HashSet::new() }
    }
}

impl<'a> Visit<'a> for CallerChecks<'_> {
    fn visit_binop_expr(&mut self, expr: &'a BinOpExpr) {
        if matches!(expr.op, BinOp::Eq | BinOp::Ne) && (expr.lhs.is_evm_caller() || expr.rhs.is_evm_caller())
        {
            self.checks_caller = true;
            for side in [&expr.lhs, &expr.rhs] {
                if let Expr::Var(v) = &**side
                    && self.storage_vars.contains(&v.name)
                {
                    self.compared_vars.insert(v.name.clone());
                }
            }
        }
        default::visit_binop_expr(self, expr)
    }
}

/// Collects the storage variables written in the visited code.
struct StorageWrites<'s> {
    storage_vars: &'s [String],
    written: Vec<String>,
}

impl<'a> Visit<'a> for StorageWrites<'_> {
    fn visit_assign_stmt(&mut self, stmt: &'a AssignStmt) {
        if let Some(var) = ContractDecl::storage_root(&stmt.lhs, self.storage_vars) {
            self.written.push(var.clone());
        }
        default::visit_assign_stmt(self, stmt)
    }

    fn visit_aug_assign_stmt(&mut self, stmt: &'a AugAssignStmt) {
        if let Some(var) = ContractDecl::storage_root(&stmt.lhs, self.storage_vars) {
            self.written.push(var.clone());
        }
        default::visit_aug_assign_stmt(self, stmt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_access_control_detector() {
        let detector = MissingAccessControlDetector::new();
        assert_eq!(detector.meta().id, DetectorId::MissingAccessControl);
        assert_eq!(detector.meta().risk_level, RiskLevel::High);
    }
}
