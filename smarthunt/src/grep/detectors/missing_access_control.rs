//! Missing Access Control Detector (GREP-based)
//!
//! Detects public/external functions that lack proper access control
//! using pattern matching on function modifiers and state mutations.

use crate::analysis::context::AnalysisContext;
use crate::analysis::pass::Pass;
use crate::analysis::pass_id::PassId;
use crate::analysis::pass_level::PassLevel;
use crate::analysis::pass_representation::PassRepresentation;
use crate::detection::pass::{BugDetectionPass, ConfidenceLevel, DetectorResult, create_bug};
use bugs::bug::{Bug, BugKind, RiskLevel};
use solidity::ast::{
    Block, ContractDef, ContractElem, Expr, FuncDef, FuncVis, Loc, SourceUnitElem, Stmt,
};

/// GREP-based detector for missing access control.
///
/// Detects functions that modify state but don't have any access control
/// modifiers like `onlyOwner`, `onlyAdmin`, etc.
#[derive(Debug, Default)]
pub struct MissingAccessControlGrepDetector;

impl MissingAccessControlGrepDetector {
    pub fn new() -> Self {
        Self
    }

    /// Known access control modifiers.
    const ACCESS_CONTROL_MODIFIERS: &'static [&'static str] = &[
        "onlyowner",
        "onlyadmin",
        "onlyrole",
        "onlyauthorized",
        "onlygovernance",
        "onlyoperator",
        "onlyminter",
        "onlypauser",
        "auth",
        "restricted",
        "requiresauth",
    ];

    fn check_contract(&self, contract: &ContractDef, bugs: &mut Vec<Bug>) {
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                self.check_function(func, &contract.name.base, bugs);
            }
        }
    }

    fn check_function(&self, func: &FuncDef, contract_name: &str, bugs: &mut Vec<Bug>) {
        let func_name = func.name.base.as_str();

        // Skip constructors, fallback, and receive
        if func_name.is_empty()
            || func_name == "constructor"
            || func_name == "fallback"
            || func_name == "receive"
        {
            return;
        }

        // Only check public/external functions
        let is_public_or_external = match &func.visibility {
            FuncVis::Public | FuncVis::External => true,
            FuncVis::None => true, // Default is public
            _ => false,
        };

        if !is_public_or_external {
            return;
        }

        // Check if function has any access control modifier
        let has_access_control = func.modifier_invocs.iter().any(|m| {
            if let Expr::Ident(ident) = m.callee.as_ref() {
                let name = ident.name.base.as_str().to_lowercase();
                Self::ACCESS_CONTROL_MODIFIERS
                    .iter()
                    .any(|ac| name.contains(ac))
            } else if let Expr::Call(call) = m.callee.as_ref() {
                // Handle modifiers called as functions (e.g., onlyRole(ADMIN))
                if let Expr::Ident(ident) = call.callee.as_ref() {
                    let name = ident.name.base.as_str().to_lowercase();
                    Self::ACCESS_CONTROL_MODIFIERS
                        .iter()
                        .any(|ac| name.contains(ac))
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

        // Check if function mutates state
        if let Some(body) = &func.body {
            if self.has_state_mutation(body) {
                // Check for sensitive operations
                let has_sensitive_ops = self.has_sensitive_operations(body);

                if has_sensitive_ops {
                    let loc = func.loc.unwrap_or(Loc::new(1, 1, 1, 1));
                    let bug = create_bug(
                        self,
                        Some(&format!(
                            "Function '{}' in '{}' performs sensitive operations \
                             without access control. Consider adding an access control modifier.",
                            func_name, contract_name,
                        )),
                        loc,
                    );
                    bugs.push(bug);
                }
            }
        }
    }

    /// Check if a block contains any state mutation.
    fn has_state_mutation(&self, block: &Block) -> bool {
        for stmt in &block.body {
            if self.stmt_mutates_state(stmt) {
                return true;
            }
        }
        false
    }

    fn stmt_mutates_state(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(expr_stmt) => self.expr_mutates_state(&expr_stmt.expr),
            Stmt::Block(block) => self.has_state_mutation(block),
            Stmt::If(if_stmt) => {
                self.stmt_mutates_state(&if_stmt.true_branch)
                    || if_stmt
                        .false_branch
                        .as_ref()
                        .map_or(false, |fb| self.stmt_mutates_state(fb))
            }
            Stmt::While(w) => self.stmt_mutates_state(&w.body),
            Stmt::DoWhile(d) => self.stmt_mutates_state(&d.body),
            Stmt::For(f) => self.stmt_mutates_state(&f.body),
            _ => false,
        }
    }

    fn expr_mutates_state(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Assign(_) => true,
            Expr::Call(call) => {
                // Check for state-modifying calls
                if let Expr::Member(member) = call.callee.as_ref() {
                    let method = member.member.base.as_str();
                    matches!(method, "push" | "pop" | "transfer" | "send" | "call")
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Check if a block contains sensitive operations (selfdestruct, transfer,
    /// etc.)
    fn has_sensitive_operations(&self, block: &Block) -> bool {
        for stmt in &block.body {
            if self.stmt_has_sensitive_ops(stmt) {
                return true;
            }
        }
        false
    }

    fn stmt_has_sensitive_ops(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(expr_stmt) => self.expr_has_sensitive_ops(&expr_stmt.expr),
            Stmt::Block(block) => self.has_sensitive_operations(block),
            Stmt::If(if_stmt) => {
                self.stmt_has_sensitive_ops(&if_stmt.true_branch)
                    || if_stmt
                        .false_branch
                        .as_ref()
                        .map_or(false, |fb| self.stmt_has_sensitive_ops(fb))
            }
            Stmt::While(w) => self.stmt_has_sensitive_ops(&w.body),
            Stmt::DoWhile(d) => self.stmt_has_sensitive_ops(&d.body),
            Stmt::For(f) => self.stmt_has_sensitive_ops(&f.body),
            _ => false,
        }
    }

    fn expr_has_sensitive_ops(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call(call) => {
                // Check for selfdestruct
                if let Expr::Ident(ident) = call.callee.as_ref() {
                    let name = ident.name.base.as_str();
                    if matches!(name, "selfdestruct" | "suicide") {
                        return true;
                    }
                }
                // Check for transfer, send, call with value
                if let Expr::Member(member) = call.callee.as_ref() {
                    let method = member.member.base.as_str();
                    if matches!(method, "transfer" | "send" | "call") {
                        return true;
                    }
                }
                false
            }
            Expr::CallOpts(call_opts) => {
                if let Expr::Member(member) = call_opts.callee.as_ref() {
                    let method = member.member.base.as_str();
                    matches!(method, "call" | "transfer" | "send")
                } else {
                    false
                }
            }
            Expr::Assign(_) => true, // State assignment
            _ => false,
        }
    }
}

impl Pass for MissingAccessControlGrepDetector {
    fn id(&self) -> PassId {
        PassId::MissingAccessControl
    }

    fn name(&self) -> &'static str {
        "Missing Access Control"
    }

    fn description(&self) -> &'static str {
        "Detects functions lacking proper access control"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![]
    }
}

impl BugDetectionPass for MissingAccessControlGrepDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        for source_unit in &context.source_units {
            for elem in &source_unit.elems {
                if let SourceUnitElem::Contract(contract) = elem {
                    self.check_contract(contract, &mut bugs);
                }
            }
        }

        Ok(bugs)
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
        vec![105, 106]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_access_control_grep_detector() {
        let detector = MissingAccessControlGrepDetector::new();
        assert_eq!(detector.id(), PassId::MissingAccessControl);
        assert_eq!(detector.risk_level(), RiskLevel::High);
    }
}
