//! Control Flow Graph Pass.
//!
//! This pass builds control flow graphs for all functions.

use crate::engine::context::AnalysisContext;
use crate::graph::{
    BasicBlock, BlockId, CfgCollection, CfgStatement, ControlFlowGraph, FunctionId,
};
use crate::passes::{AnalysisPass, PassId, PassResult};
use solidity::ast::{
    Block, ContractDef, ContractElem, Expr, FuncDef, SourceUnit, SourceUnitElem, Stmt,
};

/// Pass for building control flow graphs.
pub struct CfgPass;

impl CfgPass {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CfgPass {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisPass for CfgPass {
    fn id(&self) -> PassId {
        PassId::Cfg
    }

    fn name(&self) -> &'static str {
        "Control Flow Graph Construction"
    }

    fn description(&self) -> &'static str {
        "Build control flow graphs for all functions"
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable]
    }

    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        let mut builder = CfgBuilder::new();
        
        for source_unit in &context.source_units {
            builder.visit_source_unit(source_unit);
        }
        
        context.cfgs = Some(builder.cfgs);
        Ok(())
    }

    fn is_completed(&self, context: &AnalysisContext) -> bool {
        context.cfgs.is_some()
    }
}

/// CFG builder that visits AST nodes and builds CFGs.
struct CfgBuilder {
    cfgs: CfgCollection,
    current_contract: Option<ContractDef>,
}

impl CfgBuilder {
    fn new() -> Self {
        Self {
            cfgs: CfgCollection::new(),
            current_contract: None,
        }
    }

    fn visit_source_unit(&mut self, source_unit: &SourceUnit) {
        for elem in &source_unit.elems {
            match elem {
                SourceUnitElem::Contract(contract) => {
                    self.current_contract = Some(contract.clone());
                    self.visit_contract(contract);
                    self.current_contract = None;
                }
                SourceUnitElem::Func(func) => {
                    self.build_function_cfg(func, None);
                }
                _ => {}
            }
        }
    }

    fn visit_contract(&mut self, contract: &ContractDef) {
        for elem in &contract.body {
            if let ContractElem::Func(func) = elem {
                self.build_function_cfg(func, Some(contract));
            }
        }
    }

    fn build_function_cfg(&mut self, func: &FuncDef, contract: Option<&ContractDef>) {
        let func_id = FunctionId::from_func(func, contract);
        let mut cfg = ControlFlowGraph::new(func_id.clone());
        
        if let Some(body) = &func.body {
            let mut builder = FunctionCfgBuilder::new(&mut cfg);
            builder.build_from_block(body);
        }
        
        self.cfgs.insert(cfg);
    }
}

/// Builder for a single function's CFG.
struct FunctionCfgBuilder<'a> {
    cfg: &'a mut ControlFlowGraph,
    current_block: BlockId,
}

impl<'a> FunctionCfgBuilder<'a> {
    fn new(cfg: &'a mut ControlFlowGraph) -> Self {
        let entry = cfg.entry;
        Self {
            cfg,
            current_block: entry,
        }
    }

