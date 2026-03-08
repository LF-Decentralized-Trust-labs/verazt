//! Pass Identifier Types
//!
//! This module defines unique identifiers for all analysis passes.

use std::fmt::{self, Display};

/// Unique identifier for an analysis pass.
///
/// Pass IDs are used to:
/// - Track pass dependencies
/// - Mark passes as completed in the context
/// - Register and lookup passes in the manager
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassId {
    // ========================================
    // AST Foundation Passes
    // ========================================
    /// Symbol table construction
    SymbolTable,
    /// Type index construction
    TypeIndex,
    /// Syntax validation
    SyntaxAnalysis,

    // ========================================
    // AST Analysis Passes
    // ========================================
    /// Inheritance graph construction
    InheritanceGraph,
    /// Storage layout computation
    StorageLayout,
    /// Event analysis
    EventAnalysis,
    /// Call graph construction (AST-based)
    CallGraph,
    /// Modifier analysis
    ModifierAnalysis,

    // ========================================
    // IR Generation
    // ========================================
    /// IR generation from AST
    IrGeneration,

    // ========================================
    // IR Control Flow Passes
    // ========================================
    /// IR-based CFG construction
    IrCfg,
    /// IR-based call graph
    IrCallGraph,
    /// Dominator tree construction
    DominatorTree,
    /// Loop analysis
    LoopAnalysis,

    // ========================================
    // IR Data Flow Passes
    // ========================================
    /// SSA construction
    SsaConstruction,
    /// Def-use chain construction
    DefUseChain,
    /// Liveness analysis
    LivenessAnalysis,
    /// Taint analysis
    TaintAnalysis,
    /// IR state mutation tracking
    IrStateMutation,

    // ========================================
    // Hybrid Passes (AST + IR)
    // ========================================
    /// Access control analysis
    AccessControl,
    /// State consistency analysis
    StateConsistency,

    // ========================================
    // Legacy Passes (for backward compatibility)
    // ========================================
    /// Legacy CFG pass
    Cfg,
    /// Legacy data flow pass
    DataFlow,
    /// Legacy state mutation pass
    StateMutation,

    // ========================================
    // Bug Detection Passes (AST-based)
    // ========================================
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

    // ========================================
    // Bug Detection Passes (IR-based)
    // ========================================
    /// Dead code detection
    DeadCode,
    /// Uninitialized storage detection
    UninitializedStorage,
    /// Integer overflow detection
    IntegerOverflow,

    // ========================================
    // Bug Detection Passes (Hybrid)
    // ========================================
    /// Reentrancy detection
    Reentrancy,
    /// CEI violation detection
    CeiViolation,
    /// Centralization risk detection
    CentralizationRisk,

    // ========================================
    // SIR Structural Passes (PassRepresentation::Ir)
    // ========================================
    /// Public function writes storage without any auth guard (SIR tree
    /// pattern)
    SirMissingAccessControl,
    /// Public storage-writing function has no @modifies annotation
    SirMissingModifies,
    /// BinOp with OverflowSemantics::Wrapping on a non-constant operand
    SirUncheckedArithmetic,
    /// EVM: tx.origin used for authentication instead of msg.sender
    SirTxOriginAuth,
    /// Move: move.borrow_global<T> called on type not in #move.acquires
    SirAcquiresMismatch,
    /// Anchor: account loaded without #anchor.constraint="seeds=[...]"
    SirMissingPdaConstraint,

    // ========================================
    // ANIR Generation (PassRepresentation::Ir — needs SIR)
    // ========================================
    /// Run Pass 2a: SIR → AnirModule
    AnirGeneration,

    // ========================================
    // ANIR Analysis Passes (PassRepresentation::Air)
    // ========================================
    /// Taint propagation through SSA def-use chains
    AnirTaintPropagation,
    /// Interprocedural reentrancy detection via ICFG pattern
    AnirReentrancy,
    /// Access control: taint-flow path to storage write lacks SignerArg guard
    AnirAccessControl,
    /// Arithmetic overflow on tainted operands with Wrapping semantics
    AnirArithmetic,
    /// Storage alias collision (proxy storage, delegatecall layout clash)
    AnirStorageAliasing,
    /// Resource linearity violation (Move, Tezos)
    AnirResourceLinearity,
    /// Anchor PDA validation check
    AnirPdaValidation,
}

