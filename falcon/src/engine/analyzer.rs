use super::{config::Config, task_generator_ast, task_generator_ir};
// use solidity::ast::{ContractDef, FuncDef, SourceUnit, Stmt, source_unit};

//-------------------------------------------------------------------------
// Data structures representing Analyzer
//-------------------------------------------------------------------------

/// Analyzer is responsible for analyzing smart contract AST and IR.
///
/// Make it generic to handle different smart contract languages.
pub struct Analyzer {
    pub config: Config,
}

//-------------------------------------------------------------------------
// Implementation for Analyzer
//-------------------------------------------------------------------------

impl Analyzer {
    /// Function to analyze Solidity AST.
    pub fn analyze_solidity_ast(&self, source_unit: &solidity::ast::SourceUnit) {
        let tasks = task_generator_ast::generate_tasks(&self.config, source_unit);
        for task in tasks {
            task.check();
        }
    }

    /// Function to analyze Solidity IR.
    pub fn analyze_solidity_ir(&self, source_unit: solidity::ir::SourceUnit) {
        let tasks = task_generator_ir::initiate_tasks(source_unit);
        for task in tasks {
            task.check();
        }
    }
}
