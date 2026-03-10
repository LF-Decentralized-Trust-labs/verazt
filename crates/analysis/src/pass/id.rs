//! Pass Identifier Types
//!
//! This module defines unique identifiers for all analysis passes,
//! organised into sub-enums by representation layer (step 2.1).
//! Cross-layer dependency violations are now a compile error because
//! `SirPassId` and `AirPassId` are distinct types.

use std::fmt::{self, Display};

// ========================================
// Sub-enums by representation layer
// ========================================

/// SIR-level analysis pass identifiers.
///
/// These passes operate on `mlir::sir::Module` structures (the SIR tree).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SirPassId {
    /// IR-based CFG construction
    Cfg,
    /// IR-based call graph
    CallGraph,
    /// Dominator tree construction
    DominatorTree,
    /// Loop analysis
    LoopAnalysis,
    /// SSA construction
    SsaConstruction,
    /// Def-use chain construction
    DefUseChain,
    /// Liveness analysis
    LivenessAnalysis,
    /// Taint analysis
    TaintAnalysis,
    /// IR state mutation tracking
    StateMutation,
    /// Public function writes storage without any auth guard (SIR tree pattern)
    MissingAccessControl,
    /// Public storage-writing function has no @modifies annotation
    MissingModifies,
    /// BinOp with OverflowSemantics::Wrapping on a non-constant operand
    UncheckedArithmetic,
    /// EVM: tx.origin used for authentication instead of msg.sender
    TxOriginAuth,
    /// Move: move.borrow_global<T> called on type not in #move.acquires
    AcquiresMismatch,
    /// Anchor: account loaded without #anchor.constraint="seeds=[...]"
    MissingPdaConstraint,
}

/// AIR-level analysis pass identifiers.
///
/// These passes operate on `mlir::air::AIRModule` structures (the AIR CFG).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AirPassId {
    /// Taint propagation through SSA def-use chains
    TaintPropagation,
    /// Interprocedural reentrancy detection via ICFG pattern
    Reentrancy,
    /// Access control: taint-flow path to storage write lacks SignerArg guard
    AccessControl,
    /// Arithmetic overflow on tainted operands with Wrapping semantics
    Arithmetic,
    /// Storage alias collision (proxy storage, delegatecall layout clash)
    StorageAliasing,
    /// Resource linearity violation (Move, Tezos)
    ResourceLinearity,
    /// Anchor PDA validation check
    PdaValidation,
}

/// Bug detection pass identifiers.
///
/// These correspond to scanner-side detectors (GREP / AST-based). They
/// are kept in the analysis crate so that the pipeline can schedule them
/// uniformly alongside real analysis passes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DetectionPassId {
    /// tx.origin usage detection
    TxOrigin,
    /// Floating pragma detection
    FloatingPragma,
    /// Visibility issues detection
    Visibility,
    /// Deprecated features detection
    Deprecated,
    /// Low-level call detection
    LowLevelCall,
    /// Unchecked call detection
    UncheckedCall,
    /// Variable shadowing detection
    Shadowing,
    /// Timestamp dependence detection
    TimestampDependence,
    /// Delegatecall detection
    Delegatecall,
    /// Missing access control detection
    MissingAccessControl,
    /// Constant state variable detection
    ConstantStateVar,
    /// Dead code detection
    DeadCode,
    /// Uninitialized storage detection
    UninitializedStorage,
    /// Integer overflow detection
    IntegerOverflow,
    /// Reentrancy detection
    Reentrancy,
    /// CEI violation detection
    CeiViolation,
    /// Centralization risk detection
    CentralizationRisk,
}

// ========================================
// Outer PassId enum
// ========================================

/// Unique identifier for an analysis pass.
///
/// Pass IDs are used to:
/// - Track pass dependencies
/// - Mark passes as completed in the context
/// - Register and lookup passes in the manager
///
/// Organised into sub-enums by representation layer so that
/// cross-layer dependency mistakes are caught at compile time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassId {
    /// SIR-level analysis passes
    Sir(SirPassId),
    /// AIR-level analysis passes
    Air(AirPassId),
    /// Bug detection passes (scanner-side)
    Detection(DetectionPassId),

    // ========================================
    // Legacy Passes (for backward compatibility)
    // ========================================
    /// Legacy CFG pass
    Cfg,
    /// Legacy data flow pass
    DataFlow,
    /// Legacy state mutation pass
    StateMutation,
}

// ========================================
// Convenience constructors (preserve old names)
// ========================================

