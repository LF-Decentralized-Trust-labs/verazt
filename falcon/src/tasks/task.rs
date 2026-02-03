use bugs::bug::Bug;

pub trait Task {
    /// Generic function to run the task.
    fn check(&self) -> Vec<Bug>;
}
