use crate::{engine::task_generator_ast::TaskContext, tasks::task::Task};
use issue::issue::{Issue, IssueKind, RiskLevel};
use meta::Loc;
use solidity::ast::CallExpr;

pub struct AccessControl {
    modifier_invoc: CallExpr,
    context: TaskContext,
}

impl AccessControl {
    pub fn new(modifier_invoc: &CallExpr, context: &TaskContext) -> Self {
        AccessControl { modifier_invoc: modifier_invoc.clone(), context: context.clone() }
    }

    pub fn create_issue(&self, loc: Loc) -> Issue {
        Issue::new(
            "Access Control",
            None, // FIXME: update description
            loc,
            IssueKind::Vulnerability,
            RiskLevel::Medium,
            vec![], // FIXME: update related CWE IDs
            vec![], // FIXME: update related SWC IDs
        )
    }
}

impl Task for AccessControl {
    fn check(&self) -> Vec<Issue> {
        // Implementation of the centralization risk task
        vec![]
    }
}
