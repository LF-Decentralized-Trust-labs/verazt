use crate::{engine::task_generator_ast::TaskContext, tasks::task::Task};
use bugs::bug::{Bug, BugKind, RiskLevel};
use meta::Loc;
use solidity::ast::CallExpr;

pub struct AccessControl {
    _modifier_invoc: CallExpr,
    _context: TaskContext,
}

impl AccessControl {
    pub fn new(modifier_invoc: &CallExpr, context: &TaskContext) -> Self {
        AccessControl { _modifier_invoc: modifier_invoc.clone(), _context: context.clone() }
    }

    pub fn create_issue(&self, loc: Loc) -> Bug {
        Bug::new(
            "Access Control",
            None, // FIXME: update description
            loc,
            BugKind::Vulnerability,
            RiskLevel::Medium,
            vec![], // FIXME: update related CWE IDs
            vec![], // FIXME: update related SWC IDs
        )
    }
}

impl Task for AccessControl {
    fn check(&self) -> Vec<Bug> {
        // Implementation of the centralization risk task
        vec![]
    }
}
