use issue::issue::Issue;

pub trait Task {
    /// Generic function to run the task.
    fn check(&self) -> Vec<Issue>;
}
