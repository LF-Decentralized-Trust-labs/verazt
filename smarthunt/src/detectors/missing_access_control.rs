//! Missing Access Control detector.
//!
//! Detects public/external functions that modify state but lack access control.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::graph::FunctionId;
use crate::passes::PassId;
use solidity::ast::{
    Block, ContractDef, ContractElem, Expr, FuncDef, FuncVis, Loc, SourceUnit, SourceUnitElem, Stmt,
};

/// Detector for missing access control.
pub struct MissingAccessControlDetector;

impl MissingAccessControlDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MissingAccessControlDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for MissingAccessControlDetector {
    fn id(&self) -> &'static str {
        "missing-access-control"
    }

    fn name(&self) -> &'static str {
        "Missing Access Control"
    }

    fn description(&self) -> &'static str {
        "Public or external functions that modify sensitive state should have \
         access control modifiers to prevent unauthorized access."
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable, PassId::StateMutation, PassId::AccessControl]
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::High
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![284] // CWE-284: Improper Access Control
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![105, 106] // SWC-105: Unprotected Ether Withdrawal, SWC-106: Unprotected SELFDESTRUCT
    }

    fn detect(&self, context: &AnalysisContext) -> Vec<Bug> {
        let mut bugs = Vec::new();

        for source_unit in &context.source_units {
            self.visit_source_unit(source_unit, context, &mut bugs);
        }

        bugs
    }

    fn recommendation(&self) -> &'static str {
        "Add access control modifiers (e.g., onlyOwner, onlyRole) to functions \
         that modify sensitive state or perform privileged operations."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-105",
            "https://swcregistry.io/docs/SWC-106",
        ]
    }
}

