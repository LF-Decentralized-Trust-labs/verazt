use crate::{engine::task_generator_ast::TaskContext, tasks::task::Task};
use bugs::bug::{Bug, BugKind, RiskLevel};
use solidity::ast::Loc;
use solidity::ast::CallExpr;

pub struct CentralizationRisk {
    // Modifier invocation to check the centralization risk.
    _modifier_invoc: CallExpr,
    _context: TaskContext,
}

impl CentralizationRisk {
    pub fn new(modifier_invoc: &CallExpr, context: &TaskContext) -> Self {
        CentralizationRisk { _modifier_invoc: modifier_invoc.clone(), _context: context.clone() }
    }

    pub fn create_issue(&self, loc: Loc) -> Bug {
        Bug::new(
            "Centralization Risk",
            None, // FIXME: update description
            loc,
            BugKind::Vulnerability,
            RiskLevel::Medium,
            vec![], // FIXME: update related CWE IDs
            vec![], // FIXME: update related SWC IDs
        )
    }
}

impl Task for CentralizationRisk {
    fn check(&self) -> Vec<Bug> {
        // Implementation of the centralization risk task
        vec![]
    }
}