/// Convenience aliases so callers can continue writing `PassId::IrCfg`
/// instead of `PassId::Sir(SirPassId::Cfg)`.
#[allow(non_upper_case_globals)]
impl PassId {
    // — SIR convenience aliases —
    pub const IrCfg: PassId = PassId::Sir(SirPassId::Cfg);
    pub const IrCallGraph: PassId = PassId::Sir(SirPassId::CallGraph);
    pub const DominatorTree: PassId = PassId::Sir(SirPassId::DominatorTree);
    pub const LoopAnalysis: PassId = PassId::Sir(SirPassId::LoopAnalysis);
    pub const SsaConstruction: PassId = PassId::Sir(SirPassId::SsaConstruction);
    pub const DefUseChain: PassId = PassId::Sir(SirPassId::DefUseChain);
    pub const LivenessAnalysis: PassId = PassId::Sir(SirPassId::LivenessAnalysis);
    pub const TaintAnalysis: PassId = PassId::Sir(SirPassId::TaintAnalysis);
    pub const IrStateMutation: PassId = PassId::Sir(SirPassId::StateMutation);
    pub const SirMissingAccessControl: PassId = PassId::Sir(SirPassId::MissingAccessControl);
    pub const SirMissingModifies: PassId = PassId::Sir(SirPassId::MissingModifies);
    pub const SirUncheckedArithmetic: PassId = PassId::Sir(SirPassId::UncheckedArithmetic);
    pub const SirTxOriginAuth: PassId = PassId::Sir(SirPassId::TxOriginAuth);
    pub const SirAcquiresMismatch: PassId = PassId::Sir(SirPassId::AcquiresMismatch);
    pub const SirMissingPdaConstraint: PassId = PassId::Sir(SirPassId::MissingPdaConstraint);

    // — AIR convenience aliases —
    pub const AIRTaintPropagation: PassId = PassId::Air(AirPassId::TaintPropagation);
    pub const AIRReentrancy: PassId = PassId::Air(AirPassId::Reentrancy);
    pub const AIRAccessControl: PassId = PassId::Air(AirPassId::AccessControl);
    pub const AIRArithmetic: PassId = PassId::Air(AirPassId::Arithmetic);
    pub const AIRStorageAliasing: PassId = PassId::Air(AirPassId::StorageAliasing);
    pub const AIRResourceLinearity: PassId = PassId::Air(AirPassId::ResourceLinearity);
    pub const AIRPdaValidation: PassId = PassId::Air(AirPassId::PdaValidation);

    // — Detection convenience aliases —
    pub const TxOrigin: PassId = PassId::Detection(DetectionPassId::TxOrigin);
    pub const FloatingPragma: PassId = PassId::Detection(DetectionPassId::FloatingPragma);
    pub const Visibility: PassId = PassId::Detection(DetectionPassId::Visibility);
    pub const Deprecated: PassId = PassId::Detection(DetectionPassId::Deprecated);
    pub const LowLevelCall: PassId = PassId::Detection(DetectionPassId::LowLevelCall);
    pub const UncheckedCall: PassId = PassId::Detection(DetectionPassId::UncheckedCall);
    pub const Shadowing: PassId = PassId::Detection(DetectionPassId::Shadowing);
    pub const TimestampDependence: PassId =
        PassId::Detection(DetectionPassId::TimestampDependence);
    pub const Delegatecall: PassId = PassId::Detection(DetectionPassId::Delegatecall);
    pub const MissingAccessControl: PassId =
        PassId::Detection(DetectionPassId::MissingAccessControl);
    pub const ConstantStateVar: PassId = PassId::Detection(DetectionPassId::ConstantStateVar);
    pub const DeadCode: PassId = PassId::Detection(DetectionPassId::DeadCode);
    pub const UninitializedStorage: PassId =
        PassId::Detection(DetectionPassId::UninitializedStorage);
    pub const IntegerOverflow: PassId = PassId::Detection(DetectionPassId::IntegerOverflow);
    pub const Reentrancy: PassId = PassId::Detection(DetectionPassId::Reentrancy);
    pub const CeiViolation: PassId = PassId::Detection(DetectionPassId::CeiViolation);
    pub const CentralizationRisk: PassId = PassId::Detection(DetectionPassId::CentralizationRisk);
}

// ========================================
// Layer queries
// ========================================

impl PassId {
    /// Get the representation layer of this pass.
    pub fn layer(&self) -> crate::pass::meta::PassRepresentation {
        use crate::pass::meta::PassRepresentation;
        match self {
            PassId::Sir(_) => PassRepresentation::Ir,
            PassId::Air(_) => PassRepresentation::Air,
            PassId::Detection(_) => PassRepresentation::Ast, // detection layer
            PassId::Cfg | PassId::DataFlow | PassId::StateMutation => PassRepresentation::Ir,
        }
    }

