use crate::detectors::Detector;

//-------------------------------------------------------------------------
// Data structures representing analysis tasks
//-------------------------------------------------------------------------

pub struct Task {
    pub context: Context,
    pub detector: Detector,
}

pub struct Context {}
