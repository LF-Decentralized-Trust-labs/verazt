use crate::tasks::task::Task;
use issue::issue::{Issue, Severity};

pub struct CentralizationRisk {}

impl CentralizationRisk {
    pub fn create_issue(&self) -> Issue {
        Issue::new(
            "Centralization Risk".into(),
            "This issue is related to centralization risk in the smart contract.".into(),
            Severity::Medium,
            vec![], // FIXME: fill SWC IDs later
            vec![], // FIXME: fill CWE IDs later
        )
    }
}

impl Task for CentralizationRisk {
    fn check(&self) -> Vec<Issue> {
        // Implementation of the centralization risk task
        vec![]
    }
}
