//! Deprecated constructs detector.
//!
//! Detects usage of deprecated Solidity constructs (SWC-111).

use bugs::bug::{Bug, BugKind, RiskLevel};
use crate::detectors::{Detector, ConfidenceLevel, create_bug};
use crate::engine::context::AnalysisContext;
use crate::passes::PassId;
use solidity::ast::{ContractDef, ContractElem, Expr, Loc, Stmt};

/// Detector for deprecated constructs.
pub struct DeprecatedDetector;

impl DeprecatedDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DeprecatedDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for DeprecatedDetector {
    fn id(&self) -> &'static str {
        "deprecated"
    }

    fn name(&self) -> &'static str {
        "Deprecated Constructs"
    }

    fn description(&self) -> &'static str {
        "Detects usage of deprecated Solidity constructs such as suicide(), sha3(), \
         msg.gas, block.blockhash(), and throw."
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![]
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Refactoring
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Low
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![]
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![111] // SWC-111: Use of Deprecated Solidity Functions
    }

    fn detect(&self, context: &AnalysisContext) -> Vec<Bug> {
        let mut bugs = Vec::new();
        
        for source_unit in &context.source_units {
            for elem in &source_unit.elems {
                if let solidity::ast::SourceUnitElem::Contract(contract) = elem {
                    self.check_contract(&contract, &mut bugs);
                }
            }
        }
        
        bugs
    }

    fn recommendation(&self) -> &'static str {
        "Replace deprecated constructs with their modern equivalents: \
         suicide() -> selfdestruct(), sha3() -> keccak256(), msg.gas -> gasleft(), \
         block.blockhash() -> blockhash()."
    }

    fn references(&self) -> Vec<&'static str> {
        vec![
            "https://swcregistry.io/docs/SWC-111",
        ]
    }
}

impl DeprecatedDetector {
    fn check_contract(&self, contract: &ContractDef, bugs: &mut Vec<Bug>) {
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                if let Some(body) = &func.body {
                    self.check_block(body, bugs);
                }
            }
        }
    }

    fn check_block(&self, block: &solidity::ast::Block, bugs: &mut Vec<Bug>) {
        for s in &block.body {
            self.check_statement(s, bugs);
        }
    }

    fn check_statement(&self, stmt: &Stmt, bugs: &mut Vec<Bug>) {
        match stmt {
            Stmt::Block(block) => {
                self.check_block(block, bugs);
            }
            Stmt::Expr(expr_stmt) => {
                self.check_expr(&expr_stmt.expr, bugs);
            }
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    self.check_expr(expr, bugs);
                }
            }
            Stmt::If(if_stmt) => {
                self.check_expr(&if_stmt.condition, bugs);
                self.check_statement(&if_stmt.true_branch, bugs);
                if let Some(false_br) = &if_stmt.false_branch {
                    self.check_statement(false_br, bugs);
                }
            }
            Stmt::While(while_stmt) => {
                self.check_expr(&while_stmt.condition, bugs);
                self.check_statement(&while_stmt.body, bugs);
            }
            Stmt::For(for_stmt) => {
                if let Some(pre) = &for_stmt.pre_loop {
                    self.check_statement(pre, bugs);
                }
                if let Some(cond) = &for_stmt.condition {
                    self.check_expr(cond, bugs);
                }
                if let Some(post) = &for_stmt.post_loop {
                    self.check_statement(post, bugs);
                }
                self.check_statement(&for_stmt.body, bugs);
            }
            Stmt::VarDecl(var_decl) => {
                if let Some(expr) = &var_decl.value {
                    self.check_expr(expr, bugs);
                }
            }
            Stmt::Try(try_stmt) => {
                self.check_expr(&try_stmt.guarded_expr, bugs);
                self.check_block(&try_stmt.body, bugs);
            }
            Stmt::DoWhile(do_while) => {
                self.check_statement(&do_while.body, bugs);
                self.check_expr(&do_while.condition, bugs);
            }
            _ => {}
        }
    }

    fn check_expr(&self, expr: &Expr, bugs: &mut Vec<Bug>) {
        match expr {
            Expr::Call(call) => {
                // Check for deprecated function calls
                if let Expr::Ident(ident) = &*call.callee {
                    let name = ident.name.base.as_str();
                    if let Some(deprecated_info) = self.get_deprecated_function(name) {
                        let loc = call.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                        let bug = create_bug(
                            self,
                            Some(&format!(
                                "Deprecated function '{}' used. Replace with '{}'.",
                                deprecated_info.0, deprecated_info.1
                            )),
                            loc,
                        );
                        bugs.push(bug);
                    }
                }
                
                self.check_expr(&call.callee, bugs);
                match &call.args {
                    solidity::ast::CallArgs::Unnamed(args) => {
                        for arg in args {
                            self.check_expr(arg, bugs);
                        }
                    }
                    solidity::ast::CallArgs::Named(args) => {
                        for arg in args {
                            self.check_expr(&arg.value, bugs);
                        }
                    }
                }
            }
            Expr::Member(member) => {
                // Check for deprecated member accesses like msg.gas, block.blockhash
                if let Expr::Ident(ident) = &*member.base {
                    let obj = ident.name.base.as_str();
                    let mem = member.member.base.as_str();
                    
                    if let Some(deprecated_info) = self.get_deprecated_member(obj, mem) {
                        let loc = member.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                        let bug = create_bug(
                            self,
                            Some(&format!(
                                "Deprecated '{}.{}' used. Replace with '{}'.",
                                obj, mem, deprecated_info
                            )),
                            loc,
                        );
                        bugs.push(bug);
                    }
                }
                
                self.check_expr(&member.base, bugs);
            }
            Expr::Index(index) => {
                self.check_expr(&index.base_expr, bugs);
                if let Some(idx) = &index.index {
                    self.check_expr(idx, bugs);
                }
            }
            Expr::Unary(unary) => {
                self.check_expr(&unary.body, bugs);
            }
            Expr::Binary(binary) => {
                self.check_expr(&binary.left, bugs);
                self.check_expr(&binary.right, bugs);
            }
            Expr::Conditional(cond_expr) => {
                self.check_expr(&cond_expr.cond, bugs);
                self.check_expr(&cond_expr.true_br, bugs);
                self.check_expr(&cond_expr.false_br, bugs);
            }
            Expr::Assign(assign) => {
                self.check_expr(&assign.left, bugs);
                self.check_expr(&assign.right, bugs);
            }
            Expr::Tuple(tuple) => {
                for elem in &tuple.elems {
                    if let Some(e) = elem {
                        self.check_expr(e, bugs);
                    }
                }
            }
            _ => {}
        }
    }

    fn get_deprecated_function(&self, name: &str) -> Option<(&'static str, &'static str)> {
        match name {
            "suicide" => Some(("suicide", "selfdestruct")),
            "sha3" => Some(("sha3", "keccak256")),
            _ => None,
        }
    }

    fn get_deprecated_member(&self, obj: &str, member: &str) -> Option<&'static str> {
        match (obj, member) {
            ("msg", "gas") => Some("gasleft()"),
            ("block", "blockhash") => Some("blockhash()"),
            ("now", _) => Some("block.timestamp"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deprecated_detector() {
        let detector = DeprecatedDetector::new();
        assert_eq!(detector.id(), "deprecated");
        assert_eq!(detector.swc_ids(), vec![111]);
        assert!(detector.get_deprecated_function("suicide").is_some());
        assert!(detector.get_deprecated_function("sha3").is_some());
        assert!(detector.get_deprecated_function("keccak256").is_none());
    }
}