    /// Check if this is a SIR-level pass (requires SIR modules).
    pub fn requires_ir(&self) -> bool {
        matches!(self, PassId::Sir(_) | PassId::Cfg | PassId::DataFlow | PassId::StateMutation)
    }

    /// Check if this is an AIR-level pass (requires AIR modules).
    pub fn requires_air(&self) -> bool {
        matches!(self, PassId::Air(_))
    }

    /// Get the string representation of the pass ID.
    pub fn as_str(&self) -> &'static str {
        match self {
            // SIR
            PassId::Sir(SirPassId::Cfg) => "ir-cfg",
            PassId::Sir(SirPassId::CallGraph) => "ir-call-graph",
            PassId::Sir(SirPassId::DominatorTree) => "dominator-tree",
            PassId::Sir(SirPassId::LoopAnalysis) => "loop-analysis",
            PassId::Sir(SirPassId::SsaConstruction) => "ssa-construction",
            PassId::Sir(SirPassId::DefUseChain) => "def-use-chain",
            PassId::Sir(SirPassId::LivenessAnalysis) => "liveness-analysis",
            PassId::Sir(SirPassId::TaintAnalysis) => "taint-analysis",
            PassId::Sir(SirPassId::StateMutation) => "ir-state-mutation",
            PassId::Sir(SirPassId::MissingAccessControl) => "sir-missing-access-control",
            PassId::Sir(SirPassId::MissingModifies) => "sir-missing-modifies",
            PassId::Sir(SirPassId::UncheckedArithmetic) => "sir-unchecked-arithmetic",
            PassId::Sir(SirPassId::TxOriginAuth) => "sir-tx-origin-auth",
            PassId::Sir(SirPassId::AcquiresMismatch) => "sir-acquires-mismatch",
            PassId::Sir(SirPassId::MissingPdaConstraint) => "sir-missing-pda-constraint",
            // AIR
            PassId::Air(AirPassId::TaintPropagation) => "AIR-taint-propagation",
            PassId::Air(AirPassId::Reentrancy) => "AIR-reentrancy",
            PassId::Air(AirPassId::AccessControl) => "AIR-access-control",
            PassId::Air(AirPassId::Arithmetic) => "AIR-arithmetic",
            PassId::Air(AirPassId::StorageAliasing) => "AIR-storage-aliasing",
            PassId::Air(AirPassId::ResourceLinearity) => "AIR-resource-linearity",
            PassId::Air(AirPassId::PdaValidation) => "AIR-pda-validation",
            // Detection
            PassId::Detection(DetectionPassId::TxOrigin) => "tx-origin",
            PassId::Detection(DetectionPassId::FloatingPragma) => "floating-pragma",
            PassId::Detection(DetectionPassId::Visibility) => "visibility",
            PassId::Detection(DetectionPassId::Deprecated) => "deprecated",
            PassId::Detection(DetectionPassId::LowLevelCall) => "low-level-call",
            PassId::Detection(DetectionPassId::UncheckedCall) => "unchecked-call",
            PassId::Detection(DetectionPassId::Shadowing) => "shadowing",
            PassId::Detection(DetectionPassId::TimestampDependence) => "timestamp-dependence",
            PassId::Detection(DetectionPassId::Delegatecall) => "delegatecall",
            PassId::Detection(DetectionPassId::MissingAccessControl) => "missing-access-control",
            PassId::Detection(DetectionPassId::ConstantStateVar) => "constant-state-var",
            PassId::Detection(DetectionPassId::DeadCode) => "dead-code",
            PassId::Detection(DetectionPassId::UninitializedStorage) => "uninitialized-storage",
            PassId::Detection(DetectionPassId::IntegerOverflow) => "integer-overflow",
            PassId::Detection(DetectionPassId::Reentrancy) => "reentrancy",
            PassId::Detection(DetectionPassId::CeiViolation) => "cei-violation",
            PassId::Detection(DetectionPassId::CentralizationRisk) => "centralization-risk",
            // Legacy
            PassId::Cfg => "cfg",
            PassId::DataFlow => "data-flow",
            PassId::StateMutation => "state-mutation",
        }
    }
}

