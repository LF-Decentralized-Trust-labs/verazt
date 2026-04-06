//! Contract-level EVM detectors

pub mod centralization_risk;
pub mod constant_state_var;
pub mod dead_code;
pub mod front_running;
pub mod missing_access_control;
pub mod uninitialized;
pub mod visibility;

pub use centralization_risk::CentralizationRiskDetector;
pub use constant_state_var::ConstantStateVarDetector;
pub use dead_code::DeadCodeDetector;
pub use front_running::FrontRunningDetector;
pub use missing_access_control::MissingAccessControlDetector;
pub use uninitialized::UninitializedDetector;
pub use visibility::VisibilityDetector;