    fn build_from_block(&mut self, block: &Block) {
        // Connect entry to first real block
        let first_block = self.cfg.create_block();
        self.cfg.add_edge(self.current_block, first_block);
        self.current_block = first_block;
        
        for stmt in &block.body {
            self.visit_stmt(stmt);
        }
        
        // Connect last block to exit
        let exit = self.cfg.exits[0];
        self.cfg.add_edge(self.current_block, exit);
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(block) => {
                for s in &block.body {
                    self.visit_stmt(s);
                }
            }
            
            Stmt::If(if_stmt) => {
                let condition_block = self.current_block;
                let true_block = self.cfg.create_block();
                let false_block = self.cfg.create_block();
                let merge_block = self.cfg.create_block();
                
                // Add branch statement to condition block
                if let Some(block) = self.cfg.get_block_mut(&condition_block) {
                    block.add_statement(CfgStatement::Branch {
                        condition: if_stmt.condition.clone(),
                        true_target: true_block,
                        false_target: false_block,
                        loc: if_stmt.loc,
                    });
                }
                
                self.cfg.add_edge(condition_block, true_block);
                self.cfg.add_edge(condition_block, false_block);
                
                // Build true branch
                self.current_block = true_block;
                self.visit_stmt(&if_stmt.true_branch);
                self.cfg.add_edge(self.current_block, merge_block);
                
                // Build false branch
                self.current_block = false_block;
                if let Some(false_br) = &if_stmt.false_branch {
                    self.visit_stmt(false_br);
                }
                self.cfg.add_edge(self.current_block, merge_block);
                
                self.current_block = merge_block;
            }
            
            Stmt::For(for_stmt) => {
                // Pre-loop (initialization)
                if let Some(pre) = &for_stmt.pre_loop {
                    self.visit_stmt(pre);
                }
                
                let header_block = self.cfg.create_block();
                let body_block = self.cfg.create_block();
                let post_block = self.cfg.create_block();
                let exit_block = self.cfg.create_block();
                
                self.cfg.add_edge(self.current_block, header_block);
                
                // Header (condition check)
                if let Some(cond) = &for_stmt.condition {
                    if let Some(block) = self.cfg.get_block_mut(&header_block) {
                        block.add_statement(CfgStatement::Branch {
                            condition: cond.clone(),
                            true_target: body_block,
                            false_target: exit_block,
                            loc: for_stmt.loc,
                        });
                    }
                }
                
                self.cfg.add_edge(header_block, body_block);
                self.cfg.add_edge(header_block, exit_block);
                
                // Body
                self.current_block = body_block;
                self.visit_stmt(&for_stmt.body);
                self.cfg.add_edge(self.current_block, post_block);
                
                // Post-loop (increment)
                self.current_block = post_block;
                if let Some(post) = &for_stmt.post_loop {
                    self.visit_stmt(post);
                }
                self.cfg.add_edge(self.current_block, header_block);
                
                self.current_block = exit_block;
            }
            
            Stmt::While(while_stmt) => {
                let header_block = self.cfg.create_block();
                let body_block = self.cfg.create_block();
                let exit_block = self.cfg.create_block();
                
                self.cfg.add_edge(self.current_block, header_block);
                
                if let Some(block) = self.cfg.get_block_mut(&header_block) {
                    block.add_statement(CfgStatement::Branch {
                        condition: while_stmt.condition.clone(),
                        true_target: body_block,
                        false_target: exit_block,
                        loc: while_stmt.loc,
                    });
                }
                
                self.cfg.add_edge(header_block, body_block);
                self.cfg.add_edge(header_block, exit_block);
                
                self.current_block = body_block;
                self.visit_stmt(&while_stmt.body);
                self.cfg.add_edge(self.current_block, header_block);
                
                self.current_block = exit_block;
            }
            
            Stmt::DoWhile(do_while) => {
                let body_block = self.cfg.create_block();
                let condition_block = self.cfg.create_block();
                let exit_block = self.cfg.create_block();
                
                self.cfg.add_edge(self.current_block, body_block);
                
                self.current_block = body_block;
                self.visit_stmt(&do_while.body);
                self.cfg.add_edge(self.current_block, condition_block);
                
                if let Some(block) = self.cfg.get_block_mut(&condition_block) {
                    block.add_statement(CfgStatement::Branch {
                        condition: do_while.condition.clone(),
                        true_target: body_block,
                        false_target: exit_block,
                        loc: do_while.loc,
                    });
                }
                
                self.cfg.add_edge(condition_block, body_block);
                self.cfg.add_edge(condition_block, exit_block);
                
                self.current_block = exit_block;
            }
            
            Stmt::Return(ret) => {
                if let Some(block) = self.cfg.get_block_mut(&self.current_block) {
                    block.add_statement(CfgStatement::Return {
                        value: ret.expr.clone(),
                        loc: ret.loc,
                    });
                }
                // After return, create a new block that won't be connected
                // (dead code after return)
                self.current_block = self.cfg.create_block();
            }
            
            Stmt::Expr(expr_stmt) => {
                let stmt = match &expr_stmt.expr {
                    Expr::Call(_) => CfgStatement::Call {
                        expr: expr_stmt.expr.clone(),
                        loc: expr_stmt.loc,
                    },
                    Expr::Assign(assign) => CfgStatement::Assign {
                        lhs: assign.left.as_ref().clone(),
                        rhs: assign.right.as_ref().clone(),
                        loc: expr_stmt.loc,
                    },
                    _ => CfgStatement::Expr {
                        expr: expr_stmt.expr.clone(),
                        loc: expr_stmt.loc,
                    },
                };
                
                if let Some(block) = self.cfg.get_block_mut(&self.current_block) {
                    block.add_statement(stmt);
                }
            }
            
            Stmt::VarDecl(var_decl) => {
                for var in var_decl.var_decls.iter().flatten() {
                    if let Some(block) = self.cfg.get_block_mut(&self.current_block) {
                        block.add_statement(CfgStatement::VarDecl {
                            var: var.clone(),
                            value: var_decl.value.clone(),
                            loc: var_decl.loc,
                        });
                    }
                }
            }
            
            Stmt::Revert(revert) => {
                if let Some(block) = self.cfg.get_block_mut(&self.current_block) {
                    block.add_statement(CfgStatement::Revert {
                        error: revert.error.clone(),
                        loc: revert.loc,
                    });
                }
                self.current_block = self.cfg.create_block();
            }
            
            Stmt::Asm(asm) => {
                if let Some(block) = self.cfg.get_block_mut(&self.current_block) {
                    block.add_statement(CfgStatement::Assembly {
                        loc: asm.loc,
                    });
                }
            }
            
            Stmt::Placeholder(ph) => {
                if let Some(block) = self.cfg.get_block_mut(&self.current_block) {
                    block.add_statement(CfgStatement::Placeholder {
                        loc: ph.loc,
                    });
                }
            }
            
            Stmt::Try(try_stmt) => {
                // Simplified: treat try-catch as sequential
                if let Some(block) = self.cfg.get_block_mut(&self.current_block) {
                    block.add_statement(CfgStatement::Call {
                        expr: try_stmt.guarded_expr.clone(),
                        loc: try_stmt.loc,
                    });
                }
                
                for s in &try_stmt.body.body {
                    self.visit_stmt(s);
                }
                
                // Visit catch clauses
                for clause in &try_stmt.catch_clauses {
                    for s in &clause.body.body {
                        self.visit_stmt(s);
                    }
                }
            }
            
            Stmt::Emit(_) | Stmt::Break(_) | Stmt::Continue(_) | Stmt::Throw(_) => {
                // These don't add statements to CFG or are handled separately
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfg_pass() {
        let pass = CfgPass::new();
        assert_eq!(pass.id(), PassId::Cfg);
        assert_eq!(pass.dependencies(), vec![PassId::SymbolTable]);
    }
}
