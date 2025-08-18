use super::{task_initiator_ast, task_initiator_ir};
use solidity::ast::{ContractDef, FuncDef, SourceUnit, Stmt, source_unit};

//-------------------------------------------------------------------------
// Data structures representing Analyzer
//-------------------------------------------------------------------------

/// Analyzer is responsible for analyzing smart contract AST and IR.
///
/// Make it generic to handle different smart contract languages.
pub struct Analyzer {}

//-------------------------------------------------------------------------
// Implementation for Analyzer
//-------------------------------------------------------------------------

impl Analyzer {
    /// Function to analyze Solidity AST.
    pub fn analyze_solidity_ast(&self, source_unit: solidity::ast::SourceUnit) {
        let tasks = task_initiator_ast::initiate_tasks(source_unit);
        for task in tasks {
            task.run();
        }
    }

    /// Function to analyze Solidity IR.
    pub fn analyze_solidity_ir(&self, source_unit: smartir::ir::SourceUnit) {
        let tasks = task_initiator_ir::initiate_tasks(source_unit);
        for task in tasks {
            task.run();
        }
    }
}
