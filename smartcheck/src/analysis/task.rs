use crate::detectors::Detector;

pub trait Task {
    /// Generic function to run the task.
    fn run(&self);
}
