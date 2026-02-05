pub mod analyzer;
pub mod config;
pub mod context;
pub mod parallel;
pub mod scheduler;
pub mod task_generator_ast;
pub mod task_generator_ir;

pub use config::Config;
pub use context::AnalysisContext;
pub use scheduler::{PassScheduler, create_default_scheduler};
pub use parallel::{ParallelConfig, DetectorRunner, run_detectors, run_detectors_parallel};


