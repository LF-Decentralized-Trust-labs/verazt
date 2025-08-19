//! Module to generate tasks

use std::vec;

use crate::analysis::task::Task;
use solidity::ast::*;

use super::config::Config;

//-------------------------------------------------------------------------
// Data structures representing task generator
//-------------------------------------------------------------------------

pub struct TaskGenerator {
    pub config: Config,
    pub source_unit: Option<SourceUnit>,
    pub contract: Option<ContractDef>,
    pub function: Option<FuncDef>,
}

/// An analysis context of a Task
pub struct TaskContext {
    pub source_unit: Option<SourceUnit>, // Current source unit
    pub contract: Option<ContractDef>,   // Current contract
    pub function: Option<FuncDef>,       // Current function
}

//-------------------------------------------------------------------------
// Implementation for TaskGenerator
//-------------------------------------------------------------------------

impl TaskGenerator {
    /// Constructor
    pub fn new(config: &Config) -> Self {
        TaskGenerator { config: config.clone(), source_unit: None, contract: None, function: None }
    }

    /// Populate analysis tasks for Source Unit AST.
    pub fn populate_ast_tasks(&mut self, source_unit: &SourceUnit) -> Vec<Box<dyn Task>> {
        self.source_unit = Some(source_unit.clone());

        let mut tasks = vec![];
        for elem in source_unit.elems.iter() {
            match elem {
                SourceUnitElem::Func(func) => {
                    tasks.extend(self.generate_function_task(func));
                }
                SourceUnitElem::Contract(contract) => {
                    tasks.extend(self.generate_contract_task(contract));
                }
                _ => {}
            }
        }
        tasks
    }

    fn generate_contract_task(&mut self, contract: &ContractDef) -> Vec<Box<dyn Task>> {
        self.contract = Some(contract.clone());

        let mut tasks = vec![];
        for elem in contract.body.iter() {
            match elem {
                ContractElem::Func(func) => {
                    tasks.extend(self.generate_function_task(func));
                }
                _ => {}
            }
        }
        tasks
    }

    fn generate_function_task(&mut self, function: &FuncDef) -> Vec<Box<dyn Task>> {
        self.function = Some(function.clone());

        let mut tasks = vec![];
        if let Some(blk) = &function.body {
            for stmt in blk.body.iter() {
                tasks.extend(self.generate_statement_task(stmt));
            }
        }
        tasks
    }

    fn generate_statement_task(&mut self, stmt: &Stmt) -> Vec<Box<dyn Task>> {
        vec![]
    }
}

//-------------------------------------------------------------------------
// Implementation for TaskContext
//-------------------------------------------------------------------------

impl TaskContext {
    /// Constructor
    pub fn new(
        source_unit: Option<SourceUnit>,
        contract: Option<ContractDef>,
        function: Option<FuncDef>,
    ) -> Self {
        TaskContext { source_unit, contract, function }
    }
}

//-------------------------------------------------------------------------
// Public functions
//-------------------------------------------------------------------------

pub fn generate_tasks(config: &Config, source_unit: &SourceUnit) -> Vec<Box<dyn Task>> {
    let mut generator = TaskGenerator::new(config);
    let tasks = generator.populate_ast_tasks(source_unit);
    tasks
}