impl Display for PassId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for PassId {
    fn from(s: &str) -> Self {
        match s {
            // SIR
            "ir-cfg" => PassId::IrCfg,
            "ir-call-graph" => PassId::IrCallGraph,
            "dominator-tree" => PassId::DominatorTree,
            "loop-analysis" => PassId::LoopAnalysis,
            "ssa-construction" => PassId::SsaConstruction,
            "def-use-chain" => PassId::DefUseChain,
            "liveness-analysis" => PassId::LivenessAnalysis,
            "taint-analysis" => PassId::TaintAnalysis,
            "ir-state-mutation" => PassId::IrStateMutation,
            "sir-missing-access-control" => PassId::SirMissingAccessControl,
            "sir-missing-modifies" => PassId::SirMissingModifies,
            "sir-unchecked-arithmetic" => PassId::SirUncheckedArithmetic,
            "sir-tx-origin-auth" => PassId::SirTxOriginAuth,
            "sir-acquires-mismatch" => PassId::SirAcquiresMismatch,
            "sir-missing-pda-constraint" => PassId::SirMissingPdaConstraint,
            // AIR
            "AIR-taint-propagation" => PassId::AIRTaintPropagation,
            "AIR-reentrancy" => PassId::AIRReentrancy,
            "AIR-access-control" => PassId::AIRAccessControl,
            "AIR-arithmetic" => PassId::AIRArithmetic,
            "AIR-storage-aliasing" => PassId::AIRStorageAliasing,
            "AIR-resource-linearity" => PassId::AIRResourceLinearity,
            "AIR-pda-validation" => PassId::AIRPdaValidation,
            // Detection
            "tx-origin" => PassId::TxOrigin,
            "floating-pragma" => PassId::FloatingPragma,
            "visibility" => PassId::Visibility,
            "deprecated" => PassId::Deprecated,
            "low-level-call" => PassId::LowLevelCall,
            "unchecked-call" => PassId::UncheckedCall,
            "shadowing" => PassId::Shadowing,
            "timestamp-dependence" => PassId::TimestampDependence,
            "delegatecall" => PassId::Delegatecall,
            "missing-access-control" => PassId::MissingAccessControl,
            "constant-state-var" => PassId::ConstantStateVar,
            "dead-code" => PassId::DeadCode,
            "uninitialized-storage" => PassId::UninitializedStorage,
            "integer-overflow" => PassId::IntegerOverflow,
            "reentrancy" => PassId::Reentrancy,
            "cei-violation" => PassId::CeiViolation,
            "centralization-risk" => PassId::CentralizationRisk,
            // Legacy
            "cfg" => PassId::Cfg,
            "data-flow" => PassId::DataFlow,
            "state-mutation" => PassId::StateMutation,
            _ => panic!("Unknown pass ID: {}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pass_id_display() {
        assert_eq!(PassId::IrCfg.to_string(), "ir-cfg");
        assert_eq!(PassId::AIRTaintPropagation.to_string(), "AIR-taint-propagation");
    }

    #[test]
    fn test_pass_id_from_str() {
        assert_eq!(PassId::from("ir-cfg"), PassId::IrCfg);
        assert_eq!(PassId::from("sir-missing-access-control"), PassId::SirMissingAccessControl);
    }

    #[test]
    fn test_requires_ir() {
        assert!(PassId::IrCfg.requires_ir());
        assert!(PassId::SirMissingAccessControl.requires_ir());
        assert!(!PassId::AIRTaintPropagation.requires_ir());
    }

    #[test]
    fn test_requires_air() {
        assert!(PassId::AIRTaintPropagation.requires_air());
        assert!(PassId::AIRReentrancy.requires_air());
        assert!(!PassId::SirMissingAccessControl.requires_air());
    }

    #[test]
    fn test_sir_air_roundtrip() {
        assert_eq!(PassId::from("sir-missing-access-control"), PassId::SirMissingAccessControl);
        assert_eq!(PassId::from("AIR-taint-propagation"), PassId::AIRTaintPropagation);
        assert_eq!(PassId::AIRPdaValidation.as_str(), "AIR-pda-validation");
    }

    #[test]
    fn test_layer() {
        use crate::pass::meta::PassRepresentation;
        assert_eq!(PassId::IrCfg.layer(), PassRepresentation::Ir);
        assert_eq!(PassId::AIRTaintPropagation.layer(), PassRepresentation::Air);
        assert_eq!(PassId::TxOrigin.layer(), PassRepresentation::Ast);
    }

    #[test]
    fn test_sub_enum_variants() {
        // Verify that the convenience aliases resolve to the expected sub-enum
        assert!(matches!(PassId::IrCfg, PassId::Sir(SirPassId::Cfg)));
        assert!(matches!(PassId::AIRTaintPropagation, PassId::Air(AirPassId::TaintPropagation)));
        assert!(matches!(PassId::TxOrigin, PassId::Detection(DetectionPassId::TxOrigin)));
    }
}
