//! Module to initiate tasks

use crate::tasks::task::Task;
use solidity::ir::SourceUnit;

//-------------------------------------------------------------------------
// Data structures for task initiator
//-------------------------------------------------------------------------

pub struct TaskInitiator {}

//-------------------------------------------------------------------------
// Implementation for Initiator
//-------------------------------------------------------------------------

impl TaskInitiator {
    /// Constructor
    pub fn new() -> Self {
        TaskInitiator {}
    }

    /// Populate analysis tasks for Source Unit IR.
    pub fn populate_ast_tasks(&self, _source_unit: SourceUnit) -> Vec<Box<dyn Task>> {
        // This function would typically gather tasks from various sources
        // and return them as a vector of boxed Task trait objects.
        vec![]
    }
}

//-------------------------------------------------------------------------
// Public functions
//-------------------------------------------------------------------------

pub fn initiate_tasks(source_unit: SourceUnit) -> Vec<Box<dyn Task>> {
    let initator = TaskInitiator::new();
    let tasks = initator.populate_ast_tasks(source_unit);
    tasks
}
