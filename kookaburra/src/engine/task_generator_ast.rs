//! Module to generate tasks

use super::config::Config;
use crate::tasks::{
    modf_tasks::{self},
    task::Task,
};
use solidity::ast::*;
use std::vec;

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
#[derive(Debug, Clone)]
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

    /// Generate analysis tasks for Source Unit AST.
    pub fn generate_tasks(&mut self, source_unit: &SourceUnit) -> Vec<Box<dyn Task>> {
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

    /// Get the current context of the task generator.
    fn current_context(&self) -> TaskContext {
        TaskContext {
            source_unit: self.source_unit.clone(),
            contract: self.contract.clone(),
            function: self.function.clone(),
        }
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

    fn generate_function_task(&mut self, func: &FuncDef) -> Vec<Box<dyn Task>> {
        self.function = Some(func.clone());

        let mut tasks = vec![];

        // Modifier tasks
        for modifier_invoc in func.modifier_invocs.iter() {
            tasks.extend(self.generate_modifier_task(modifier_invoc));
        }

        // Statement tasks
        if let Some(blk) = &func.body {
            for stmt in blk.body.iter() {
                tasks.extend(self.generate_statement_task(stmt));
            }
        }

        tasks
    }

    fn generate_statement_task(&mut self, _stmt: &Stmt) -> Vec<Box<dyn Task>> {
        vec![]
    }

    fn generate_modifier_task(&mut self, modf_invoc: &CallExpr) -> Vec<Box<dyn Task>> {
        let ctx = self.current_context();
        let access_control = modf_tasks::AccessControl::new(modf_invoc, &ctx);
        let central_risk = modf_tasks::CentralizationRisk::new(modf_invoc, &ctx);
        vec![Box::new(central_risk), Box::new(access_control)]
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
    let tasks = generator.generate_tasks(source_unit);
    tasks
}
