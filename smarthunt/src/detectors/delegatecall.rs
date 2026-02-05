//! Delegatecall detector.
//!
//! Detects potentially unsafe delegatecall usage.

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::passes::PassId;
use solidity::ast::{
    Block, ContractDef, ContractElem, Expr, FuncDef, Loc, SourceUnit, SourceUnitElem, Stmt,
};

/// Detector for unsafe delegatecall usage.
pub struct DelegatecallDetector;

impl DelegatecallDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DelegatecallDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for DelegatecallDetector {
    fn id(&self) -> &'static str {
        "delegatecall"
    }

    fn name(&self) -> &'static str {
        "Unsafe Delegatecall"
    }

    fn description(&self) -> &'static str {
        "Delegatecall to an address controlled by user input can allow an attacker \
         to execute arbitrary code in the context of the calling contract."
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![PassId::DataFlow]
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Critical
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::Medium
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![94] // CWE-94: Improper Control of Generation of Code
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![112] // SWC-112: Delegatecall to Untrusted Callee
    }

    fn detect(&self, context: &AnalysisContext) -> Vec<Bug> {
        let mut bugs = Vec::new();

        for source_unit in &context.source_units {
            self.visit_source_unit(source_unit, context, &mut bugs);
        }

        bugs
    }

    fn recommendation(&self) -> &'static str {
        "Avoid using delegatecall with user-controlled addresses. If necessary, \
         use a whitelist of trusted implementation contracts."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-112",
        ]
    }
}

impl DelegatecallDetector {
    fn visit_source_unit(&self, source_unit: &SourceUnit, context: &AnalysisContext, bugs: &mut Vec<Bug>) {
        for elem in &source_unit.elems {
            match elem {
                SourceUnitElem::Contract(contract) => {
                    self.visit_contract(contract, context, bugs);
                }
                SourceUnitElem::Func(func) => {
                    self.visit_function(func, context, bugs);
                }
                _ => {}
            }
        }
    }

    fn visit_contract(&self, contract: &ContractDef, context: &AnalysisContext, bugs: &mut Vec<Bug>) {
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                self.visit_function(func, context, bugs);
            }
        }
    }

    fn visit_function(&self, func: &FuncDef, context: &AnalysisContext, bugs: &mut Vec<Bug>) {
        if let Some(body) = &func.body {
            self.visit_block(body, context, bugs);
        }
    }

    fn visit_block(&self, block: &Block, context: &AnalysisContext, bugs: &mut Vec<Bug>) {
        for stmt in &block.body {
            self.visit_stmt(stmt, context, bugs);
        }
    }

    fn visit_stmt(&self, stmt: &Stmt, context: &AnalysisContext, bugs: &mut Vec<Bug>) {
        match stmt {
            Stmt::Block(block) => self.visit_block(block, context, bugs),

            Stmt::Expr(expr_stmt) => {
                self.check_expr(&expr_stmt.expr, context, bugs);
            }

            Stmt::VarDecl(var_decl) => {
                if let Some(value) = &var_decl.value {
                    self.check_expr(value, context, bugs);
                }
            }

            Stmt::If(if_stmt) => {
                self.check_expr(&if_stmt.condition, context, bugs);
                self.visit_stmt(&if_stmt.true_branch, context, bugs);
                if let Some(false_br) = &if_stmt.false_branch {
                    self.visit_stmt(false_br, context, bugs);
                }
            }

            Stmt::For(for_stmt) => {
                if let Some(pre) = &for_stmt.pre_loop {
                    self.visit_stmt(pre, context, bugs);
                }
                if let Some(cond) = &for_stmt.condition {
                    self.check_expr(cond, context, bugs);
                }
                if let Some(post) = &for_stmt.post_loop {
                    self.visit_stmt(post, context, bugs);
                }
                self.visit_stmt(&for_stmt.body, context, bugs);
            }

            Stmt::While(while_stmt) => {
                self.check_expr(&while_stmt.condition, context, bugs);
                self.visit_stmt(&while_stmt.body, context, bugs);
            }

            Stmt::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    self.check_expr(expr, context, bugs);
                }
            }

            _ => {}
        }
    }

    fn check_expr(&self, expr: &Expr, context: &AnalysisContext, bugs: &mut Vec<Bug>) {
        match expr {
            Expr::Call(call) => {
                // Check for delegatecall
                if let Expr::Member(member) = call.callee.as_ref() {
                    if member.member.base.as_str() == "delegatecall" {
                        // Check if target is user-controlled
                        if self.is_potentially_user_controlled(&member.base) {
                            let loc = call.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                            let bug = create_bug(
                                self,
                                Some("Delegatecall to a potentially user-controlled address."),
                                loc,
                            );
                            bugs.push(bug);
                        }
                    }
                }

                // Recursively check arguments
                match &call.args {
                    solidity::ast::CallArgs::Unnamed(args) => {
                        for arg in args {
                            self.check_expr(arg, context, bugs);
                        }
                    }
                    solidity::ast::CallArgs::Named(args) => {
                        for arg in args {
                            self.check_expr(&arg.value, context, bugs);
                        }
                    }
                }
            }

            Expr::CallOpts(call_opts) => {
                if let Expr::Member(member) = call_opts.callee.as_ref() {
                    if member.member.base.as_str() == "delegatecall" {
                        if self.is_potentially_user_controlled(&member.base) {
                            let loc = call_opts.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                            let bug = create_bug(
                                self,
                                Some("Delegatecall to a potentially user-controlled address."),
                                loc,
                            );
                            bugs.push(bug);
                        }
                    }
                }
            }

            Expr::Binary(binary) => {
                self.check_expr(&binary.left, context, bugs);
                self.check_expr(&binary.right, context, bugs);
            }

            Expr::Unary(unary) => {
                self.check_expr(&unary.body, context, bugs);
            }

            Expr::Assign(assign) => {
                self.check_expr(&assign.left, context, bugs);
                self.check_expr(&assign.right, context, bugs);
            }

            _ => {}
        }
    }

    fn is_potentially_user_controlled(&self, expr: &Expr) -> bool {
        match expr {
            // Function parameters are user-controlled
            Expr::Ident(ident) => {
                // Simple heuristic: non-constant identifiers could be user-controlled
                let name = ident.name.base.as_str();
                !name.starts_with("_") &&
                !matches!(name, "this" | "msg" | "block" | "tx")
            }

            // Member access on user-controlled base
            Expr::Member(member) => {
                self.is_potentially_user_controlled(&member.base)
            }

            // Index access on user-controlled array
            Expr::Index(index) => {
                self.is_potentially_user_controlled(&index.base_expr)
            }

            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegatecall_detector_new() {
        let detector = DelegatecallDetector::new();
        assert_eq!(detector.id(), "delegatecall");
    }
}