impl PassId {
    /// Get the string representation of the pass ID.
    pub fn as_str(&self) -> &'static str {
        match self {
            // AST Foundation
            PassId::SymbolTable => "symbol-table",
            PassId::TypeIndex => "type-index",
            PassId::SyntaxAnalysis => "syntax-analysis",
            // AST Analysis
            PassId::InheritanceGraph => "inheritance-graph",
            PassId::StorageLayout => "storage-layout",
            PassId::EventAnalysis => "event-analysis",
            PassId::CallGraph => "call-graph",
            PassId::ModifierAnalysis => "modifier-analysis",
            // IR Generation
            PassId::IrGeneration => "ir-generation",
            // IR Control Flow
            PassId::IrCfg => "ir-cfg",
            PassId::IrCallGraph => "ir-call-graph",
            PassId::DominatorTree => "dominator-tree",
            PassId::LoopAnalysis => "loop-analysis",
            // IR Data Flow
            PassId::SsaConstruction => "ssa-construction",
            PassId::DefUseChain => "def-use-chain",
            PassId::LivenessAnalysis => "liveness-analysis",
            PassId::TaintAnalysis => "taint-analysis",
            PassId::IrStateMutation => "ir-state-mutation",
            // Hybrid
            PassId::AccessControl => "access-control",
            PassId::StateConsistency => "state-consistency",
            // Legacy
            PassId::Cfg => "cfg",
            PassId::DataFlow => "data-flow",
            PassId::StateMutation => "state-mutation",
            // AST Bug Detection
            PassId::TxOrigin => "tx-origin",
            PassId::FloatingPragma => "floating-pragma",
            PassId::Visibility => "visibility",
            PassId::Deprecated => "deprecated",
            PassId::LowLevelCall => "low-level-call",
            PassId::UncheckedCall => "unchecked-call",
            PassId::Shadowing => "shadowing",
            PassId::TimestampDependence => "timestamp-dependence",
            PassId::Delegatecall => "delegatecall",
            PassId::MissingAccessControl => "missing-access-control",
            PassId::ConstantStateVar => "constant-state-var",
            // IR Bug Detection
            PassId::DeadCode => "dead-code",
            PassId::UninitializedStorage => "uninitialized-storage",
            PassId::IntegerOverflow => "integer-overflow",
            // Hybrid Bug Detection
            PassId::Reentrancy => "reentrancy",
            PassId::CeiViolation => "cei-violation",
            PassId::CentralizationRisk => "centralization-risk",
            // SIR Structural
            PassId::SirMissingAccessControl => "sir-missing-access-control",
            PassId::SirMissingModifies => "sir-missing-modifies",
            PassId::SirUncheckedArithmetic => "sir-unchecked-arithmetic",
            PassId::SirTxOriginAuth => "sir-tx-origin-auth",
            PassId::SirAcquiresMismatch => "sir-acquires-mismatch",
            PassId::SirMissingPdaConstraint => "sir-missing-pda-constraint",
            // ANIR Generation
            PassId::AnirGeneration => "anir-generation",
            // ANIR Analysis
            PassId::AnirTaintPropagation => "anir-taint-propagation",
            PassId::AnirReentrancy => "anir-reentrancy",
            PassId::AnirAccessControl => "anir-access-control",
            PassId::AnirArithmetic => "anir-arithmetic",
            PassId::AnirStorageAliasing => "anir-storage-aliasing",
            PassId::AnirResourceLinearity => "anir-resource-linearity",
            PassId::AnirPdaValidation => "anir-pda-validation",
        }
    }

    /// Check if this is an IR-based pass (requires IR generation).
    pub fn requires_ir(&self) -> bool {
        matches!(
            self,
            PassId::IrGeneration
                | PassId::IrCfg
                | PassId::IrCallGraph
                | PassId::DominatorTree
                | PassId::LoopAnalysis
                | PassId::SsaConstruction
                | PassId::DefUseChain
                | PassId::LivenessAnalysis
                | PassId::TaintAnalysis
                | PassId::IrStateMutation
                // SIR structural passes operate on SIR (which is "IR")
                | PassId::SirMissingAccessControl
                | PassId::SirMissingModifies
                | PassId::SirUncheckedArithmetic
                | PassId::SirTxOriginAuth
                | PassId::SirAcquiresMismatch
                | PassId::SirMissingPdaConstraint
                // ANIR generation needs SIR
                | PassId::AnirGeneration
        )
    }

    /// Check if this is an AIR-based pass (requires ANIR generation).
    pub fn requires_air(&self) -> bool {
        matches!(
            self,
            PassId::AnirTaintPropagation
                | PassId::AnirReentrancy
                | PassId::AnirAccessControl
                | PassId::AnirArithmetic
                | PassId::AnirStorageAliasing
                | PassId::AnirResourceLinearity
                | PassId::AnirPdaValidation
        )
    }

    /// Check if this is a hybrid pass (uses both AST and IR).
    pub fn is_hybrid(&self) -> bool {
        matches!(self, PassId::AccessControl | PassId::StateConsistency)
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
            "symbol-table" => PassId::SymbolTable,
            "type-index" => PassId::TypeIndex,
            "syntax-analysis" => PassId::SyntaxAnalysis,
            "inheritance-graph" => PassId::InheritanceGraph,
            "storage-layout" => PassId::StorageLayout,
            "event-analysis" => PassId::EventAnalysis,
            "call-graph" => PassId::CallGraph,
            "modifier-analysis" => PassId::ModifierAnalysis,
            "ir-generation" => PassId::IrGeneration,
            "ir-cfg" => PassId::IrCfg,
            "ir-call-graph" => PassId::IrCallGraph,
            "dominator-tree" => PassId::DominatorTree,
            "loop-analysis" => PassId::LoopAnalysis,
            "ssa-construction" => PassId::SsaConstruction,
            "def-use-chain" => PassId::DefUseChain,
            "liveness-analysis" => PassId::LivenessAnalysis,
            "taint-analysis" => PassId::TaintAnalysis,
            "ir-state-mutation" => PassId::IrStateMutation,
            "access-control" => PassId::AccessControl,
            "state-consistency" => PassId::StateConsistency,
            "cfg" => PassId::Cfg,
            "data-flow" => PassId::DataFlow,
            "state-mutation" => PassId::StateMutation,
            // AST Bug Detection
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
            // IR Bug Detection
            "dead-code" => PassId::DeadCode,
            "uninitialized-storage" => PassId::UninitializedStorage,
            "integer-overflow" => PassId::IntegerOverflow,
            // Hybrid Bug Detection
            "reentrancy" => PassId::Reentrancy,
            "cei-violation" => PassId::CeiViolation,
            "centralization-risk" => PassId::CentralizationRisk,
            // SIR Structural
            "sir-missing-access-control" => PassId::SirMissingAccessControl,
            "sir-missing-modifies" => PassId::SirMissingModifies,
            "sir-unchecked-arithmetic" => PassId::SirUncheckedArithmetic,
            "sir-tx-origin-auth" => PassId::SirTxOriginAuth,
            "sir-acquires-mismatch" => PassId::SirAcquiresMismatch,
            "sir-missing-pda-constraint" => PassId::SirMissingPdaConstraint,
            // ANIR Generation
            "anir-generation" => PassId::AnirGeneration,
            // ANIR Analysis
            "anir-taint-propagation" => PassId::AnirTaintPropagation,
            "anir-reentrancy" => PassId::AnirReentrancy,
            "anir-access-control" => PassId::AnirAccessControl,
            "anir-arithmetic" => PassId::AnirArithmetic,
            "anir-storage-aliasing" => PassId::AnirStorageAliasing,
            "anir-resource-linearity" => PassId::AnirResourceLinearity,
            "anir-pda-validation" => PassId::AnirPdaValidation,
            _ => panic!("Unknown pass ID: {}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pass_id_display() {
        assert_eq!(PassId::SymbolTable.to_string(), "symbol-table");
        assert_eq!(PassId::IrGeneration.to_string(), "ir-generation");
    }

    #[test]
    fn test_pass_id_from_str() {
        assert_eq!(PassId::from("symbol-table"), PassId::SymbolTable);
        assert_eq!(PassId::from("ir-generation"), PassId::IrGeneration);
    }

    #[test]
    fn test_requires_ir() {
        assert!(!PassId::SymbolTable.requires_ir());
        assert!(PassId::IrCfg.requires_ir());
        assert!(PassId::TaintAnalysis.requires_ir());
        // SIR structural passes require IR
        assert!(PassId::SirMissingAccessControl.requires_ir());
        assert!(PassId::SirUncheckedArithmetic.requires_ir());
        assert!(PassId::AnirGeneration.requires_ir());
    }

    #[test]
    fn test_requires_air() {
        assert!(!PassId::SymbolTable.requires_air());
        assert!(!PassId::SirMissingAccessControl.requires_air());
        assert!(PassId::AnirTaintPropagation.requires_air());
        assert!(PassId::AnirReentrancy.requires_air());
        assert!(PassId::AnirAccessControl.requires_air());
        assert!(PassId::AnirArithmetic.requires_air());
    }

    #[test]
    fn test_is_hybrid() {
        assert!(!PassId::SymbolTable.is_hybrid());
        assert!(!PassId::IrCfg.is_hybrid());
        assert!(PassId::AccessControl.is_hybrid());
    }

    #[test]
    fn test_sir_anir_roundtrip() {
        assert_eq!(PassId::from("sir-missing-access-control"), PassId::SirMissingAccessControl);
        assert_eq!(PassId::from("anir-generation"), PassId::AnirGeneration);
        assert_eq!(PassId::from("anir-taint-propagation"), PassId::AnirTaintPropagation);
        assert_eq!(PassId::AnirPdaValidation.as_str(), "anir-pda-validation");
    }
}