impl MissingAccessControlDetector {
    // Sensitive operation keywords
    const SENSITIVE_OPERATIONS: &'static [&'static str] = &[
        "withdraw", "transfer", "send", "selfdestruct", "suicide",
        "setOwner", "changeOwner", "transferOwnership",
        "mint", "burn", "pause", "unpause",
        "upgrade", "setImplementation", "setAdmin",
        "setFee", "setPrice", "setRate",
        "initialize", "init",
    ];

    // Common access control modifier names
    const ACCESS_CONTROL_MODIFIERS: &'static [&'static str] = &[
        "onlyOwner", "onlyAdmin", "onlyRole", "onlyMinter",
        "onlyPauser", "onlyGovernance", "onlyController",
        "onlyAuthorized", "whenNotPaused", "nonReentrant",
        "initializer", "onlyProxy", "onlyDelegateCall",
    ];

    fn visit_source_unit(&self, source_unit: &SourceUnit, context: &AnalysisContext, bugs: &mut Vec<Bug>) {
        for elem in &source_unit.elems {
            if let SourceUnitElem::Contract(contract) = elem {
                self.visit_contract(contract, context, bugs);
            }
        }
    }

    fn visit_contract(&self, contract: &ContractDef, context: &AnalysisContext, bugs: &mut Vec<Bug>) {
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                self.check_function(func, contract, context, bugs);
            }
        }
    }

    fn check_function(
        &self,
        func: &FuncDef,
        contract: &ContractDef,
        context: &AnalysisContext,
        bugs: &mut Vec<Bug>,
    ) {
        // Only check public/external functions
        let is_externally_callable = matches!(
            func.visibility,
            FuncVis::Public | FuncVis::External | FuncVis::None
        );

        if !is_externally_callable {
            return;
        }

        // Skip constructors and receive/fallback functions
        if matches!(func.kind, solidity::ast::FuncKind::Constructor | solidity::ast::FuncKind::Receive | solidity::ast::FuncKind::Fallback) {
            return;
        }

        // Check if function has access control modifier
        let has_access_control = func.modifier_invocs.iter().any(|m| {
            if let Expr::Ident(ident) = m.callee.as_ref() {
                Self::ACCESS_CONTROL_MODIFIERS.contains(&ident.name.base.as_str())
            } else if let Expr::Call(call) = m.callee.as_ref() {
                if let Expr::Ident(ident) = call.callee.as_ref() {
                    Self::ACCESS_CONTROL_MODIFIERS.contains(&ident.name.base.as_str())
                } else {
                    false
                }
            } else {
                false
            }
        });

        if has_access_control {
            return;
        }

        // Check if function body contains msg.sender check
        let has_sender_check = func.body.as_ref()
            .map(|body| self.has_msg_sender_check(body))
            .unwrap_or(false);

        if has_sender_check {
            return;
        }

        // Check if function name suggests it's sensitive
        let func_name = func.name.base.as_str().to_lowercase();
        let is_sensitive_name = Self::SENSITIVE_OPERATIONS.iter()
            .any(|op| func_name.contains(&op.to_lowercase()));

        // Check if function modifies state
        let func_id = FunctionId::from_func(func, Some(contract));
        let modifies_state = context.modifies_state(&func_id);

        // Check for sensitive operations in body
        let has_sensitive_ops = func.body.as_ref()
            .map(|body| self.has_sensitive_operation(body))
            .unwrap_or(false);

        if (is_sensitive_name || has_sensitive_ops) && !has_access_control {
            let loc = func.loc.unwrap_or(Loc::new(1, 1, 1, 1));
            let bug = create_bug(
                self,
                Some(&format!(
                    "Function '{}' performs sensitive operations but lacks access control.",
                    func.name.base.as_str()
                )),
                loc,
            );
            bugs.push(bug);
        }
    }

    fn has_msg_sender_check(&self, block: &Block) -> bool {
        for stmt in &block.body {
            if self.stmt_has_sender_check(stmt) {
                return true;
            }
        }
        false
    }

    fn stmt_has_sender_check(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::If(if_stmt) => {
                if self.expr_uses_msg_sender(&if_stmt.condition) {
                    return true;
                }
                if self.stmt_has_sender_check(&if_stmt.true_branch) {
                    return true;
                }
                if let Some(false_br) = &if_stmt.false_branch {
                    if self.stmt_has_sender_check(false_br) {
                        return true;
                    }
                }
            }
            Stmt::Block(block) => {
                return self.has_msg_sender_check(block);
            }
            Stmt::Expr(expr_stmt) => {
                if let Expr::Call(call) = &expr_stmt.expr {
                    if let Expr::Ident(ident) = call.callee.as_ref() {
                        if matches!(ident.name.base.as_str(), "require" | "assert") {
                            if let solidity::ast::CallArgs::Unnamed(args) = &call.args {
                                if let Some(first) = args.first() {
                                    if self.expr_uses_msg_sender(first) {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn expr_uses_msg_sender(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Member(member) => {
                if let Expr::Ident(base) = member.base.as_ref() {
                    if base.name.base.as_str() == "msg" && member.member.base.as_str() == "sender" {
                        return true;
                    }
                }
                self.expr_uses_msg_sender(&member.base)
            }
            Expr::Binary(bin) => {
                self.expr_uses_msg_sender(&bin.left) || self.expr_uses_msg_sender(&bin.right)
            }
            Expr::Unary(unary) => self.expr_uses_msg_sender(&unary.body),
            Expr::Call(call) => {
                self.expr_uses_msg_sender(&call.callee) ||
                    match &call.args {
                        solidity::ast::CallArgs::Unnamed(args) => {
                            args.iter().any(|a| self.expr_uses_msg_sender(a))
                        }
                        solidity::ast::CallArgs::Named(args) => {
                            args.iter().any(|a| self.expr_uses_msg_sender(&a.value))
                        }
                    }
            }
            _ => false,
        }
    }

    fn has_sensitive_operation(&self, block: &Block) -> bool {
        for stmt in &block.body {
            if self.stmt_has_sensitive_op(stmt) {
                return true;
            }
        }
        false
    }

    fn stmt_has_sensitive_op(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(expr_stmt) => self.expr_has_sensitive_op(&expr_stmt.expr),
            Stmt::Block(block) => self.has_sensitive_operation(block),
            Stmt::If(if_stmt) => {
                self.stmt_has_sensitive_op(&if_stmt.true_branch) ||
                    if_stmt.false_branch.as_ref()
                        .map(|b| self.stmt_has_sensitive_op(b))
                        .unwrap_or(false)
            }
            Stmt::For(for_stmt) => self.stmt_has_sensitive_op(&for_stmt.body),
            Stmt::While(while_stmt) => self.stmt_has_sensitive_op(&while_stmt.body),
            _ => false,
        }
    }

    fn expr_has_sensitive_op(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call(call) => {
                // Check for selfdestruct
                if let Expr::Ident(ident) = call.callee.as_ref() {
                    if matches!(ident.name.base.as_str(), "selfdestruct" | "suicide") {
                        return true;
                    }
                }
                // Check for transfer/send
                if let Expr::Member(member) = call.callee.as_ref() {
                    if matches!(member.member.base.as_str(), "transfer" | "send" | "call") {
                        return true;
                    }
                }
                false
            }
            Expr::CallOpts(call_opts) => {
                if let Expr::Member(member) = call_opts.callee.as_ref() {
                    if matches!(member.member.base.as_str(), "transfer" | "send" | "call") {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_access_control_detector_new() {
        let detector = MissingAccessControlDetector::new();
        assert_eq!(detector.id(), "missing-access-control");
    }
}
