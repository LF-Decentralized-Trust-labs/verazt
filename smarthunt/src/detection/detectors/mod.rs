//! Bug Detectors
//!
//! This module contains all bug detection passes organized by the
//! representation they operate on.
//!
//! # Organization
//!
//! - `ast`: Legacy AST-based detectors (currently empty, reserved for future use)
//! - `dfa`: DFA-based detectors operating on IR with data flow analysis
//! - `grep`: GREP-based detectors using declarative AST pattern matching
//!
//! # Detector Naming Convention
//!
//! - DFA detectors: `<Issue>DfaDetector`
//! - GREP detectors: `<Issue>GrepDetector`
//!
//! # Available Detectors
//!
//! ## DFA-Based Detectors (IR + Data Flow Analysis)
//!
//! - `CeiViolationDfaDetector` - Checks-Effects-Interactions pattern violations
//! - `DeadCodeDfaDetector` - Unreachable code detection via liveness analysis
//! - `ReentrancyDfaDetector` - Reentrancy vulnerability detection
//! - `UncheckedCallDfaDetector` - Unchecked external call returns
//! - `UninitializedDfaDetector` - Uninitialized storage variables
//!
//! ## GREP-Based Detectors (AST Pattern Matching)
//!
//! - `CentralizationRiskGrepDetector` - Centralization risks (onlyOwner, etc.)
//! - `ConstantStateVarGrepDetector` - State variables that should be constants
//! - `DelegatecallGrepDetector` - Unsafe delegatecall usage
//! - `DeprecatedGrepDetector` - Usage of deprecated functions
//! - `FloatingPragmaGrepDetector` - Unlocked compiler pragma versions
//! - `LowLevelCallGrepDetector` - Low-level call usage (call, delegatecall)
//! - `MissingAccessControlGrepDetector` - Missing access control on sensitive functions
//! - `ShadowingGrepDetector` - Variable shadowing issues
//! - `TimestampDependenceGrepDetector` - Timestamp dependence in critical logic
//! - `TxOriginGrepDetector` - Dangerous tx.origin usage for authentication
//! - `VisibilityGrepDetector` - Function visibility issues

pub mod ast;

// Re-export DFA detectors from the dfa module
pub use crate::dfa::detectors::*;

// Re-export GREP detectors from the grep module
pub use crate::grep::detectors::*;
