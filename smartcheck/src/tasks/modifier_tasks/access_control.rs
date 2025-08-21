use crate::{engine::task_generator_ast::TaskContext, tasks::task::Task};
use issue::issue::{Issue, Severity};
use meta::Loc;
use solidity::ast::CallExpr;

pub struct AccessControlDetector {
    modifier_invoc: CallExpr,
    context: TaskContext,
}

impl AccessControlDetector {
    pub fn new(modifier_invoc: CallExpr, context: TaskContext) -> Self {
        AccessControlDetector { modifier_invoc, context }
    }

    pub fn create_issue(&self, loc: Loc) -> Issue {
        Issue::new(
            "Access Control",
            None, // FIXME: update description
            loc,
            Severity::Medium,
            vec![], // FIXME: fill SWC IDs later
            vec![], // FIXME: fill CWE IDs later
        )
    }
}

impl Task for AccessControlDetector {
    fn check(&self) -> Vec<Issue> {
        // Implementation of the centralization risk task
        vec![]
    }
}
