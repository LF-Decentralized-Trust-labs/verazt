//! Access Control Analysis Pass.
//!
//! This pass analyzes access control patterns in contracts.

use crate::engine::context::{AccessControlInfo, AnalysisContext};
use crate::graph::FunctionId;
use crate::passes::{AnalysisPass, PassId, PassResult};
use solidity::ast::{
    Block, ContractDef, ContractElem, Expr, FuncDef, SourceUnit, SourceUnitElem, Stmt,
};

/// Pass for analyzing access control.
pub struct AccessControlPass;

impl AccessControlPass {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AccessControlPass {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisPass for AccessControlPass {
    fn id(&self) -> PassId {
        PassId::AccessControl
    }

    fn name(&self) -> &'static str {
        "Access Control Analysis"
    }

    fn description(&self) -> &'static str {
        "Verify access control patterns"
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable]
    }

    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        let mut analyzer = AccessControlAnalyzer::new();
        
        for source_unit in &context.source_units {
            analyzer.visit_source_unit(source_unit);
        }
        
        context.access_control = Some(analyzer.info);
        Ok(())
    }

    fn is_completed(&self, context: &AnalysisContext) -> bool {
        context.access_control.is_some()
    }
}

/// Access control analyzer.
struct AccessControlAnalyzer {
    info: AccessControlInfo,
}

impl AccessControlAnalyzer {
    fn new() -> Self {
        Self {
            info: AccessControlInfo::new(),
        }
    }

    fn visit_source_unit(&mut self, source_unit: &SourceUnit) {
        for elem in &source_unit.elems {
            match elem {
                SourceUnitElem::Contract(contract) => {
                    self.visit_contract(contract);
                }
                SourceUnitElem::Func(func) => {
                    self.visit_function(func, None);
                }
                _ => {}
            }
        }
    }

    fn visit_contract(&mut self, contract: &ContractDef) {
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                self.visit_function(func, Some(contract));
            }
        }
    }

    fn visit_function(&mut self, func: &FuncDef, contract: Option<&ContractDef>) {
        let func_id = FunctionId::from_func(func, contract);
        
        // Check if function has access control modifiers
        for modifier in &func.modifier_invocs {
            if let Expr::Ident(ident) = modifier.callee.as_ref() {
                if self.info.access_control_modifiers.contains(ident.name.base.as_str()) {
                    self.info.mark_protected(func_id.clone());
                }
            } else if let Expr::Call(call) = modifier.callee.as_ref() {
                if let Expr::Ident(ident) = call.callee.as_ref() {
                    if self.info.access_control_modifiers.contains(ident.name.base.as_str()) {
                        self.info.mark_protected(func_id.clone());
                    }
                }
            }
        }
        
        // Check if function body checks msg.sender
        if let Some(body) = &func.body {
            if self.checks_msg_sender(body) {
                self.info.mark_sender_checked(func_id);
            }
        }
    }

    fn checks_msg_sender(&self, block: &Block) -> bool {
        for stmt in &block.body {
            if self.stmt_checks_msg_sender(stmt) {
                return true;
            }
        }
        false
    }

    fn stmt_checks_msg_sender(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::If(if_stmt) => {
                if self.expr_uses_msg_sender(&if_stmt.condition) {
                    return true;
                }
                if self.stmt_checks_msg_sender(&if_stmt.true_branch) {
                    return true;
                }
                if let Some(false_br) = &if_stmt.false_branch {
                    if self.stmt_checks_msg_sender(false_br) {
                        return true;
                    }
                }
            }
            
            Stmt::Block(block) => {
                return self.checks_msg_sender(block);
            }
            
            Stmt::Expr(expr_stmt) => {
                // Check for require(msg.sender == ...) or similar
                if let Expr::Call(call) = &expr_stmt.expr {
                    if let Expr::Ident(ident) = call.callee.as_ref() {
                        if matches!(ident.name.base.as_str(), "require" | "assert") {
                            if let solidity::ast::CallArgs::Unnamed(args) = &call.args {
                                if let Some(first_arg) = args.first() {
                                    if self.expr_uses_msg_sender(first_arg) {
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
                self.expr_uses_msg_sender(&call.callee)
                    || match &call.args {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_control_pass() {
        let pass = AccessControlPass::new();
        assert_eq!(pass.id(), PassId::AccessControl);
    }
}
