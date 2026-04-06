//! Front Running / Transaction Order Dependence Detector
//!
//! Detects patterns vulnerable to front-running:
//! 1. ERC-20 `approve` functions that directly set allowance without checking
//!    the old value (SWC-114)
//! 2. State-dependent ETH transfers where another public function can modify
//!    the state variable

use crate::detector::{Confidence, DetectionLevel, ScanDetector, Target};
use bugs::bug::{Bug, BugCategory, BugKind, RiskLevel};
use common::loc::Loc;
use scirs::sir::dialect::EvmFunctionExt;
use scirs::sir::exprs::Expr;
use scirs::sir::stmts::Stmt;
use scirs::sir::{ContractDecl, FunctionDecl, MemberDecl, Module};

/// Scan detector for front-running vulnerabilities.
#[derive(Debug, Default)]
pub struct FrontRunningDetector;

impl FrontRunningDetector {
    pub fn new() -> Self {
        Self
    }
}

/// Check if a function is an `approve` function (ERC-20 pattern).
fn is_approve_function(func: &FunctionDecl) -> bool {
    func.name == "approve" && func.params.len() == 2 && func.is_public()
}

/// Check if a function body directly sets a mapping value without
/// requiring the old value to be zero first.
fn has_direct_allowance_set(body: &[Stmt]) -> bool {
    for stmt in body {
        match stmt {
            Stmt::Assign(a) => {
                if is_mapping_access(&a.lhs) {
                    return true;
                }
            }
            Stmt::Block(stmts) => {
                if has_direct_allowance_set(stmts) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Check if a function body contains a require that checks the old
/// allowance value.
fn has_allowance_check(body: &[Stmt]) -> bool {
    for stmt in body {
        match stmt {
            Stmt::Assert(_) => return true,
            Stmt::If(_) => return true,
            Stmt::Block(stmts) => {
                if has_allowance_check(stmts) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Check if an expression is a mapping index access.
fn is_mapping_access(expr: &Expr) -> bool {
    matches!(expr, Expr::IndexAccess(_))
}

/// Check if a function body contains an ETH transfer.
fn contains_eth_transfer(stmts: &[Stmt]) -> bool {
    for stmt in stmts {
        match stmt {
            Stmt::Expr(es) => {
                if expr_is_eth_transfer(&es.expr) {
                    return true;
                }
            }
            Stmt::If(s) => {
                if contains_eth_transfer(&s.then_body) {
                    return true;
                }
                if let Some(else_body) = &s.else_body {
                    if contains_eth_transfer(else_body) {
                        return true;
                    }
                }
            }
            Stmt::Block(stmts) => {
                if contains_eth_transfer(stmts) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Check if an expression is an ETH transfer.
fn expr_is_eth_transfer(expr: &Expr) -> bool {
    match expr {
        Expr::FunctionCall(call) => {
            if let Expr::FieldAccess(fa) = &*call.callee {
                return matches!(fa.field.as_str(), "send" | "transfer" | "call");
            }
            if let Expr::Dialect(scirs::sir::DialectExpr::Evm(evm)) = &*call.callee {
                return matches!(
                    evm,
                    scirs::sir::dialect::evm::EvmExpr::Send(_)
                        | scirs::sir::dialect::evm::EvmExpr::Transfer(_)
                        | scirs::sir::dialect::evm::EvmExpr::LowLevelCall(_)
                        | scirs::sir::dialect::evm::EvmExpr::RawCall(_)
                );
            }
            false
        }
        _ => false,
    }
}

/// Collect storage variable names referenced in ETH transfer amounts.
fn collect_transfer_storage_deps(stmts: &[Stmt], storage_vars: &[String]) -> Vec<String> {
    let mut deps = Vec::new();
    for stmt in stmts {
        collect_transfer_storage_deps_inner(stmt, storage_vars, &mut deps);
    }
    deps
}

fn collect_transfer_storage_deps_inner(
    stmt: &Stmt,
    storage_vars: &[String],
    deps: &mut Vec<String>,
) {
    match stmt {
        Stmt::Expr(es) => {
            if let Expr::FunctionCall(call) = &es.expr {
                for arg in call.args.exprs() {
                    collect_storage_refs(arg, storage_vars, deps);
                }
            }
        }
        Stmt::If(s) => {
            for st in &s.then_body {
                collect_transfer_storage_deps_inner(st, storage_vars, deps);
            }
            if let Some(else_body) = &s.else_body {
                for st in else_body {
                    collect_transfer_storage_deps_inner(st, storage_vars, deps);
                }
            }
        }
        Stmt::Block(stmts) => {
            for st in stmts {
                collect_transfer_storage_deps_inner(st, storage_vars, deps);
            }
        }
        _ => {}
    }
}

fn collect_storage_refs(expr: &Expr, storage_vars: &[String], deps: &mut Vec<String>) {
    match expr {
        Expr::Var(v) => {
            if storage_vars.contains(&v.name) && !deps.contains(&v.name) {
                deps.push(v.name.clone());
            }
        }
        Expr::BinOp(bin) => {
            collect_storage_refs(&bin.lhs, storage_vars, deps);
            collect_storage_refs(&bin.rhs, storage_vars, deps);
        }
        Expr::IndexAccess(ia) => {
            collect_storage_refs(&ia.base, storage_vars, deps);
            if let Some(idx) = &ia.index {
                collect_storage_refs(idx, storage_vars, deps);
            }
        }
        Expr::FieldAccess(fa) => collect_storage_refs(&fa.base, storage_vars, deps),
        _ => {}
    }
}

/// Check if a function body writes to a specific storage variable.
fn writes_to_storage(stmts: &[Stmt], var_name: &str) -> bool {
    for stmt in stmts {
        match stmt {
            Stmt::Assign(a) => {
                if expr_references_var(&a.lhs, var_name) {
                    return true;
                }
            }
            Stmt::AugAssign(a) => {
                if expr_references_var(&a.lhs, var_name) {
                    return true;
                }
            }
            Stmt::If(s) => {
                if writes_to_storage(&s.then_body, var_name) {
                    return true;
                }
                if let Some(else_body) = &s.else_body {
                    if writes_to_storage(else_body, var_name) {
                        return true;
                    }
                }
            }
            Stmt::Block(stmts) => {
                if writes_to_storage(stmts, var_name) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn expr_references_var(expr: &Expr, var_name: &str) -> bool {
    match expr {
        Expr::Var(v) => v.name == var_name,
        Expr::IndexAccess(ia) => expr_references_var(&ia.base, var_name),
        Expr::FieldAccess(fa) => expr_references_var(&fa.base, var_name),
        _ => false,
    }
}

#[derive(Clone)]
struct FuncInfo {
    name: String,
    has_transfer: bool,
    transfer_deps: Vec<String>,
    writes: Vec<String>,
    span: Option<Loc>,
}

impl ScanDetector for FrontRunningDetector {
    fn id(&self) -> &'static str {
        "front-running"
    }

    fn name(&self) -> &'static str {
        "Front Running"
    }

    fn description(&self) -> &'static str {
        "Detects patterns vulnerable to transaction order dependence \
         (front-running)."
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn bug_category(&self) -> BugCategory {
        BugCategory::FrontRunning
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Medium
    }

    fn confidence(&self) -> Confidence {
        Confidence::Medium
    }

    fn target(&self) -> Target {
        Target::Evm
    }

    fn level(&self) -> DetectionLevel {
        DetectionLevel::Contract
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![362]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![114]
    }

    fn recommendation(&self) -> &'static str {
        "For ERC-20 approve: use increaseAllowance/decreaseAllowance \
         instead of approve, or require the current allowance to be zero \
         before setting a new value. For state-dependent transfers: use a \
         commit-reveal scheme or mutex to prevent front-running."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-114",
            "https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729",
        ]
    }

    fn check_contract(&self, contract: &ContractDecl, _module: &Module) -> Vec<Bug> {
        let mut bugs = Vec::new();
        let storage_vars = contract.storage_names();

        // Collect function info
        let mut public_functions = Vec::new();
        for member in &contract.members {
            if let MemberDecl::Function(func) = member {
                if func.is_public() {
                    if let Some(body) = &func.body {
                        let has_transfer = contains_eth_transfer(body);
                        let transfer_deps = if has_transfer {
                            collect_transfer_storage_deps(body, &storage_vars)
                        } else {
                            vec![]
                        };
                        let mut writes = Vec::new();
                        for sv in &storage_vars {
                            if writes_to_storage(body, sv) {
                                writes.push(sv.clone());
                            }
                        }
                        public_functions.push(FuncInfo {
                            name: func.name.clone(),
                            has_transfer,
                            transfer_deps,
                            writes,
                            span: func.span.clone(),
                        });
                    }
                }

                // Sub-pattern 1: ERC-20 approve race condition
                if is_approve_function(func) {
                    if let Some(body) = &func.body {
                        if has_direct_allowance_set(body) && !has_allowance_check(body) {
                            bugs.push(Bug::new(
                                self.name(),
                                Some(&format!(
                                    "ERC-20 approve race condition in '{}.approve': \
                                     allowance is set directly without checking the \
                                     old value. An attacker can front-run the approval \
                                     and spend both the old and new allowance.",
                                    contract.name
                                )),
                                func.span.clone().unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                                self.bug_kind(),
                                self.bug_category(),
                                self.risk_level(),
                                self.cwe_ids(),
                                self.swc_ids(),
                                Some(self.recommendation()),
                            ));
                        }
                    }
                }
            }
        }

        // Sub-pattern 2: State-dependent ETH transfer
        for func_info in &public_functions {
            if func_info.has_transfer && !func_info.transfer_deps.is_empty() {
                for dep_var in &func_info.transfer_deps {
                    for other in &public_functions {
                        if other.name != func_info.name && other.writes.contains(dep_var) {
                            bugs.push(Bug::new(
                                self.name(),
                                Some(&format!(
                                    "Transaction order dependence in \
                                     '{}.{}': ETH transfer amount \
                                     depends on '{}', which can be \
                                     modified by '{}'.",
                                    contract.name,
                                    func_info.name,
                                    dep_var,
                                    other.name
                                )),
                                func_info
                                    .span
                                    .clone()
                                    .unwrap_or_else(|| Loc::new(0, 0, 0, 0)),
                                self.bug_kind(),
                                self.bug_category(),
                                self.risk_level(),
                                self.cwe_ids(),
                                self.swc_ids(),
                                Some(self.recommendation()),
                            ));
                            break;
                        }
                    }
                }
            }
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
        assert_eq!(detector.id(), "front-running");
        assert_eq!(detector.swc_ids(), vec![114]);
        assert_eq!(detector.cwe_ids(), vec![362]);
        assert_eq!(detector.risk_level(), RiskLevel::Medium);
        assert_eq!(detector.bug_category(), BugCategory::FrontRunning);
    }
}
