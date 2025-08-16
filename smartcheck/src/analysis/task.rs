use crate::detectors::Detector;

pub trait Task {
    fn analyze(&self);
}
