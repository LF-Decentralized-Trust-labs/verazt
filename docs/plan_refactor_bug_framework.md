# Plan: Refactor SmartHunt to LLVM-Inspired Pass-Based Architecture

## Executive Summary

This document outlines the plan to refactor the SmartHunt bug detection framework from its current monolithic detector pattern to a unified **LLVM-inspired pass-based architecture operating on both AST and IR representations**.

**Key Architectural Decision**: Analysis infrastructure will be moved to the **`solidity` crate** to enable reusability across multiple tools (smarthunt for bug detection, smartproof for verification, and future tools). Bug detection passes remain in smarthunt as specialized vulnerability detectors.

The new design will:
1. **Shared Analysis Framework** (in `solidity` crate): IR generation, symbol tables, CFG, data flow, taint analysis, etc.
2. **Bug Detection Passes** (in `smarthunt` crate): Specialized vulnerability detectors that consume analysis results
3. Organize passes at different granularity levels (contract, function, block, statement, expression, variable)
4. Operate on both high-level AST and low-level IR representations
5. Enable parallel execution and reduce redundant traversals
6. Improve extensibility by leveraging the strengths of both representations

**Note**: IR generation in the solidity crate is currently **in development** and not yet complete. The plan accounts for completing IR infrastructure as part of the migration.

---

## Table of Contents

1. [Current Architecture Analysis](#1-current-architecture-analysis)
2. [Crate Organization & Responsibility](#2-crate-organization--responsibility)
3. [LLVM Pass Architecture Inspiration](#3-llvm-pass-architecture-inspiration)
4. [AST vs IR: Complementary Representations](#4-ast-vs-ir-complementary-representations)
5. [Proposed Unified Pass-Based Architecture](#5-proposed-unified-pass-based-architecture)
6. [Pass Hierarchy & Levels](#6-pass-hierarchy--levels)
7. [Pass Types & Classification](#7-pass-types--classification)
8. [Core Engine Design](#8-core-engine-design)
9. [Migration Strategy](#9-migration-strategy)
10. [Implementation Plan](#10-implementation-plan)
11. [Benefits & Trade-offs](#11-benefits--trade-offs)
12. [Future Extensions](#12-future-extensions)

---

## 1. Current Architecture Analysis

### 1.1 Existing Structure

**Components:**
- **16 bug detectors** implementing `Detector` trait
- **7 analysis passes** (SymbolTable, TypeIndex, CFG, CallGraph, DataFlow, StateMutation, AccessControl)
- **PassScheduler** for executing analysis passes with dependency resolution
- **DetectorRegistry** for managing detectors
- **Parallel execution** using Rayon for detectors (not passes)

**Execution Flow:**
```
Parse AST → Execute Analysis Passes (Sequential) → Run Detectors (Parallel) → Generate Report
```

### 1.2 Key Issues

1. **Monolithic Detector Pattern**: Each detector independently traverses the entire AST
2. **Multiple AST Traversals**: AST is traversed 16 times (once per detector)
3. **No Unified Pass System**: Detectors and analysis passes are separate concepts
4. **Sequential Pass Execution**: Analysis passes run sequentially even when parallelizable
5. **Inefficient Resource Usage**: Redundant computation across detectors
6. **Tight Coupling**: Detectors directly access AST structure, making AST changes expensive
7. **Limited Composability**: Hard to share intermediate results between detectors

### 1.3 What Works Well (Keep)

- ✅ **Dependency tracking** via `required_passes()`
- ✅ **Topological sorting** in PassScheduler
- ✅ **AnalysisContext** as central artifact storage
- ✅ **Parallel detector execution** infrastructure (Rayon)
- ✅ **DetectorRegistry** pattern for discovery
- ✅ **Bug reporting** with CWE/SWC mappings

---

## 2. Crate Organization & Responsibility

### 2.1 Motivation for Crate Separation

**Problem**: Currently, analysis infrastructure is tightly coupled with bug detection in smarthunt, making it impossible to reuse analysis capabilities in other tools like smartproof (verification) or future analysis tools.

**Solution**: Split responsibilities across crates with clear boundaries:
- **`solidity` crate**: Provides general-purpose analysis infrastructure
- **`smarthunt` crate**: Consumes analysis infrastructure for bug detection
- **`smartproof` crate**: Consumes analysis infrastructure for formal verification
- **Future tools**: Can easily leverage the shared analysis framework

### 2.2 Crate Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      solidity Crate                             │
│  (Analysis Framework Provider)                                  │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ AST Representation                                        │ │
│  │  - Parser                                                 │ │
│  │  - AST types                                              │ │
│  │  - Visitor traits                                         │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ IR Representation (In Development)                        │ │
│  │  - SmartIR types                                          │ │
│  │  - IR builder                                             │ │
│  │  - IR utilities                                           │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ Analysis Pass Infrastructure                              │ │
│  │  - Pass, AnalysisPass traits                              │ │
│  │  - PassManager                                            │ │
│  │  - PassScheduler                                          │ │
│  │  - AnalysisContext                                        │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ Analysis Passes (AST, IR, Hybrid)                         │ │
│  │  - SymbolTable, TypeIndex, Inheritance                    │ │
│  │  - IrGeneration, IrCfg, SSA, DefUse                       │ │
│  │  - Taint, CallGraph, AccessControl                        │ │
│  │  - ... (all analysis passes)                              │ │
│  └───────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                           ▲                    ▲
                           │                    │
         ┌─────────────────┴────────┐    ┌─────┴──────────────┐
         │                          │    │                    │
┌────────▼──────────┐    ┌──────────▼────▼─────┐    ┌────────▼────────┐
│  smarthunt Crate  │    │  smartproof Crate   │    │  Future Tools   │
│  (Bug Detection)  │    │  (Verification)     │    │                 │
│                   │    │                     │    │                 │
│  - BugDetection   │    │  - Verification     │    │  - Custom       │
│    Pass trait     │    │    Pass trait       │    │    Analysis     │
│  - Detector       │    │  - Proof            │    │                 │
│    passes         │    │    strategies       │    │                 │
│  - Bug reporting  │    │  - SMT integration  │    │                 │
└───────────────────┘    └─────────────────────┘    └─────────────────┘
```

### 2.3 Responsibility Breakdown

#### `solidity` Crate (Analysis Framework)

**Purpose**: Provide reusable analysis infrastructure for Solidity code

**Responsibilities**:
1. **AST Representation**
   - Parse Solidity source → AST
   - AST types and visitor patterns
   - AST utilities

2. **IR Representation** (In Development)
   - AST → SmartIR transformation
   - IR types and builder
   - IR validation

3. **Pass Infrastructure**
   - Pass traits (`Pass`, `AnalysisPass`)
   - PassManager (scheduling, execution, caching)
   - AnalysisContext (unified storage for analysis results)
   - Parallel execution engine

4. **Analysis Passes**
   - **AST Analysis**: SymbolTable, TypeIndex, Inheritance, StorageLayout, EventAnalysis, CallGraph, ModifierAnalysis
   - **IR Generation**: IrGenerationPass (AST → IR)
   - **IR Analysis**: IrCfg, SsaConstruction, DefUseChain, DominatorTree, LoopAnalysis, Liveness, TaintAnalysis, IrStateMutation, IrCallGraph
   - **Hybrid Analysis**: AccessControl, StateConsistency

**Public API**:
```rust
// In solidity crate
pub mod analysis {
    pub mod pass;           // Pass traits
    pub mod context;        // AnalysisContext
    pub mod manager;        // PassManager

    pub mod passes {
        pub mod ast;        // AST analysis passes
        pub mod ir;         // IR analysis passes (when IR ready)
        pub mod hybrid;     // Hybrid passes
    }
}
```

#### `smarthunt` Crate (Bug Detection)

**Purpose**: Specialized vulnerability detection using analysis infrastructure

**Responsibilities**:
1. **Bug Detection Pass Trait**
   - Extends `Pass` trait from solidity crate
   - Adds bug-specific metadata (CWE, SWC, risk level)

2. **Detector Passes**
   - AST-based detectors: TxOrigin, UncheckedCall, Visibility, Shadowing, etc.
   - IR-based detectors: DeadCode, UninitializedStorage, IntegerOverflow, etc.
   - Hybrid detectors: Reentrancy, CEIViolation, CentralizationRisk, etc.

3. **Bug Reporting**
   - Bug types and formatting
   - Report generation (JSON, SARIF, Markdown)
   - Severity classification

4. **CLI & Integration**
   - Command-line interface
   - Configuration management
   - Output formatting

**Dependencies**:
```toml
[dependencies]
solidity = { path = "../solidity" }  # Uses analysis framework
```

**Public API**:
```rust
// In smarthunt crate
pub mod detection {
    pub mod pass;           // BugDetectionPass trait
    pub mod detectors {
        pub mod ast;        // AST-based detectors
        pub mod ir;         // IR-based detectors
        pub mod hybrid;     // Hybrid detectors
    }
}

pub mod report;             // Bug reporting
pub mod cli;                // Command-line interface
```

#### `smartproof` Crate (Formal Verification)

**Purpose**: Formal verification using analysis infrastructure

**Responsibilities**:
1. Verification strategies using analysis passes
2. SMT solver integration using IR
3. Proof generation and checking

**Dependencies**:
```toml
[dependencies]
solidity = { path = "../solidity" }  # Uses analysis framework
```

### 2.4 Benefits of This Organization

**Reusability**:
- ✅ Analysis passes written once, used by multiple tools
- ✅ IR generation amortized across smarthunt, smartproof, and future tools
- ✅ Consistent analysis results across all consumers

**Maintainability**:
- ✅ Clear separation of concerns (analysis vs application)
- ✅ Analysis improvements benefit all tools automatically
- ✅ Bug detection logic isolated from analysis infrastructure

**Development Efficiency**:
- ✅ Can develop smarthunt detectors independently from analysis passes
- ✅ Parallel development: IR infrastructure and bug detectors
- ✅ Testing: Analysis passes tested independently from detectors

**Extensibility**:
- ✅ New tools can easily leverage analysis framework
- ✅ Third-party tools can depend on solidity crate
- ✅ Analysis passes can be contributed to solidity crate

### 2.5 Migration Implications

**Key Insight**: This crate organization changes where code lives, but not the overall architecture. The pass-based design remains the same.

**Migration Strategy**:
1. First build pass infrastructure in `solidity` crate
2. Implement analysis passes in `solidity` crate
3. Create `BugDetectionPass` trait in `smarthunt` crate (extends solidity's `Pass`)
4. Implement detectors in `smarthunt` crate using analysis from solidity

**Backward Compatibility**:
- Existing smarthunt code temporarily remains
- New code goes to appropriate crate
- Gradual migration with compatibility layer

---

## 3. LLVM Pass Architecture Inspiration

### 2.1 LLVM Pass Concept

LLVM organizes optimizations and analyses as **passes** that:
- Operate at specific granularity levels (Module, Function, Loop, BasicBlock)
- Declare dependencies explicitly
- Run in optimized order determined by a pass manager
- Can be composed and reused
- Cache results for dependent passes

### 2.2 Key LLVM Principles to Adopt

1. **Granularity Levels**: Passes operate at different AST levels
2. **Pass Manager**: Central orchestrator for scheduling and execution
3. **Analysis vs Transformation**: Separate analysis (read-only) from transformation (mutating)
4. **Pass Dependencies**: Explicit declaration of required analyses
5. **Result Caching**: Store and reuse pass results
6. **Pipeline Composition**: Build analysis pipelines declaratively

### 2.3 Adaptation to Bug Detection

Unlike LLVM's optimization focus, SmartHunt needs:
- **Analysis passes** to collect information (similar to LLVM)
- **Bug detection passes** instead of transformation passes
- **Immutable AST** (no transformations, only analysis)
- **Parallel execution** where dependencies allow
- **Incremental analysis** for IDE integration (future)

---

## 3. AST vs IR: Complementary Representations

### 3.1 The Case for Dual Representation

Bug detection requires analyzing smart contracts at different abstraction levels. Both Abstract Syntax Tree (AST) and Intermediate Representation (IR) offer unique advantages:

**AST Advantages**:
- **Preserves Source Structure**: Maintains original code organization, variable names, comments
- **High-Level Semantics**: Directly represents Solidity constructs (modifiers, inheritance, events)
- **User-Facing Diagnostics**: Bug locations map naturally to source code
- **Pattern Matching**: Easy to detect syntactic patterns (tx.origin, unchecked calls)
- **Visibility Analysis**: Access modifiers, function visibility directly available

**IR Advantages**:
- **Normalized Form**: Eliminates syntactic sugar, makes analysis simpler
- **Control Flow Explicit**: CFG construction is straightforward
- **Data Flow Analysis**: SSA form enables precise def-use chains
- **Optimization-Ready**: Transformations are easier on IR
- **Low-Level Bugs**: Detects issues obscured by high-level syntax (uninitialized storage, reentrancy patterns)

**Key Insight**: Some bugs are **naturally expressed** at one level but **hard to detect** at the other:

| Bug Type | Best Representation | Why |
|----------|---------------------|-----|
| tx.origin usage | AST | Direct syntactic pattern |
| Unchecked call returns | AST | High-level statement analysis |
| Visibility issues | AST | Visibility modifiers in AST |
| Floating pragma | AST | Pragma directives in AST |
| Reentrancy | IR | Requires precise control flow + state tracking |
| Uninitialized storage | IR | SSA form reveals missing initialization |
| Integer overflow | IR | Arithmetic operations normalized in IR |
| Dead code | IR | CFG-based reachability analysis |
| CEI violations | Both | High-level pattern (AST) + precise flow (IR) |
| Shadowing | AST | Name resolution at source level |
| Access control | Both | Modifier analysis (AST) + flow analysis (IR) |

### 3.2 Unified Analysis Strategy

**Three Categories of Bug Detection**:

1. **AST-Only Detection**: Bugs detectable from source structure alone
   - Pattern-based detectors (tx.origin, deprecated features)
   - Syntactic issues (visibility, pragma)
   - High-level semantic issues (shadowing, inheritance)

2. **IR-Only Detection**: Bugs requiring low-level flow analysis
   - Data flow bugs (uninitialized variables, use-after-free)
   - Control flow bugs (dead code, unreachable paths)
   - Optimization-level bugs (redundant operations)

3. **Hybrid Detection**: Bugs requiring both representations
   - **AST for structure + IR for flow**: Reentrancy (modifiers from AST, flow from IR)
   - **AST for diagnostics + IR for precision**: Access control (understand roles from AST, track flows in IR)
   - **Cross-validation**: Detect inconsistencies between high-level intent and low-level behavior

### 3.3 IR Generation in the Pass Pipeline

**IR as a Pass Result**:
```
AST → [IR Generation Pass] → IR → [IR-based Analysis Passes] → Results
                           ↓
                      [AST-based Analysis Passes]
```

**Key Design Decisions**:
1. **IR Generation is an Analysis Pass**: Produces IR from AST, stores in context
2. **AST Always Available**: Never discard AST; both representations coexist
3. **Lazy IR Generation**: Only generate IR if IR-based passes are enabled
4. **Pass Dependencies**: IR-based passes declare `IrGeneration` as dependency

### 3.4 Representation-Aware Pass System

**Pass Representation Types**:
```rust
enum PassRepresentation {
    Ast,     // Operates on AST only
    Ir,      // Operates on IR only
    Hybrid,  // Uses both AST and IR
}

trait Pass {
    fn representation(&self) -> PassRepresentation;
    // ... other methods
}
```

**Benefits**:
- **Explicit**: Each pass declares what it operates on
- **Optimizable**: Engine can schedule AST and IR passes separately
- **Flexible**: Hybrid passes can leverage both representations

---

## 4. Proposed Unified Pass-Based Architecture

### 4.1 Core Concepts

#### Pass Definition
A **pass** is a self-contained analysis unit that:
- Operates at a specific granularity level (contract, function, block, statement, expression, variable)
- Operates on a specific representation (AST, IR, or both)
- Declares its dependencies explicitly
- Produces artifacts stored in `AnalysisContext`
- Can be an analysis pass (collect data) or bug detection pass (find vulnerabilities)

#### Unified Pass Manager
The **PassManager** (enhanced PassScheduler):
- Registers all passes (AST-based, IR-based, hybrid, analysis, detection)
- Computes execution schedule respecting dependencies and representations
- Orchestrates IR generation when needed
- Executes passes in parallel where possible
- Caches results in `AnalysisContext`
- Tracks pass completion and timing

### 4.2 Architecture Diagram

```
┌────────────────────────────────────────────────────────────────────────────┐
│                         Unified PassManager                                │
│  - Pass Registration (AST/IR/Hybrid)                                       │
│  - Dependency Resolution                                                   │
│  - Representation-Aware Scheduling                                         │
│  - Parallel Execution Orchestration                                        │
└────────────────────────┬───────────────────────────────────────────────────┘
                         │
         ┌───────────────┴───────────────┐
         │                               │
    ┌────▼────────┐                ┌────▼─────────┐
    │  Analysis   │                │ Bug Detection│
    │   Passes    │                │    Passes    │
    │  (Phase 1)  │                │  (Phase 2)   │
    └─────┬───────┘                └──────┬───────┘
          │                               │
    ┌─────┴──────┬─────────┬──────┐      │
    │            │         │      │      │
┌───▼───┐  ┌────▼───┐ ┌───▼──┐ ┌─▼──────▼──────┐
│  AST  │  │   IR   │ │Hybrid│ │   AST / IR    │
│ Passes│  │ Passes │ │Passes│ │    / Hybrid   │
└───┬───┘  └────┬───┘ └───┬──┘ └───────┬───────┘
    │           │         │            │
    │      ┌────▼─────┐   │            │
    │      │    IR    │◄──┘            │
    │      │Generation│                │
    │      │  Pass    │                │
    │      └────┬─────┘                │
    │           │                      │
    └───────────┴──────────────────────┘
                │
                ▼
┌───────────────────────────────────────────────────┐
│            AnalysisContext                        │
│  - AST (immutable)                                │
│  - IR (optional, immutable)                       │
│  - Analysis Artifacts (from both AST and IR)      │
│  - Bug Reports                                    │
└───────────────────────────────────────────────────┘
```

### 4.3 Three-Phase Execution Model

**Phase 0: IR Generation** (Conditional)
- If any pass requires IR, generate IR from AST
- `IrGenerationPass` transforms AST → IR
- IR stored in AnalysisContext
- Otherwise, skip (optimize for AST-only analysis)

**Phase 1: Analysis Passes** (Information Gathering)
- Run AST-based analysis passes (symbol tables, inheritance)
- Run IR-based analysis passes (CFGs, data flow, SSA)
- Run hybrid analysis passes (require both)
- Execute in parallel where dependencies allow
- Store results in AnalysisContext

**Phase 2: Bug Detection Passes** (Vulnerability Detection)
- Run AST-based bug detection passes (pattern matching)
- Run IR-based bug detection passes (flow-sensitive)
- Run hybrid bug detection passes (comprehensive)
- Execute in parallel (most bug detections are independent)
- Produce bug reports with source-level locations

---

## 5. Pass Hierarchy & Levels

### 5.1 Dual Representation Hierarchy

The pass system operates on two parallel hierarchies:

**AST Hierarchy** (High-Level, Source-Oriented):
```
Program → SourceUnit → Contract → Function → Statement → Expression → Variable
```

**IR Hierarchy** (Low-Level, Execution-Oriented):
```
Module → Contract → Function → BasicBlock → Instruction → Operand
```

Both hierarchies represent the same program at different abstraction levels, and passes can operate at corresponding levels in either representation.

### 5.2 Granularity Levels

Each pass operates at one or more levels of the AST or IR hierarchy:

```
┌─────────────────────────────────────────────────┐
│ Level 0: Program Level (Multi-Contract)        │
│  - Cross-contract analysis                      │
│  - Library dependencies                         │
└────────────────┬────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────┐
│ Level 1: Contract Level                         │
│  - Contract-wide properties                     │
│  - State variables                              │
│  - Inheritance analysis                         │
└────────────────┬────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────┐
│ Level 2: Function Level                         │
│  - Function properties                          │
│  - Control flow graphs                          │
│  - Parameter analysis                           │
└────────────────┬────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────┐
│ Level 3: Block Level                            │
│  - Basic blocks                                 │
│  - Loop analysis                                │
│  - Branch analysis                              │
└────────────────┬────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────┐
│ Level 4: Statement Level                        │
│  - Individual statements                        │
│  - State modifications                          │
│  - Call sites                                   │
└────────────────┬────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────┐
│ Level 5: Expression Level                       │
│  - Expression evaluation                        │
│  - Operator usage                               │
│  - Value flow                                   │
└────────────────┬────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────┐
│ Level 6: Variable Level                         │
│  - Variable usage                               │
│  - Def-use chains                               │
│  - Taint tracking                               │
└─────────────────────────────────────────────────┘
```

### 5.3 Pass Level and Representation Specification

Each pass declares its operating level(s) and representation:

```rust
/// Granularity level (applies to both AST and IR)
enum PassLevel {
    Program,      // Multi-contract analysis (AST: SourceUnit, IR: Module)
    Contract,     // Per-contract analysis (AST: ContractDefinition, IR: Contract)
    Function,     // Per-function analysis (AST: FunctionDefinition, IR: Function)
    Block,        // Per-block analysis (AST: Block, IR: BasicBlock)
    Statement,    // Per-statement analysis (AST: Statement, IR: Instruction)
    Expression,   // Per-expression analysis (AST: Expression, IR: Operand)
    Variable,     // Per-variable analysis (AST: VariableDeclaration, IR: SSA Variable)
}

/// Representation the pass operates on
enum PassRepresentation {
    Ast,     // Operates on AST only
    Ir,      // Operates on IR only (requires IrGeneration pass)
    Hybrid,  // Operates on both AST and IR
}

trait Pass {
    fn level(&self) -> PassLevel;
    fn levels(&self) -> Vec<PassLevel> { vec![self.level()] }  // Support multiple levels
    fn representation(&self) -> PassRepresentation;
}
```

**Examples**:
- `SymbolTablePass`: `representation = Ast, level = Contract`
- `SsaConstructionPass`: `representation = Ir, level = Function`
- `ReentrancyDetectionPass`: `representation = Hybrid, level = Function`

### 5.4 Level-Based Optimization Benefits

1. **Efficient Traversal**: Traverse AST/IR once per level, not once per detector
2. **Representation Isolation**: AST and IR passes can run independently
3. **Shared Context**: Passes at same level and representation can share traversal state
4. **Parallel Execution**: Independent passes at same level run in parallel
5. **Lazy IR Generation**: IR only generated if IR-based passes are enabled
6. **Incremental Updates**: Changes at one level only invalidate that level's passes (future)

---

## 6. Pass Types & Classification

### 6.1 Pass Type Hierarchy

```rust
trait Pass: Send + Sync {
    fn id(&self) -> PassId;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn level(&self) -> PassLevel;
    fn representation(&self) -> PassRepresentation;
    fn required_passes(&self) -> Vec<PassId>;
    fn invalidates(&self) -> Vec<PassId> { vec![] }  // For transformation passes (future)
}

trait AnalysisPass: Pass {
    fn run_analysis(&self, context: &mut AnalysisContext) -> Result<(), AnalysisError>;
}

trait BugDetectionPass: Pass {
    fn detect_bugs(&self, context: &AnalysisContext) -> Vec<Bug>;
    fn bug_kind(&self) -> BugKind;
    fn risk_level(&self) -> RiskLevel;
    fn confidence(&self) -> ConfidenceLevel;
    fn cwe_ids(&self) -> Vec<usize>;
    fn swc_ids(&self) -> Vec<usize>;
}
```

### 6.2 Analysis Pass Categories by Representation

#### A. IR Generation Pass (Foundation for IR-based analysis)

**`IrGenerationPass`** (Representation: Ast→Ir, Level: All)
- **Dependencies**: SymbolTable
- **Produces**: IR representation (SmartIR) from AST
- **Details**: Transforms high-level Solidity AST to low-level IR
- **Lazy Execution**: Only runs if IR-based passes are registered

#### B. AST-Based Analysis Passes

These passes operate solely on the AST and don't require IR:

**Category 1: Context Building** (No dependencies)
- `SymbolTablePass` (Ast, Contract + Function level)
  - Builds symbol table for name resolution
  - Required by almost all other passes

- `TypeIndexPass` (Ast, Contract level)
  - Indexes type definitions and relationships
  - Required for type checking and storage layout

- `SyntaxAnalysisPass` (Ast, All levels)
  - Basic AST structure validation
  - Detects malformed syntax patterns

**Category 2: Contract-Level AST Analysis**
- `InheritanceGraphPass` (Ast, Contract level) - Requires: SymbolTable
  - Builds inheritance hierarchy
  - Detects linearization issues

- `StorageLayoutPass` (Ast, Contract level) - Requires: TypeIndex
  - Computes storage slot assignments
  - Detects storage collisions

- `EventAnalysisPass` (Ast, Contract level) - Requires: SymbolTable
  - Analyzes event definitions and emissions
  - Tracks event signatures

**Category 3: Interprocedural AST Analysis**
- `CallGraphPass` (Ast, Function level) - Requires: SymbolTable
  - Builds function call relationships
  - Identifies call sites and targets

- `ModifierAnalysisPass` (Ast, Function level) - Requires: SymbolTable
  - Analyzes modifier usage and semantics
  - Tracks modifier call chains

#### C. IR-Based Analysis Passes

These passes operate on IR and require `IrGenerationPass`:

**Category 1: IR Control Flow Analysis**
- `IrCfgPass` (Ir, Function + Block level) - Requires: IrGeneration
  - Builds precise control flow graphs from IR
  - Produces basic blocks and edges

- `DominatorTreePass` (Ir, Block level) - Requires: IrCfg
  - Computes dominator tree for each function
  - Enables advanced flow analysis

- `LoopAnalysisPass` (Ir, Block level) - Requires: IrCfg, DominatorTree
  - Identifies natural loops
  - Computes loop nesting and headers

**Category 2: IR Data Flow Analysis**
- `SsaConstructionPass` (Ir, Variable level) - Requires: IrCfg
  - Converts to SSA form
  - Inserts phi nodes at join points

- `DefUseChainPass` (Ir, Variable level) - Requires: SsaConstruction
  - Builds def-use and use-def chains
  - Enables precise data flow tracking

- `LivenessAnalysisPass` (Ir, Variable level) - Requires: IrCfg
  - Computes live variable ranges
  - Identifies dead stores

- `TaintAnalysisPass` (Ir, Variable level) - Requires: DefUseChain, IrCfg
  - Tracks data flow from sources to sinks
  - Identifies tainted values

**Category 3: IR Interprocedural Analysis**
- `IrCallGraphPass` (Ir, Function level) - Requires: IrGeneration
  - Builds call graph from IR
  - Resolves indirect calls more precisely

- `IrStateMutationPass` (Ir, Function level) - Requires: IrCfg, TaintAnalysis
  - Tracks state variable reads/writes at IR level
  - More precise than AST-based version

#### D. Hybrid Analysis Passes (AST + IR)

These passes use both representations for comprehensive analysis:

**`AccessControlPass`** (Hybrid, Function level)
- **Dependencies**: SymbolTable (AST), IrCfg, IrStateMutation
- **AST Usage**: Extract modifier semantics, role definitions
- **IR Usage**: Precise flow analysis for access control enforcement
- **Why Hybrid**: Modifiers are high-level (AST) but enforcement requires flow analysis (IR)

**`StateConsistencyPass`** (Hybrid, Contract level)
- **Dependencies**: StorageLayout (AST), IrStateMutation
- **AST Usage**: Storage layout and invariants
- **IR Usage**: Track all state modifications
- **Why Hybrid**: Validate high-level invariants against low-level mutations

### 6.3 Bug Detection Pass Categories by Representation

#### A. AST-Based Bug Detection Passes

These detectors operate on AST and detect bugs from source-level patterns:

**Category A1: Syntactic Pattern Detection** (AST-only, minimal dependencies)
- `TxOriginDetectionPass` (Ast, Expression level) - Requires: SymbolTable
  - Detects `tx.origin` usage in authentication
  - **Why AST**: Direct syntactic pattern in expressions

- `UncheckedCallDetectionPass` (Ast, Statement level) - Requires: SymbolTable
  - Detects unchecked low-level call return values
  - **Why AST**: Statement-level pattern matching

- `FloatingPragmaDetectionPass` (Ast, Program level) - Requires: None
  - Detects unlocked compiler versions
  - **Why AST**: Pragma directives only in AST

- `VisibilityDetectionPass` (Ast, Function level) - Requires: SymbolTable
  - Detects missing or incorrect visibility specifiers
  - **Why AST**: Visibility is AST-level metadata

- `DeprecatedDetectionPass` (Ast, Expression level) - Requires: None
  - Detects usage of deprecated Solidity features
  - **Why AST**: Deprecated constructs at syntax level

- `LowLevelCallDetectionPass` (Ast, Expression level) - Requires: SymbolTable
  - Detects potentially dangerous low-level calls
  - **Why AST**: Call expression pattern matching

- `DelegatecallDetectionPass` (Ast, Expression level) - Requires: SymbolTable
  - Detects delegatecall to untrusted contracts
  - **Why AST**: Expression-level call detection

**Category A2: Semantic AST Detection** (AST-only, requires analysis)
- `ShadowingDetectionPass` (Ast, Variable level) - Requires: SymbolTable, Inheritance
  - Detects variable shadowing across scopes and inheritance
  - **Why AST**: Name resolution at source level

- `ConstantStateVarPass` (Ast, Contract level) - Requires: SymbolTable, IrStateMutation
  - Detects state variables that should be constant
  - **Why AST Primary**: Variable declarations in AST, uses IR for write detection

- `EventMismatchDetectionPass` (Ast, Contract level) - Requires: EventAnalysis
  - Detects missing or incorrect event emissions
  - **Why AST**: Events are high-level source constructs

#### B. IR-Based Bug Detection Passes

These detectors operate on IR and detect bugs from low-level flow analysis:

**Category B1: Control Flow Detection** (IR-only)
- `DeadCodeDetectionPass` (Ir, Block level) - Requires: IrCfg, Liveness
  - Detects unreachable basic blocks
  - **Why IR**: CFG-based reachability analysis

- `UnreachableAfterRevertPass` (Ir, Block level) - Requires: IrCfg
  - Detects code after revert/require that can't execute
  - **Why IR**: Precise control flow in IR

- `InfiniteLoopDetectionPass` (Ir, Block level) - Requires: LoopAnalysis
  - Detects loops without exit conditions
  - **Why IR**: Loop analysis on IR CFG

**Category B2: Data Flow Detection** (IR-only)
- `UninitializedStorageDetectionPass` (Ir, Variable level) - Requires: SsaConstruction, DefUseChain
  - Detects reads from uninitialized storage variables
  - **Why IR**: SSA form makes def-use explicit

- `UseAfterDeleteDetectionPass` (Ir, Variable level) - Requires: DefUseChain
  - Detects use of variables after deletion
  - **Why IR**: Precise def-use tracking in SSA

- `IntegerOverflowDetectionPass` (Ir, Expression level) - Requires: IrCfg, TaintAnalysis
  - Detects potential integer overflows (pre-0.8.0)
  - **Why IR**: Arithmetic operations normalized in IR

- `DivisionByZeroDetectionPass` (Ir, Expression level) - Requires: IrCfg, TaintAnalysis
  - Detects potential division by zero
  - **Why IR**: Precise value tracking in IR

**Category B3: IR Interprocedural Detection**
- `UnusedReturnValuePass` (Ir, Function level) - Requires: IrCallGraph, DefUseChain
  - Detects functions whose return values are never used
  - **Why IR**: Precise use tracking across call sites

#### C. Hybrid Bug Detection Passes (AST + IR)

These detectors require both representations for comprehensive analysis:

**`ReentrancyDetectionPass`** (Hybrid, Function level)
- **Dependencies**: SymbolTable, ModifierAnalysis (AST), IrCfg, IrCallGraph, IrStateMutation (IR)
- **AST Usage**: Identify reentrancy guards (modifiers like `nonReentrant`)
- **IR Usage**: Precise control flow + state mutation tracking + external call detection
- **Why Hybrid**: Guards are high-level (AST), but vulnerability requires precise flow (IR)
- **Detection**: Check if state writes occur after external calls without guards

**`CEIViolationDetectionPass`** (Hybrid, Function level)
- **Dependencies**: SymbolTable (AST), IrCfg, IrStateMutation (IR)
- **AST Usage**: Understand function intent and boundaries
- **IR Usage**: Precise ordering of state changes, external calls, and effects
- **Why Hybrid**: Pattern is high-level (Checks-Effects-Interactions), but detection requires precise flow
- **Detection**: Validate state updates happen before external calls

**`CentralizationRiskPass`** (Hybrid, Contract level)
- **Dependencies**: SymbolTable, AccessControl (Hybrid), IrStateMutation
- **AST Usage**: Identify privileged functions via modifiers (onlyOwner)
- **IR Usage**: Track what state variables these functions can modify
- **Why Hybrid**: Access control is semantic (AST), risk assessment needs flow analysis (IR)
- **Detection**: Identify functions with excessive control over contract state

**`MissingAccessControlPass`** (Hybrid, Function level)
- **Dependencies**: AccessControl (Hybrid), IrStateMutation
- **AST Usage**: Check for access control modifiers
- **IR Usage**: Determine if function modifies sensitive state
- **Why Hybrid**: Modifiers are AST, but criticality assessment needs IR
- **Detection**: Functions modifying state without access control checks

**`TimestampDependencePass`** (Hybrid, Expression level)
- **Dependencies**: SymbolTable (AST), IrCfg, TaintAnalysis (IR)
- **AST Usage**: Identify `block.timestamp` usage
- **IR Usage**: Track timestamp value flow to critical operations
- **Why Hybrid**: Timestamp is AST construct, but impact requires taint analysis (IR)
- **Detection**: Timestamp used in conditions or critical computations

**`FrontRunningVulnerabilityPass`** (Hybrid, Function level)
- **Dependencies**: SymbolTable (AST), IrCfg, IrStateMutation, TaintAnalysis (IR)
- **AST Usage**: Identify public/external functions
- **IR Usage**: Track if user input affects critical state updates
- **Why Hybrid**: Function visibility is AST, but vulnerability requires data flow (IR)
- **Detection**: User-controlled inputs that directly affect valuable state

### 6.4 Unified Pass Dependency Graph

The dependency graph spans both AST and IR passes, with IR passes depending on the IR generation pass:

```
═══════════════════════════════════════════════════════════════════════
                        ANALYSIS PASSES
═══════════════════════════════════════════════════════════════════════

Level 0: Foundation (No dependencies)
┌─────────────────────────────────────────────────────────────┐
│ AST Foundation Passes                                       │
│  - SymbolTablePass [AST]                                    │
│  - TypeIndexPass [AST]                                      │
│  - SyntaxAnalysisPass [AST]                                 │
└─────────────────────────────────────────────────────────────┘

Level 1: AST Analysis + IR Generation (Depends on Level 0)
┌─────────────────────────────────────────────────────────────┐
│ AST Analysis Passes                                         │
│  - InheritanceGraphPass [AST] → SymbolTable                 │
│  - StorageLayoutPass [AST] → TypeIndex                      │
│  - EventAnalysisPass [AST] → SymbolTable                    │
│  - CallGraphPass [AST] → SymbolTable                        │
│  - ModifierAnalysisPass [AST] → SymbolTable                 │
├─────────────────────────────────────────────────────────────┤
│ IR Generation Pass (AST → IR)                               │
│  - IrGenerationPass [AST→IR] → SymbolTable                  │
│    (Only runs if IR-based passes are registered)            │
└─────────────────────────────────────────────────────────────┘

Level 2: IR Control Flow Analysis (Depends on Level 0-1)
┌─────────────────────────────────────────────────────────────┐
│ IR Control Flow Passes                                      │
│  - IrCfgPass [IR] → IrGeneration                            │
│  - IrCallGraphPass [IR] → IrGeneration                      │
└─────────────────────────────────────────────────────────────┘

Level 3: Advanced IR Analysis (Depends on Level 0-2)
┌─────────────────────────────────────────────────────────────┐
│ IR Dominator & Loop Analysis                                │
│  - DominatorTreePass [IR] → IrCfg                           │
│  - LoopAnalysisPass [IR] → IrCfg, DominatorTree             │
├─────────────────────────────────────────────────────────────┤
│ IR Data Flow Analysis                                       │
│  - SsaConstructionPass [IR] → IrCfg                         │
│  - DefUseChainPass [IR] → SsaConstruction                   │
│  - LivenessAnalysisPass [IR] → IrCfg                        │
└─────────────────────────────────────────────────────────────┘

Level 4: High-Level IR Analysis (Depends on Level 0-3)
┌─────────────────────────────────────────────────────────────┐
│ IR Interprocedural Analysis                                 │
│  - TaintAnalysisPass [IR] → DefUseChain, IrCfg              │
│  - IrStateMutationPass [IR] → IrCfg, TaintAnalysis          │
├─────────────────────────────────────────────────────────────┤
│ Hybrid Analysis (AST + IR)                                  │
│  - AccessControlPass [HYBRID] → SymbolTable, ModifierAnalysis│
│                                  IrCfg, IrStateMutation     │
│  - StateConsistencyPass [HYBRID] → StorageLayout,           │
│                                    IrStateMutation          │
└─────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════════
                     BUG DETECTION PASSES
═══════════════════════════════════════════════════════════════════════

Detection Level 0: AST Pattern Detection (Depends on Level 0-1)
┌─────────────────────────────────────────────────────────────┐
│ Syntactic Detectors [AST]                                   │
│  - TxOriginDetectionPass → SymbolTable                      │
│  - UncheckedCallDetectionPass → SymbolTable                 │
│  - FloatingPragmaDetectionPass → (none)                     │
│  - VisibilityDetectionPass → SymbolTable                    │
│  - DeprecatedDetectionPass → (none)                         │
│  - LowLevelCallDetectionPass → SymbolTable                  │
│  - DelegatecallDetectionPass → SymbolTable                  │
├─────────────────────────────────────────────────────────────┤
│ Semantic AST Detectors [AST]                                │
│  - ShadowingDetectionPass → SymbolTable, Inheritance        │
│  - EventMismatchDetectionPass → EventAnalysis               │
│  - ConstantStateVarPass → SymbolTable, IrStateMutation      │
└─────────────────────────────────────────────────────────────┘

Detection Level 1: IR Flow Detection (Depends on Level 2-4)
┌─────────────────────────────────────────────────────────────┐
│ IR Control Flow Detectors [IR]                              │
│  - DeadCodeDetectionPass → IrCfg, Liveness                  │
│  - UnreachableAfterRevertPass → IrCfg                       │
│  - InfiniteLoopDetectionPass → LoopAnalysis                 │
├─────────────────────────────────────────────────────────────┤
│ IR Data Flow Detectors [IR]                                 │
│  - UninitializedStorageDetectionPass → SsaConstruction,     │
│                                         DefUseChain         │
│  - UseAfterDeleteDetectionPass → DefUseChain                │
│  - IntegerOverflowDetectionPass → IrCfg, TaintAnalysis      │
│  - DivisionByZeroDetectionPass → IrCfg, TaintAnalysis       │
│  - UnusedReturnValuePass → IrCallGraph, DefUseChain         │
└─────────────────────────────────────────────────────────────┘

Detection Level 2: Hybrid Detection (Depends on Level 0-4)
┌─────────────────────────────────────────────────────────────┐
│ Hybrid Security Detectors [AST + IR]                        │
│  - ReentrancyDetectionPass → SymbolTable, ModifierAnalysis, │
│                               IrCfg, IrCallGraph,           │
│                               IrStateMutation               │
│  - CEIViolationDetectionPass → SymbolTable, IrCfg,          │
│                                 IrStateMutation             │
│  - CentralizationRiskPass → SymbolTable, AccessControl,     │
│                              IrStateMutation                │
│  - MissingAccessControlPass → AccessControl,                │
│                                IrStateMutation              │
│  - TimestampDependencePass → SymbolTable, IrCfg,            │
│                               TaintAnalysis                 │
│  - FrontRunningVulnerabilityPass → SymbolTable, IrCfg,      │
│                                     IrStateMutation,        │
│                                     TaintAnalysis           │
└─────────────────────────────────────────────────────────────┘
```

**Key Observations**:
1. **IR passes depend on IrGenerationPass**: All IR-based analysis requires IR generation first
2. **Hybrid passes depend on both**: Comprehensive detectors leverage both AST and IR
3. **Parallel opportunities**:
   - Level 0 AST passes can run in parallel
   - Level 1 AST passes can run in parallel with IrGenerationPass
   - Level 2 IR passes can run in parallel after IrGeneration completes
   - Most detection passes can run in parallel within their detection level
4. **Lazy IR generation**: If no IR-based passes are enabled, skip IR generation entirely

---

## 7. Core Engine Design

### 7.1 Unified PassManager Architecture

The PassManager orchestrates execution across both AST and IR representations:

```rust
struct PassManager {
    // Pass registration (organized by representation)
    ast_analysis_passes: HashMap<PassId, Box<dyn AnalysisPass>>,
    ir_analysis_passes: HashMap<PassId, Box<dyn AnalysisPass>>,
    hybrid_analysis_passes: HashMap<PassId, Box<dyn AnalysisPass>>,

    ast_detection_passes: HashMap<PassId, Box<dyn BugDetectionPass>>,
    ir_detection_passes: HashMap<PassId, Box<dyn BugDetectionPass>>,
    hybrid_detection_passes: HashMap<PassId, Box<dyn BugDetectionPass>>,

    // Special: IR generation pass
    ir_generation_pass: Option<Box<dyn AnalysisPass>>,

    // Dependency graph (unified across all passes)
    dependency_graph: DependencyGraph,

    // Scheduling
    scheduler: RepresentationAwareScheduler,

    // Execution
    executor: PassExecutor,

    // Configuration
    config: PassManagerConfig,
}

struct PassManagerConfig {
    enable_parallel: bool,
    max_parallelism: usize,
    enable_caching: bool,
    enable_timing: bool,

    // IR-specific configuration
    enable_ir_generation: bool,  // Auto-detected based on registered passes
    lazy_ir_generation: bool,    // Only generate IR if IR passes are enabled
}
```

### 7.2 Representation-Aware Pass Registration

```rust
impl PassManager {
    pub fn new(config: PassManagerConfig) -> Self { /* ... */ }

    // Register analysis passes (automatically routes to correct map based on representation)
    pub fn register_analysis_pass(&mut self, pass: Box<dyn AnalysisPass>) {
        let id = pass.id();
        let repr = pass.representation();
        self.dependency_graph.add_node(id, pass.required_passes());

        match repr {
            PassRepresentation::Ast => {
                self.ast_analysis_passes.insert(id, pass);
            }
            PassRepresentation::Ir => {
                self.ir_analysis_passes.insert(id, pass);
                // Mark that IR generation is needed
                self.config.enable_ir_generation = true;
            }
            PassRepresentation::Hybrid => {
                self.hybrid_analysis_passes.insert(id, pass);
                // Hybrid passes also require IR
                self.config.enable_ir_generation = true;
            }
        }
    }

    // Register bug detection passes
    pub fn register_bug_detection_pass(&mut self, pass: Box<dyn BugDetectionPass>) {
        let id = pass.id();
        let repr = pass.representation();
        self.dependency_graph.add_node(id, pass.required_passes());

        match repr {
            PassRepresentation::Ast => {
                self.ast_detection_passes.insert(id, pass);
            }
            PassRepresentation::Ir => {
                self.ir_detection_passes.insert(id, pass);
                self.config.enable_ir_generation = true;
            }
            PassRepresentation::Hybrid => {
                self.hybrid_detection_passes.insert(id, pass);
                self.config.enable_ir_generation = true;
            }
        }
    }

    // Register IR generation pass (called automatically during default registration)
    pub fn register_ir_generation_pass(&mut self, pass: Box<dyn AnalysisPass>) {
        self.ir_generation_pass = Some(pass);
    }

    // Batch registration from registry
    pub fn register_default_passes(&mut self) {
        // Register all built-in AST, IR, and hybrid passes
        // IR generation pass is registered but only executed if needed
    }
}
```

### 7.3 Representation-Aware Scheduling Algorithm

**Goal**: Execute passes in parallel while respecting dependencies and representation boundaries

**Enhanced Algorithm**:
1. **Representation Detection**: Identify which representations are needed (AST-only vs AST+IR)
2. **IR Generation Scheduling**: If IR needed, schedule IR generation after AST foundation passes
3. **Dependency Resolution**: Topological sort considering representation dependencies
4. **Level Assignment**: Group passes into execution levels respecting representation barriers
5. **Parallelization**: Within each level and representation, execute independent passes in parallel
6. **Phase Separation**: Run all analysis passes before bug detection passes

**Key Insight**: AST and IR passes can run in parallel once IR is generated, maximizing throughput.

**Pseudocode**:
```python
function schedule_passes(pass_manager, dependency_graph):
    # Phase 0: Determine if IR is needed
    needs_ir = pass_manager.config.enable_ir_generation

    # Phase 1: Schedule AST foundation passes (always run)
    ast_foundation = [SymbolTable, TypeIndex, SyntaxAnalysis]
    schedule = [[ast_foundation]]  # Level 0

    # Phase 2: Schedule IR generation (if needed)
    if needs_ir:
        schedule.append([[IrGenerationPass]])  # Level 1

    # Phase 3: Schedule remaining analysis passes
    #   - AST passes can run after foundation
    #   - IR passes can run after IR generation
    #   - Hybrid passes wait for both their dependencies

    remaining_analysis = all_analysis_passes - scheduled
    analysis_schedule = schedule_with_representation_awareness(
        remaining_analysis,
        dependency_graph,
        needs_ir
    )
    schedule.extend(analysis_schedule)

    # Phase 4: Schedule bug detection passes
    #   Group by representation and dependency level
    detection_schedule = schedule_with_representation_awareness(
        all_detection_passes,
        dependency_graph,
        needs_ir
    )
    schedule.extend(detection_schedule)

    return schedule

function schedule_with_representation_awareness(passes, graph, needs_ir):
    """
    Enhanced topological sort that considers representation boundaries
    and enables parallel execution across representations
    """
    levels = []
    remaining = passes.copy()
    completed = set()

    # If IR exists, track which passes can run on which representation
    if needs_ir:
        completed.add(IrGeneration)  # Mark as completed in earlier phase

    while remaining:
        # Find passes with all dependencies satisfied
        ready_ast = [p for p in remaining
                     if p.representation == Ast
                     and graph.dependencies(p) ⊆ completed]

        ready_ir = [p for p in remaining
                    if p.representation == Ir
                    and graph.dependencies(p) ⊆ completed
                    and needs_ir]

        ready_hybrid = [p for p in remaining
                        if p.representation == Hybrid
                        and graph.dependencies(p) ⊆ completed
                        and needs_ir]

        ready = ready_ast + ready_ir + ready_hybrid

        if not ready:
            if remaining:
                raise CyclicDependencyError
            break

        # Group passes by representation for efficient execution
        level = {
            'ast': ready_ast,
            'ir': ready_ir,
            'hybrid': ready_hybrid
        }
        levels.append(level)

        completed.update(ready)
        remaining = [p for p in remaining if p not in ready]

    return levels
```

**Example Execution Schedule**:
```
Level 0: { ast: [SymbolTable, TypeIndex], ir: [], hybrid: [] }
         → Run AST passes in parallel

Level 1: { ast: [], ir: [IrGeneration], hybrid: [] }
         → Generate IR (sequential, transforms AST → IR)

Level 2: { ast: [Inheritance, StorageLayout, EventAnalysis, CallGraph],
           ir: [IrCfg, IrCallGraph],
           hybrid: [] }
         → Run AST and IR passes IN PARALLEL (independent)

Level 3: { ast: [],
           ir: [DominatorTree, LoopAnalysis, SsaConstruction],
           hybrid: [] }
         → Run IR passes in parallel

Level 4: { ast: [],
           ir: [DefUseChain, Liveness, TaintAnalysis, IrStateMutation],
           hybrid: [AccessControl] }
         → Run IR and hybrid passes in parallel

Detection Level 0: { ast: [TxOrigin, UncheckedCall, Visibility, ...],
                     ir: [],
                     hybrid: [] }
         → Run AST detection passes in parallel

Detection Level 1: { ast: [],
                     ir: [DeadCode, UninitializedStorage, ...],
                     hybrid: [Reentrancy, CEIViolation, ...] }
         → Run IR and hybrid detection passes in parallel
```

### 7.4 Representation-Aware Parallel Execution Strategy

**Enhanced Execution Phases** (with AST and IR):
```
Phase 0: AST Foundation (Parallel within phase)
  Level 0: AST: [SymbolTable, TypeIndex, Syntax] | IR: [] | Hybrid: []
  → Execute AST passes in parallel
  → Skip IR generation if no IR passes enabled

Phase 1: IR Generation (Sequential - AST transformation)
  Level 1: AST: [] | IR: [IrGeneration] | Hybrid: []
  → Transform AST → IR (sequential, but only if IR needed)

Phase 2: Parallel AST & IR Analysis (TRUE PARALLELISM)
  Level 2: AST: [Inheritance, Storage, Events, CallGraph]
         | IR: [IrCfg, IrCallGraph]
         | Hybrid: []
  → AST and IR passes run IN PARALLEL on different representations
  → Maximum CPU utilization

Phase 3: Advanced IR Analysis (Parallel within phase)
  Level 3: AST: [] | IR: [Dominator, Loop, SSA] | Hybrid: []
  → IR-only passes run in parallel

Phase 4: High-Level Analysis (Parallel AST/IR/Hybrid)
  Level 4: AST: []
         | IR: [DefUse, Liveness, Taint, IrStateMutation]
         | Hybrid: [AccessControl, StateConsistency]
  → IR and hybrid passes run in parallel

Phase 5: Bug Detection (Highly Parallel)
  Detection Level 0: AST: [Pattern detectors]
                   | IR: []
                   | Hybrid: []
  → All AST detectors run in parallel

  Detection Level 1: AST: []
                   | IR: [Flow detectors]
                   | Hybrid: [Security detectors]
  → IR and hybrid detectors run in parallel
```

**Parallelization Rules**:
1. **Cross-Representation Parallelism**: AST and IR passes at same level run in parallel
2. **Within-Representation Parallelism**: Passes on same representation with no dependencies → parallel
3. **Sequential Across Levels**: Dependencies enforced across levels
4. **Read-Only Parallelism**: Bug detection passes (read-only) highly parallelizable
5. **Write Isolation**: Analysis passes write to different context fields (minimal contention)

**Performance Benefits**:
- **2-4x speedup** from cross-representation parallelism (AST + IR analysis simultaneously)
- **Additional 2-3x** from within-representation parallelism
- **Total: 4-12x speedup** on multi-core systems (vs sequential execution)

**Implementation**:
```rust
impl PassExecutor {
    /// Execute a level with representation-aware parallelism
    fn execute_level_with_representations(
        &self,
        level: &RepresentationLevel,
        context: Arc<RwLock<AnalysisContext>>,
    ) -> Result<(), PassError> {
        // Create separate task groups for each representation
        let mut task_groups = vec![];

        // AST pass group
        if !level.ast_passes.is_empty() {
            let ast_group = level.ast_passes.par_iter()
                .map(|pass| self.execute_pass(pass, context.clone()));
            task_groups.push(ast_group);
        }

        // IR pass group (only if IR exists)
        if !level.ir_passes.is_empty() && context.read().unwrap().has_ir() {
            let ir_group = level.ir_passes.par_iter()
                .map(|pass| self.execute_pass(pass, context.clone()));
            task_groups.push(ir_group);
        }

        // Hybrid pass group
        if !level.hybrid_passes.is_empty() {
            let hybrid_group = level.hybrid_passes.par_iter()
                .map(|pass| self.execute_pass(pass, context.clone()));
            task_groups.push(hybrid_group);
        }

        // Execute all groups in parallel using Rayon
        // This achieves true parallelism across representations
        let results: Vec<Result<(), PassError>> = task_groups
            .into_par_iter()
            .flatten()
            .collect();

        // Check for errors
        for result in results {
            result?;
        }

        Ok(())
    }

    fn execute_pass<P: Pass>(
        &self,
        pass: &P,
        context: Arc<RwLock<AnalysisContext>>,
    ) -> Result<(), PassError> {
        let start = std::time::Instant::now();

        // Acquire lock based on pass type
        match pass.representation() {
            PassRepresentation::Ast => {
                // AST passes: read AST, write to AST-specific context fields
                let mut ctx = context.write().unwrap();
                pass.run_analysis(&mut *ctx)?;
            }
            PassRepresentation::Ir => {
                // IR passes: read IR, write to IR-specific context fields
                let mut ctx = context.write().unwrap();
                pass.run_analysis(&mut *ctx)?;
            }
            PassRepresentation::Hybrid => {
                // Hybrid passes: read both, write to hybrid fields
                let mut ctx = context.write().unwrap();
                pass.run_analysis(&mut *ctx)?;
            }
        }

        if self.config.enable_timing {
            let duration = start.elapsed();
            context.write().unwrap().record_pass_timing(pass.id(), duration);
        }

        Ok(())
    }
}

struct RepresentationLevel {
    ast_passes: Vec<Box<dyn AnalysisPass>>,
    ir_passes: Vec<Box<dyn AnalysisPass>>,
    hybrid_passes: Vec<Box<dyn AnalysisPass>>,
}
```

### 7.5 Unified Pass Execution Pipeline

```rust
impl PassManager {
    pub fn run(&mut self, context: &mut AnalysisContext) -> Result<AnalysisReport, PassError> {
        // Step 1: Determine if IR is needed
        let needs_ir = self.config.enable_ir_generation;

        // Step 2: Schedule all passes
        let schedule = self.scheduler.schedule(needs_ir)?;

        // Step 3: Execute analysis passes
        let context_arc = Arc::new(RwLock::new(context));

        for level in schedule.analysis_levels {
            if self.config.enable_parallel {
                // Parallel execution with representation awareness
                self.executor.execute_level_with_representations(&level, context_arc.clone())?;
            } else {
                // Sequential execution (for debugging)
                for pass in level.all_passes() {
                    self.executor.execute_pass(pass, context_arc.clone())?;
                }
            }
        }

        // Step 4: Execute bug detection passes
        let all_bugs = Arc::new(Mutex::new(Vec::new()));
        let context_ref = Arc::new(context_arc.read().unwrap().clone());

        for level in schedule.detection_levels {
            let level_bugs: Vec<Bug> = if self.config.enable_parallel {
                // Parallel detection across all representations
                let ast_bugs = level.ast_passes.par_iter()
                    .flat_map(|pass| pass.detect_bugs(&context_ref));

                let ir_bugs = level.ir_passes.par_iter()
                    .flat_map(|pass| pass.detect_bugs(&context_ref));

                let hybrid_bugs = level.hybrid_passes.par_iter()
                    .flat_map(|pass| pass.detect_bugs(&context_ref));

                ast_bugs.chain(ir_bugs).chain(hybrid_bugs).collect()
            } else {
                // Sequential detection
                level.all_passes().iter()
                    .flat_map(|pass| pass.detect_bugs(&context_ref))
                    .collect()
            };

            all_bugs.lock().unwrap().extend(level_bugs);
        }

        // Step 5: Generate report with both AST and IR information
        let final_bugs = Arc::try_unwrap(all_bugs).unwrap().into_inner().unwrap();
        let final_context = Arc::try_unwrap(context_arc).unwrap().into_inner().unwrap();

        Ok(AnalysisReport::new(final_bugs, final_context))
    }

    /// Run AST-only analysis (skip IR generation)
    pub fn run_ast_only(&mut self, context: &mut AnalysisContext) -> Result<AnalysisReport, PassError> {
        // Temporarily disable IR
        let original_ir_config = self.config.enable_ir_generation;
        self.config.enable_ir_generation = false;

        let result = self.run(context);

        // Restore config
        self.config.enable_ir_generation = original_ir_config;

        result
    }

    /// Run specific detector by ID (useful for targeted analysis)
    pub fn run_detector(
        &mut self,
        detector_id: PassId,
        context: &mut AnalysisContext,
    ) -> Result<Vec<Bug>, PassError> {
        // Find detector
        let detector = self.find_detector(detector_id)?;

        // Run only required analysis passes
        let required = detector.required_passes();
        self.run_required_passes(required, context)?;

        // Run detector
        Ok(detector.detect_bugs(context))
    }
}
```

### 7.6 Unified AnalysisContext with AST and IR

```rust
struct AnalysisContext {
    // ========================================
    // Source Representations (Immutable)
    // ========================================
    /// Original AST (always present)
    pub source_units: Vec<SourceUnit>,

    /// Generated IR (optional, created by IrGenerationPass)
    pub ir_units: Option<Vec<ir::SourceUnit>>,

    // ========================================
    // Configuration
    // ========================================
    pub config: AnalysisConfig,

    // ========================================
    // Pass Result Cache (Unified for AST, IR, Hybrid)
    // ========================================
    pass_results: HashMap<PassId, Box<dyn Any + Send + Sync>>,
    completed_passes: HashSet<PassId>,

    // ========================================
    // Pass Timing & Profiling
    // ========================================
    pass_timings: HashMap<PassId, Duration>,
    representation_stats: RepresentationStats,
}

struct RepresentationStats {
    ast_traversals: usize,
    ir_traversals: usize,
    ast_analysis_time: Duration,
    ir_analysis_time: Duration,
    ir_generation_time: Duration,
}

impl AnalysisContext {
    pub fn new(source_units: Vec<SourceUnit>, config: AnalysisConfig) -> Self {
        Self {
            source_units,
            ir_units: None,
            config,
            pass_results: HashMap::new(),
            completed_passes: HashSet::new(),
            pass_timings: HashMap::new(),
            representation_stats: RepresentationStats::default(),
        }
    }

    // ========================================
    // IR Management
    // ========================================

    /// Check if IR has been generated
    pub fn has_ir(&self) -> bool {
        self.ir_units.is_some()
    }

    /// Store generated IR (called by IrGenerationPass)
    pub fn store_ir(&mut self, ir_units: Vec<ir::SourceUnit>) {
        self.ir_units = Some(ir_units);
    }

    /// Get IR (panics if not generated - passes should check dependencies)
    pub fn ir(&self) -> &[ir::SourceUnit] {
        self.ir_units.as_ref().expect("IR not generated")
    }

    /// Get IR if available
    pub fn ir_opt(&self) -> Option<&[ir::SourceUnit]> {
        self.ir_units.as_deref()
    }

    // ========================================
    // Pass Result Management
    // ========================================

    /// Store pass result (works for AST, IR, or hybrid passes)
    pub fn store_pass_result<T: 'static + Send + Sync>(
        &mut self,
        pass_id: PassId,
        result: T,
    ) {
        self.pass_results.insert(pass_id, Box::new(result));
        self.completed_passes.insert(pass_id);
    }

    /// Get pass result
    pub fn get_pass_result<T: 'static>(&self, pass_id: PassId) -> Option<&T> {
        self.pass_results.get(&pass_id)?.downcast_ref::<T>()
    }

    /// Get pass result (panics if not found - for required dependencies)
    pub fn require_pass_result<T: 'static>(
        &self,
        pass_id: PassId,
    ) -> Result<&T, AnalysisError> {
        self.get_pass_result(pass_id)
            .ok_or(AnalysisError::DependencyNotMet(pass_id))
    }

    /// Check if pass completed
    pub fn is_pass_completed(&self, pass_id: PassId) -> bool {
        self.completed_passes.contains(&pass_id)
    }

    // ========================================
    // Profiling & Timing
    // ========================================

    pub fn record_pass_timing(&mut self, pass_id: PassId, duration: Duration) {
        self.pass_timings.insert(pass_id, duration);
    }

    pub fn get_pass_timing(&self, pass_id: PassId) -> Option<Duration> {
        self.pass_timings.get(&pass_id).copied()
    }

    pub fn total_analysis_time(&self) -> Duration {
        self.pass_timings.values().sum()
    }

    pub fn representation_stats(&self) -> &RepresentationStats {
        &self.representation_stats
    }

    // ========================================
    // Convenience Accessors (Backward Compatibility)
    // ========================================

    /// Get symbol table (AST analysis result)
    pub fn symbol_table(&self) -> Option<&SymbolTable> {
        self.get_pass_result(PassId::SymbolTable)
    }

    /// Get type index (AST analysis result)
    pub fn type_index(&self) -> Option<&TypeIndex> {
        self.get_pass_result(PassId::TypeIndex)
    }

    /// Get IR CFG (IR analysis result)
    pub fn ir_cfg(&self) -> Option<&IrCfgCollection> {
        self.get_pass_result(PassId::IrCfg)
    }

    /// Get SSA form (IR analysis result)
    pub fn ssa(&self) -> Option<&SsaForm> {
        self.get_pass_result(PassId::SsaConstruction)
    }

    /// Get def-use chains (IR analysis result)
    pub fn def_use_chains(&self) -> Option<&DefUseChains> {
        self.get_pass_result(PassId::DefUseChain)
    }

    /// Get taint analysis (IR analysis result)
    pub fn taint_graph(&self) -> Option<&TaintGraph> {
        self.get_pass_result(PassId::TaintAnalysis)
    }

    /// Get access control info (Hybrid analysis result)
    pub fn access_control(&self) -> Option<&AccessControlInfo> {
        self.get_pass_result(PassId::AccessControl)
    }
}
```

---

## 9. Migration Strategy

### 9.1 Incremental Migration Approach

**Goal**: Migrate from current architecture to unified AST+IR pass-based architecture with clear crate boundaries, without breaking existing functionality.

**Strategy**: **Phased Migration with Crate Separation, Dual Representation Support, and Compatibility Layer**

**Key Principles**:
1. **Build solidity crate infrastructure first** (analysis framework)
2. **Migrate smarthunt to use solidity crate** (bug detection)
3. Introduce IR support incrementally while maintaining AST-only operation
4. Maintain backward compatibility during transition

### 9.2 Migration Phases

#### Phase 1: solidity Crate - Foundation & AST Infrastructure (Weeks 1-3)
**Goal**: Build analysis framework in solidity crate

**Tasks (in solidity crate)**:
1. Create `analysis` module with pass infrastructure
2. Create `Pass`, `AnalysisPass` traits with `representation()` method
3. Add `PassRepresentation` enum (Ast, Ir, Hybrid)
4. Add `PassLevel` enum and level-based organization
5. Implement `PassManager` with representation awareness
6. Implement representation-aware `PassScheduler`
7. Implement `AnalysisContext` to support both AST and optional IR storage
8. Add parallel execution infrastructure (`PassExecutor`)

**Tasks (in smarthunt crate)**:
1. Create compatibility layer: `LegacyDetectorWrapper` to wrap existing detectors
2. Keep existing detectors working without changes
3. Add dependency on solidity crate

**Outcome**:
- solidity crate provides analysis framework
- smarthunt can start using solidity types
- All existing smarthunt detectors still work

#### Phase 2: solidity Crate - AST Analysis Passes (Weeks 4-5)
**Goal**: Implement AST-based analysis passes in solidity crate

**Tasks (in solidity crate)**:
1. Create `passes/ast/` module
2. Implement AST analysis passes as `AnalysisPass` trait
3. Mark all as `representation = Ast`
4. Add level specifications to each pass
5. Integrate with PassManager
6. Add comprehensive tests

**AST Passes to Implement**:
- SymbolTablePass (Ast, Contract + Function level)
- TypeIndexPass (Ast, Contract level)
- InheritanceGraphPass (Ast, Contract level)
- StorageLayoutPass (Ast, Contract level)
- EventAnalysisPass (Ast, Contract level)
- CallGraphPass (Ast, Function level)
- ModifierAnalysisPass (Ast, Function level)

**Tasks (in smarthunt crate)**:
- Update existing detectors to optionally use solidity analysis passes
- Test that results match existing implementation

**Outcome**:
- solidity crate provides reusable AST analysis
- smarthunt can use these passes
- smartproof can also use these passes (benefit of crate separation)

#### Phase 3: solidity Crate - IR Infrastructure & Generation (Weeks 6-7)
**Goal**: Complete IR generation capability in solidity crate

**Tasks (in solidity crate)**:
1. Complete `ir` module (currently in development)
2. Implement IR types and builder
3. Implement `IrGenerationPass` (transforms AST → SmartIR)
4. Update `AnalysisContext` to store IR units
5. Add lazy IR generation logic to PassManager
6. Implement IR validation
7. Add comprehensive IR tests

**Milestone**:
- IR generation complete in solidity crate
- IR can be generated from AST
- IR stored in context alongside AST
- No impact on existing AST-only analysis

**Outcome**:
- solidity crate provides IR generation
- Any tool using solidity can generate IR
- Foundation ready for IR-based analysis

#### Phase 4: solidity Crate - IR Analysis Passes (Weeks 8-10)
**Goal**: Implement IR-based analysis passes in solidity crate

**Tasks (in solidity crate)**:
1. Create `passes/ir/` module
2. Implement IR analysis passes:
   - `IrCfgPass` (build CFGs from IR)
   - `SsaConstructionPass` (convert to SSA form)
   - `DefUseChainPass` (build def-use chains)
   - `DominatorTreePass` (compute dominators)
   - `LoopAnalysisPass` (identify loops)
   - `LivenessAnalysisPass` (variable liveness)
   - `TaintAnalysisPass` (taint tracking)
   - `IrStateMutationPass` (track state changes)
   - `IrCallGraphPass` (call graph from IR)
3. Mark all as `representation = Ir`
4. All declare `IrGenerationPass` as dependency
5. Integrate with PassManager scheduling
6. Add comprehensive tests

**Outcome**:
- solidity crate provides rich IR analysis infrastructure
- smarthunt can use IR passes for bug detection
- smartproof can use IR passes for verification
- All tools benefit from shared IR analysis

#### Phase 5: smarthunt Crate - Bug Detection Pass Migration (Weeks 11-15)
**Goal**: Migrate bug detectors in smarthunt to use solidity analysis framework

**Tasks**:
1. Create `BugDetectionPass` trait in smarthunt (extends solidity's `Pass`)
2. Create `detection/detectors/ast/` module in smarthunt
3. Migrate detectors to new structure:
   - TxOriginDetector → TxOriginDetectionPass (Ast, Expression level)
   - UncheckedCallDetector → UncheckedCallDetectionPass (Ast, Statement level)
   - FloatingPragmaDetector → FloatingPragmaDetectionPass (Ast, Program level)
   - VisibilityDetector → VisibilityDetectionPass (Ast, Function level)
   - DeprecatedDetector → DeprecatedDetectionPass (Ast, Expression level)
   - LowLevelCallDetector → LowLevelCallDetectionPass (Ast, Expression level)
   - DelegatecallDetector → DelegatecallDetectionPass (Ast, Expression level)
   - ShadowingDetector → ShadowingDetectionPass (Ast, Variable level)
   - EventMismatchDetector → EventMismatchDetectionPass (Ast, Contract level)
4. Use solidity's AST analysis passes (SymbolTable, etc.)
5. Regression tests to ensure identical results

**Outcome**:
- smarthunt detectors use solidity analysis framework
- AST-only detectors migrated; no IR dependency
- Clear separation: analysis (solidity) vs detection (smarthunt)

**Sub-Phase 5B: IR-Only Detectors** (Weeks 13-14)
Implement new IR-based detectors in smarthunt (these don't exist in current system):

**Tasks**:
1. Create `detection/detectors/ir/` module in smarthunt
2. Implement new IR-based detectors:
   - DeadCodeDetectionPass (Ir, Block level) - NEW
   - UninitializedStorageDetectionPass (Ir, Variable level) - Enhanced with IR
   - UseAfterDeleteDetectionPass (Ir, Variable level) - NEW
   - UnreachableAfterRevertPass (Ir, Block level) - NEW
   - IntegerOverflowDetectionPass (Ir, Expression level) - NEW (for pre-0.8.0)
   - DivisionByZeroDetectionPass (Ir, Expression level) - NEW
   - InfiniteLoopDetectionPass (Ir, Block level) - NEW
   - UnusedReturnValuePass (Ir, Function level) - NEW
3. Use solidity's IR analysis passes (IrCfg, SSA, DefUse, Taint, etc.)
4. Add comprehensive tests for new detectors

**Outcome**:
- New IR-based detectors in smarthunt
- Enhanced vulnerability detection using solidity's IR analysis
- Demonstrates value of IR infrastructure

**Sub-Phase 5C: Hybrid Detectors** (Week 15)
Convert/enhance detectors in smarthunt that benefit from both AST and IR:

**Tasks**:
1. Create `detection/detectors/hybrid/` module in smarthunt
2. Implement hybrid detectors using both solidity AST and IR passes:
   - ReentrancyDetector → ReentrancyDetectionPass (Hybrid, Function level)
     - Uses solidity's: SymbolTable, ModifierAnalysis (AST) + IrCfg, IrStateMutation (IR)
     - AST: Identify reentrancy guards (modifiers)
     - IR: Precise flow + state mutation tracking

   - CEIViolationDetector → CEIViolationDetectionPass (Hybrid, Function level)
     - Uses solidity's: SymbolTable (AST) + IrCfg, IrStateMutation (IR)
     - AST: Function boundaries
     - IR: Precise statement ordering

   - CentralizationRiskDetector → CentralizationRiskPass (Hybrid, Contract level)
     - Uses solidity's: AccessControl (Hybrid) + IrStateMutation (IR)
     - AST: Access control semantics
     - IR: State mutation tracking

   - MissingAccessControlDetector → MissingAccessControlPass (Hybrid, Function level)
     - Uses solidity's: AccessControl (Hybrid) + IrStateMutation (IR)
     - AST: Modifier checking
     - IR: State modification analysis

   - TimestampDependenceDetector → TimestampDependencePass (Hybrid, Expression level)
     - Uses solidity's: SymbolTable (AST) + IrCfg, TaintAnalysis (IR)
     - AST: Identify timestamp usage
     - IR: Taint flow tracking

   - FrontRunningVulnerabilityPass (Hybrid, Function level) - NEW
     - Uses solidity's: SymbolTable (AST) + IrCfg, IrStateMutation, TaintAnalysis (IR)
     - AST: Function visibility
     - IR: Input-to-state flow analysis

**Outcome**:
- All smarthunt detectors migrated
- Fully leverage both AST and IR from solidity crate
- Demonstrate power of hybrid analysis

#### Phase 6: Optimization & Performance Tuning (Weeks 16-18)
**Goal**: Optimize parallel execution and leverage dual representation benefits

**Tasks**:
1. **Level-Grouped Traversal** (Week 16)
   - Implement single-traversal execution for AST passes at same level
   - Implement single-traversal execution for IR passes at same level
   - Measure traversal reduction (target: 16+ → 7 traversals)

2. **Cross-Representation Parallelism** (Week 17)
   - Optimize scheduling to run AST and IR passes concurrently
   - Minimize lock contention in AnalysisContext
   - Profile parallel execution efficiency

3. **Caching & Profiling** (Week 18)
   - Add pass result caching optimizations
   - Implement detailed pass timing and profiling
   - Add representation-level statistics tracking
   - Create performance dashboard for analysis

**Performance Targets**:
- 4-12x speedup vs original sequential execution
- 2-4x speedup from cross-representation parallelism
- 2-3x speedup from within-representation parallelism

**Outcome**: Highly optimized analysis engine with maximum throughput

#### Phase 7: Hybrid Pass Optimization (Week 19)
**Goal**: Optimize hybrid passes that use both AST and IR

**Tasks**:
1. Profile hybrid pass performance
2. Optimize data transfer between AST and IR views
3. Implement caching for frequently accessed cross-representation data
4. Ensure hybrid passes don't create bottlenecks

**Outcome**: Hybrid passes perform efficiently without degrading parallelism

#### Phase 8: Cleanup & Documentation (Weeks 20-22)
**Goal**: Remove legacy code and comprehensively document dual-representation system

**Tasks**:
1. Remove old `Detector` trait and `DetectorRegistry` (Week 20)
2. Remove `LegacyDetectorWrapper` compatibility layer (Week 20)
3. Update all documentation to explain AST vs IR vs Hybrid passes (Week 21)
4. Create detailed guide: "When to Use AST vs IR for Bug Detection" (Week 21)
5. Add examples for creating custom passes:
   - AST-only pass example
   - IR-only pass example
   - Hybrid pass example
6. Write migration guide for external detector developers (Week 22)
7. Create video tutorials and architecture diagrams (Week 22)

**Documentation Deliverables**:
- Architecture overview with dual representation
- API documentation for Pass traits
- "Best Practices for Pass Development"
- Performance tuning guide
- Migration guide for v1 → v2

**Outcome**: Clean, well-documented, dual-representation pass-based architecture ready for production

### 9.3 Crate Organization Benefits Realized

By the end of migration, the crate organization delivers:

**solidity Crate** (Analysis Framework Provider):
- ✅ Complete pass infrastructure (PassManager, scheduling, execution)
- ✅ AST analysis passes (7 passes)
- ✅ IR generation and representation
- ✅ IR analysis passes (9 passes)
- ✅ Hybrid analysis passes (2 passes)
- ✅ **Reusable by any tool**: smarthunt, smartproof, future tools

**smarthunt Crate** (Bug Detection Consumer):
- ✅ BugDetectionPass trait (extends solidity::Pass)
- ✅ 25+ bug detectors (9 AST, 8 IR, 8 Hybrid)
- ✅ Bug reporting and CLI
- ✅ **Consumes solidity analysis**: No duplicate analysis code

**Benefits Achieved**:
1. **Zero Duplication**: smartproof can use same analysis as smarthunt
2. **Consistent Results**: All tools use same analysis implementation
3. **Easier Maintenance**: Fix analysis bugs once, benefits all tools
4. **Clear Boundaries**: Analysis (solidity) vs Application (smarthunt/smartproof)
5. **Extensibility**: New tools easily leverage solidity framework

### 7.3 Compatibility Layer Design

**Purpose**: Allow old detectors to work with new pass system during migration

```rust
/// Wrapper to adapt old Detector trait to new BugDetectionPass trait
struct LegacyDetectorWrapper {
    detector: Box<dyn Detector>,
    pass_id: PassId,
    level: PassLevel,  // Assigned based on detector behavior
}

impl Pass for LegacyDetectorWrapper {
    fn id(&self) -> PassId { self.pass_id }
    fn name(&self) -> &'static str { self.detector.name() }
    fn description(&self) -> &'static str { self.detector.description() }
    fn level(&self) -> PassLevel { self.level }
    fn required_passes(&self) -> Vec<PassId> {
        // Convert detector's required_passes to new PassId format
        self.detector.required_passes()
    }
}

impl BugDetectionPass for LegacyDetectorWrapper {
    fn detect_bugs(&self, context: &AnalysisContext) -> Vec<Bug> {
        // Delegate to old detector
        self.detector.detect(context)
    }

    fn bug_kind(&self) -> BugKind { self.detector.bug_kind() }
    fn risk_level(&self) -> RiskLevel { self.detector.risk_level() }
    // ... other trait methods
}
```

### 8.4 Testing Strategy

**Per-Phase Testing**:

1. **Unit Tests**: Test each pass in isolation
   - AST pass tests: Verify pass works on sample AST
   - IR pass tests: Verify pass works on sample IR
   - Hybrid pass tests: Verify pass correctly uses both representations

2. **Integration Tests**: Test PassManager and scheduling
   - Test representation-aware scheduling
   - Test lazy IR generation (IR only created when needed)
   - Test cross-representation parallelism
   - Test pass dependency resolution across representations

3. **Regression Tests**: Ensure migrated passes produce identical or better results
   - For AST-only detectors: Exact match with old implementation
   - For enhanced detectors: At least as many bugs detected (may find more with IR)

4. **IR Correctness Tests**: Validate IR generation and analysis
   - AST → IR transformation preserves semantics
   - IR CFG correctly represents control flow
   - SSA construction produces valid SSA form
   - Def-use chains are accurate

5. **Performance Tests**: Measure speedup from dual representation
   - Measure AST-only vs AST+IR overhead
   - Measure parallel execution efficiency
   - Measure memory usage (AST + IR vs AST alone)
   - Identify bottlenecks

**Regression Test Approach**:
```rust
#[test]
fn test_ast_detector_migration_identical_results() {
    let source = "..."; // Test contract
    let old_bugs = run_old_detector(source);
    let new_bugs = run_new_ast_pass(source);

    // For AST-only detectors, should be identical
    assert_eq!(old_bugs.len(), new_bugs.len());
    for (old, new) in old_bugs.iter().zip(new_bugs.iter()) {
        assert_eq!(old.id, new.id);
        assert_eq!(old.location, new.location);
        assert_eq!(old.message, new.message);
    }
}

#[test]
fn test_hybrid_detector_enhanced_detection() {
    let source = "..."; // Test contract with subtle reentrancy
    let old_bugs = run_old_detector(source);
    let new_bugs = run_new_hybrid_pass(source);

    // Hybrid detectors should find at least as many bugs
    // (may find more with IR-based precision)
    assert!(new_bugs.len() >= old_bugs.len());

    // All old bugs should still be detected
    for old_bug in old_bugs {
        assert!(new_bugs.iter().any(|b| b.is_equivalent(&old_bug)));
    }
}

#[test]
fn test_ir_generation_correctness() {
    let source = parse_solidity("contract C { ... }");
    let ir = generate_ir(&source);

    // Validate IR properties
    assert!(ir.is_valid_ssa());
    assert_eq!(ir.entry_blocks().len(), source.functions().len());
    assert!(ir.preserves_semantics(&source));
}

#[test]
fn test_lazy_ir_generation() {
    let mut manager = PassManager::new(config);

    // Register only AST passes
    manager.register_analysis_pass(Box::new(SymbolTablePass));
    manager.register_bug_detection_pass(Box::new(TxOriginDetectionPass));

    let mut context = AnalysisContext::new(source_units, config);
    manager.run(&mut context).unwrap();

    // IR should not be generated
    assert!(!context.has_ir());
}

#[test]
fn test_cross_representation_parallelism() {
    let mut manager = PassManager::new(config);

    // Register both AST and IR passes
    manager.register_analysis_pass(Box::new(InheritanceGraphPass)); // AST
    manager.register_analysis_pass(Box::new(IrCfgPass)); // IR

    let start = Instant::now();
    let mut context = AnalysisContext::new(large_source_units, config);
    manager.run(&mut context).unwrap();
    let duration = start.elapsed();

    // Should be faster than sequential execution
    // (measure against baseline)
    assert!(duration < sequential_baseline * 0.6); // At least 40% faster
}
```

**Test Coverage Goals**:
- 100% coverage of pass trait implementations
- 100% coverage of scheduling algorithm
- 95%+ coverage of all analysis and detection passes
- Comprehensive regression test suite (50+ real-world contracts)

---

## 10. Implementation Plan

### 10.1 File Structure

**Key Principle**: Analysis infrastructure in `solidity` crate, bug detection in `smarthunt` crate.

#### solidity Crate (Analysis Framework)

**New/Modified Files**:
```
solidity/src/
├── analysis/                    # NEW: Analysis framework module
│   ├── mod.rs
│   ├── pass.rs                  # Pass, AnalysisPass traits (no BugDetectionPass)
│   ├── context.rs               # AnalysisContext
│   ├── manager.rs               # PassManager
│   ├── scheduler.rs             # Representation-aware PassScheduler
│   ├── executor.rs              # PassExecutor with cross-representation parallelism
│   ├── dependency.rs            # DependencyGraph for pass ordering
│   ├── pass_level.rs            # PassLevel enum
│   └── pass_representation.rs   # PassRepresentation enum (Ast/Ir/Hybrid)
│
├── ir/                          # IR representation (IN DEVELOPMENT)
│   ├── mod.rs
│   ├── types.rs                 # IR type definitions
│   ├── builder.rs               # IR builder
│   ├── generation.rs            # IrGenerationPass (AST → IR)
│   └── validation.rs            # IR validation
│
└── passes/                      # NEW: Analysis passes
    ├── mod.rs
    ├── ast/                     # AST-based analysis passes
    │   ├── mod.rs
    │   ├── symbol_table.rs      # SymbolTablePass [AST]
    │   ├── type_index.rs        # TypeIndexPass [AST]
    │   ├── inheritance.rs       # InheritanceGraphPass [AST]
    │   ├── storage_layout.rs    # StorageLayoutPass [AST]
    │   ├── event_analysis.rs    # EventAnalysisPass [AST]
    │   ├── call_graph.rs        # CallGraphPass [AST]
    │   └── modifier.rs          # ModifierAnalysisPass [AST]
    ├── ir/                      # IR-based analysis passes
    │   ├── mod.rs
    │   ├── cfg.rs               # IrCfgPass [IR]
    │   ├── ssa.rs               # SsaConstructionPass [IR]
    │   ├── dominator.rs         # DominatorTreePass [IR]
    │   ├── loops.rs             # LoopAnalysisPass [IR]
    │   ├── def_use.rs           # DefUseChainPass [IR]
    │   ├── liveness.rs          # LivenessAnalysisPass [IR]
    │   ├── taint.rs             # TaintAnalysisPass [IR]
    │   ├── state_mutation.rs    # IrStateMutationPass [IR]
    │   └── call_graph.rs        # IrCallGraphPass [IR]
    └── hybrid/                  # Hybrid analysis passes (AST + IR)
        ├── mod.rs
        ├── access_control.rs    # AccessControlPass [HYBRID]
        └── consistency.rs       # StateConsistencyPass [HYBRID]
```

**Public API** (`solidity/src/lib.rs`):
```rust
// Existing AST exports
pub mod ast;

// NEW: Analysis framework
pub mod analysis {
    pub use crate::analysis::{
        Pass, AnalysisPass, PassLevel, PassRepresentation,
        PassManager, AnalysisContext, PassId,
    };

    pub mod passes {
        pub use crate::passes::ast::*;
        pub use crate::passes::ir::*;
        pub use crate::passes::hybrid::*;
    }
}

// NEW: IR (when ready)
pub mod ir {
    pub use crate::ir::*;
}
```

#### smarthunt Crate (Bug Detection)

**New/Modified Files**:
```
smarthunt/src/
├── detection/                   # NEW: Bug detection module
│   ├── mod.rs
│   ├── pass.rs                  # BugDetectionPass trait (extends solidity::Pass)
│   ├── bug.rs                   # Bug type definitions
│   └── detectors/
│       ├── mod.rs
│       ├── ast/                 # AST-only detection passes
│       │   ├── mod.rs
│       │   ├── tx_origin.rs
│       │   ├── unchecked_call.rs
│       │   ├── floating_pragma.rs
│       │   ├── visibility.rs
│       │   ├── deprecated.rs
│       │   ├── low_level_call.rs
│       │   ├── delegatecall.rs
│       │   ├── shadowing.rs
│       │   └── event_mismatch.rs
│       ├── ir/                  # IR-only detection passes
│       │   ├── mod.rs
│       │   ├── dead_code.rs
│       │   ├── uninitialized.rs
│       │   ├── use_after_delete.rs
│       │   ├── unreachable.rs
│       │   ├── integer_overflow.rs
│       │   ├── division_by_zero.rs
│       │   ├── infinite_loop.rs
│       │   └── unused_return.rs
│       └── hybrid/              # Hybrid detection passes (AST + IR)
│           ├── mod.rs
│           ├── reentrancy.rs
│           ├── cei_violation.rs
│           ├── centralization.rs
│           ├── missing_access_control.rs
│           ├── timestamp.rs
│           └── front_running.rs
│
├── report/                      # Bug reporting
│   ├── mod.rs
│   ├── formatter.rs             # JSON, SARIF, Markdown formatters
│   └── analysis_report.rs       # AnalysisReport type
│
├── cli/                         # Command-line interface
│   ├── mod.rs
│   └── commands.rs
│
└── compat/                      # Compatibility (temporary)
    └── legacy_wrapper.rs        # LegacyDetectorWrapper
```

**Dependencies** (`smarthunt/Cargo.toml`):
```toml
[dependencies]
solidity = { path = "../solidity" }  # Use analysis framework
rayon = "1.5"
serde = { version = "1.0", features = ["derive"] }
# ... other dependencies
```

**Public API** (`smarthunt/src/lib.rs`):
```rust
// Re-export from solidity for convenience
pub use solidity::analysis::{Pass, AnalysisPass, PassManager, AnalysisContext};

// smarthunt-specific
pub mod detection {
    pub use crate::detection::{BugDetectionPass, Bug, BugKind};
    pub mod detectors {
        pub use crate::detection::detectors::ast::*;
        pub use crate::detection::detectors::ir::*;
        pub use crate::detection::detectors::hybrid::*;
    }
}

pub mod report;
pub mod cli;
```

**Modified Files**:
```
smarthunt/src/
├── engine/
│   ├── context.rs               # Add pass result caching
│   └── mod.rs                   # Export new pass infrastructure
└── lib.rs                       # Update public API
```

**Files to Eventually Remove** (after Phase 5):
```
smarthunt/src/
├── base/detector.rs             # Old Detector trait
├── registry.rs                  # Old DetectorRegistry
└── detectors/                   # Old detector implementations (move to passes/)
```

### 9.2 Key Implementation Tasks

#### Task 1: Pass Trait Hierarchy with Representation Support (Weeks 1-2)

**File**: `smarthunt/src/engine/pass.rs`

```rust
use std::any::Any;
use std::fmt::Debug;
use crate::engine::context::AnalysisContext;
use crate::base::bug::{Bug, BugKind, RiskLevel, ConfidenceLevel};

/// Pass granularity level (applies to both AST and IR)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassLevel {
    Program,      // Multi-contract analysis (AST: SourceUnit, IR: Module)
    Contract,     // Per-contract analysis (AST: ContractDefinition, IR: Contract)
    Function,     // Per-function analysis (AST: FunctionDefinition, IR: Function)
    Block,        // Per-block analysis (AST: Block, IR: BasicBlock)
    Statement,    // Per-statement analysis (AST: Statement, IR: Instruction)
    Expression,   // Per-expression analysis (AST: Expression, IR: Operand)
    Variable,     // Per-variable analysis (AST: VariableDeclaration, IR: SSA Variable)
}

/// Representation the pass operates on
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassRepresentation {
    Ast,     // Operates on AST only
    Ir,      // Operates on IR only (automatically requires IrGeneration)
    Hybrid,  // Operates on both AST and IR
}

/// Unique identifier for passes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassId {
    // Special pass
    IrGeneration,

    // AST analysis passes
    SymbolTable,
    TypeIndex,
    Inheritance,
    StorageLayout,
    EventAnalysis,
    CallGraph,
    ModifierAnalysis,

    // IR analysis passes
    IrCfg,
    IrCallGraph,
    SsaConstruction,
    DominatorTree,
    LoopAnalysis,
    DefUseChain,
    Liveness,
    TaintAnalysis,
    IrStateMutation,

    // Hybrid analysis passes
    AccessControl,
    StateConsistency,

    // AST bug detection passes
    TxOrigin,
    UncheckedCall,
    FloatingPragma,
    Visibility,
    Deprecated,
    LowLevelCall,
    Delegatecall,
    Shadowing,
    EventMismatch,

    // IR bug detection passes
    DeadCode,
    UninitializedStorage,
    UseAfterDelete,
    UnreachableAfterRevert,
    IntegerOverflow,
    DivisionByZero,
    InfiniteLoop,
    UnusedReturn,

    // Hybrid bug detection passes
    Reentrancy,
    CEIViolation,
    CentralizationRisk,
    MissingAccessControl,
    TimestampDependence,
    FrontRunning,
}

/// Base trait for all passes
pub trait Pass: Send + Sync + Debug {
    /// Unique identifier
    fn id(&self) -> PassId;

    /// Human-readable name
    fn name(&self) -> &'static str;

    /// Description of what this pass does
    fn description(&self) -> &'static str;

    /// Operating level(s) of this pass
    fn level(&self) -> PassLevel;

    /// Representation this pass operates on
    fn representation(&self) -> PassRepresentation;

    /// Multiple levels (for passes that operate at multiple granularities)
    fn levels(&self) -> Vec<PassLevel> {
        vec![self.level()]
    }

    /// Passes required before this pass can run
    fn required_passes(&self) -> Vec<PassId> {
        vec![]
    }

    /// Whether this pass preserves all analysis results (for optimization)
    fn preserves_all(&self) -> bool {
        true  // Bug detection passes preserve everything
    }
}

/// Analysis pass that collects information
pub trait AnalysisPass: Pass {
    /// Run the analysis and store results in context
    fn run_analysis(&self, context: &mut AnalysisContext) -> Result<(), AnalysisError>;

    /// Get result type name (for debugging)
    fn result_type_name(&self) -> &'static str;
}

/// Bug detection pass that finds vulnerabilities
pub trait BugDetectionPass: Pass {
    /// Detect bugs using analysis results from context
    fn detect_bugs(&self, context: &AnalysisContext) -> Vec<Bug>;

    /// Kind of bugs this pass detects
    fn bug_kind(&self) -> BugKind;

    /// Risk level of bugs detected
    fn risk_level(&self) -> RiskLevel;

    /// Confidence level of detection
    fn confidence(&self) -> ConfidenceLevel;

    /// CWE IDs for detected bugs
    fn cwe_ids(&self) -> Vec<usize> { vec![] }

    /// SWC IDs for detected bugs
    fn swc_ids(&self) -> Vec<usize> { vec![] }
}

/// Error type for pass execution
#[derive(Debug, Clone)]
pub enum AnalysisError {
    DependencyNotMet(PassId),
    ExecutionFailed(String),
    InvalidContext(String),
    IrNotGenerated,  // IR required but not available
}
```

#### Task 2: PassManager Implementation (Week 1-2)

**File**: `smarthunt/src/engine/pass_manager.rs`

```rust
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock, Mutex};
use rayon::prelude::*;

pub struct PassManager {
    analysis_passes: HashMap<PassId, Box<dyn AnalysisPass>>,
    bug_detection_passes: HashMap<PassId, Box<dyn BugDetectionPass>>,
    dependency_graph: DependencyGraph,
    config: PassManagerConfig,
}

pub struct PassManagerConfig {
    pub enable_parallel: bool,
    pub max_parallelism: usize,
    pub enable_caching: bool,
    pub enable_timing: bool,
    pub min_parallel_size: usize,  // Minimum passes to parallelize
}

impl PassManager {
    pub fn new(config: PassManagerConfig) -> Self {
        Self {
            analysis_passes: HashMap::new(),
            bug_detection_passes: HashMap::new(),
            dependency_graph: DependencyGraph::new(),
            config,
        }
    }

    pub fn register_analysis_pass(&mut self, pass: Box<dyn AnalysisPass>) {
        let id = pass.id();
        let deps = pass.required_passes();
        self.dependency_graph.add_pass(id, deps);
        self.analysis_passes.insert(id, pass);
    }

    pub fn register_bug_detection_pass(&mut self, pass: Box<dyn BugDetectionPass>) {
        let id = pass.id();
        let deps = pass.required_passes();
        self.dependency_graph.add_pass(id, deps);
        self.bug_detection_passes.insert(id, pass);
    }

    pub fn run(&self, context: &mut AnalysisContext) -> Result<AnalysisReport, PassError> {
        // Schedule passes
        let (analysis_schedule, detection_schedule) = self.schedule_passes()?;

        // Execute analysis passes
        self.execute_analysis_passes(analysis_schedule, context)?;

        // Execute bug detection passes
        let bugs = self.execute_detection_passes(detection_schedule, context)?;

        // Create report
        Ok(AnalysisReport::new(bugs, context))
    }

    fn schedule_passes(&self) -> Result<(Vec<Vec<PassId>>, Vec<Vec<PassId>>), PassError> {
        let analysis_ids: Vec<PassId> = self.analysis_passes.keys().copied().collect();
        let detection_ids: Vec<PassId> = self.bug_detection_passes.keys().copied().collect();

        let analysis_schedule = self.dependency_graph.topological_sort_levels(&analysis_ids)?;
        let detection_schedule = self.dependency_graph.topological_sort_levels(&detection_ids)?;

        Ok((analysis_schedule, detection_schedule))
    }

    fn execute_analysis_passes(
        &self,
        schedule: Vec<Vec<PassId>>,
        context: &mut AnalysisContext,
    ) -> Result<(), PassError> {
        for level in schedule {
            if self.config.enable_parallel && level.len() >= self.config.min_parallel_size {
                // Parallel execution within level
                self.execute_analysis_level_parallel(level, context)?;
            } else {
                // Sequential execution
                for pass_id in level {
                    let pass = self.analysis_passes.get(&pass_id).unwrap();
                    let start = std::time::Instant::now();
                    pass.run_analysis(context)?;
                    if self.config.enable_timing {
                        context.record_pass_timing(pass_id, start.elapsed());
                    }
                }
            }
        }
        Ok(())
    }

    fn execute_analysis_level_parallel(
        &self,
        level: Vec<PassId>,
        context: &mut AnalysisContext,
    ) -> Result<(), PassError> {
        // Need to use Arc + RwLock for thread-safe context access
        // Each pass writes to different fields, so contention is minimal
        let context_lock = Arc::new(RwLock::new(context));

        let results: Vec<Result<(), PassError>> = level.par_iter()
            .map(|&pass_id| {
                let pass = self.analysis_passes.get(&pass_id).unwrap();
                let start = std::time::Instant::now();

                // Acquire write lock
                let mut ctx = context_lock.write().unwrap();
                let result = pass.run_analysis(&mut *ctx);

                if self.config.enable_timing {
                    ctx.record_pass_timing(pass_id, start.elapsed());
                }

                result.map_err(|e| PassError::AnalysisFailed(pass_id, e))
            })
            .collect();

        // Check for errors
        for result in results {
            result?;
        }

        Ok(())
    }

    fn execute_detection_passes(
        &self,
        schedule: Vec<Vec<PassId>>,
        context: &AnalysisContext,
    ) -> Result<Vec<Bug>, PassError> {
        let all_bugs = Arc::new(Mutex::new(Vec::new()));

        for level in schedule {
            let bugs: Vec<Bug> = if self.config.enable_parallel && level.len() >= self.config.min_parallel_size {
                // Parallel execution (read-only, highly parallelizable)
                level.par_iter()
                    .flat_map(|&pass_id| {
                        let pass = self.bug_detection_passes.get(&pass_id).unwrap();
                        pass.detect_bugs(context)
                    })
                    .collect()
            } else {
                // Sequential execution
                level.iter()
                    .flat_map(|&pass_id| {
                        let pass = self.bug_detection_passes.get(&pass_id).unwrap();
                        pass.detect_bugs(context)
                    })
                    .collect()
            };

            all_bugs.lock().unwrap().extend(bugs);
        }

        Ok(Arc::try_unwrap(all_bugs).unwrap().into_inner().unwrap())
    }
}
```

#### Task 3: DependencyGraph Implementation (Week 1)

**File**: `smarthunt/src/engine/dependency.rs`

```rust
use std::collections::{HashMap, HashSet, VecDeque};

pub struct DependencyGraph {
    // Adjacency list: pass_id -> dependencies
    dependencies: HashMap<PassId, Vec<PassId>>,
    // Reverse: pass_id -> dependents
    dependents: HashMap<PassId, Vec<PassId>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    pub fn add_pass(&mut self, pass_id: PassId, required: Vec<PassId>) {
        self.dependencies.insert(pass_id, required.clone());

        for dep in required {
            self.dependents.entry(dep)
                .or_insert_with(Vec::new)
                .push(pass_id);
        }
    }

    /// Topological sort with level grouping for parallel execution
    pub fn topological_sort_levels(&self, passes: &[PassId]) -> Result<Vec<Vec<PassId>>, PassError> {
        let mut levels: Vec<Vec<PassId>> = Vec::new();
        let mut remaining: HashSet<PassId> = passes.iter().copied().collect();
        let mut completed: HashSet<PassId> = HashSet::new();

        while !remaining.is_empty() {
            // Find all passes with satisfied dependencies
            let ready: Vec<PassId> = remaining.iter()
                .filter(|&&pass_id| {
                    let deps = self.dependencies.get(&pass_id).map(|v| v.as_slice()).unwrap_or(&[]);
                    deps.iter().all(|dep| completed.contains(dep))
                })
                .copied()
                .collect();

            if ready.is_empty() {
                // Cyclic dependency or missing pass
                return Err(PassError::CyclicDependency);
            }

            // Add this level
            levels.push(ready.clone());

            // Mark as completed
            for pass_id in ready {
                completed.insert(pass_id);
                remaining.remove(&pass_id);
            }
        }

        Ok(levels)
    }
}
```

#### Task 4: Enhanced AnalysisContext (Week 2)

**File**: `smarthunt/src/engine/context.rs` (modifications)

```rust
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::time::Duration;

pub struct AnalysisContext {
    // Existing fields
    pub source_units: Vec<SourceUnit>,
    pub config: AnalysisConfig,

    // OLD: Specific analysis results (to be deprecated)
    pub symbols: Option<SymbolTable>,
    pub type_index: Option<TypeIndex>,
    // ... other specific fields

    // NEW: Generic pass result storage
    pass_results: HashMap<PassId, Box<dyn Any + Send + Sync>>,
    completed_passes: HashSet<PassId>,
    pass_timings: HashMap<PassId, Duration>,
}

impl AnalysisContext {
    // NEW: Generic pass result storage/retrieval
    pub fn store_pass_result<T: 'static + Send + Sync>(
        &mut self,
        pass_id: PassId,
        result: T,
    ) {
        self.pass_results.insert(pass_id, Box::new(result));
        self.completed_passes.insert(pass_id);
    }

    pub fn get_pass_result<T: 'static>(&self, pass_id: PassId) -> Option<&T> {
        self.pass_results
            .get(&pass_id)?
            .downcast_ref::<T>()
    }

    pub fn is_pass_completed(&self, pass_id: PassId) -> bool {
        self.completed_passes.contains(&pass_id)
    }

    pub fn require_pass_result<T: 'static>(
        &self,
        pass_id: PassId,
    ) -> Result<&T, AnalysisError> {
        self.get_pass_result(pass_id)
            .ok_or(AnalysisError::DependencyNotMet(pass_id))
    }

    pub fn record_pass_timing(&mut self, pass_id: PassId, duration: Duration) {
        self.pass_timings.insert(pass_id, duration);
    }

    // Compatibility methods (delegate to new storage)
    pub fn symbol_table(&self) -> Option<&SymbolTable> {
        self.get_pass_result(PassId::SymbolTable)
    }

    pub fn type_index(&self) -> Option<&TypeIndex> {
        self.get_pass_result(PassId::TypeIndex)
    }

    // ... similar compatibility methods for other results
}
```

#### Task 5: Example Pass Migration (Week 3)

**Example: Migrate SymbolTablePass**

**File**: `smarthunt/src/passes/analysis/symbol_table.rs`

```rust
use crate::engine::pass::{Pass, AnalysisPass, PassId, PassLevel, AnalysisError};
use crate::engine::context::AnalysisContext;
use crate::analysis::symbol::SymbolTable;

#[derive(Debug)]
pub struct SymbolTablePass;

impl Pass for SymbolTablePass {
    fn id(&self) -> PassId {
        PassId::SymbolTable
    }

    fn name(&self) -> &'static str {
        "Symbol Table Builder"
    }

    fn description(&self) -> &'static str {
        "Builds symbol table for name resolution across contracts and functions"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Contract
    }

    fn levels(&self) -> Vec<PassLevel> {
        vec![PassLevel::Contract, PassLevel::Function]
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![]  // No dependencies
    }
}

impl AnalysisPass for SymbolTablePass {
    fn run_analysis(&self, context: &mut AnalysisContext) -> Result<(), AnalysisError> {
        let mut symbol_table = SymbolTable::new();

        // Build symbol table from source units
        for source_unit in &context.source_units {
            symbol_table.process_source_unit(source_unit)
                .map_err(|e| AnalysisError::ExecutionFailed(e.to_string()))?;
        }

        // Store result in context
        context.store_pass_result(PassId::SymbolTable, symbol_table);

        Ok(())
    }

    fn result_type_name(&self) -> &'static str {
        "SymbolTable"
    }
}
```

**Example: Migrate TxOriginDetector to Pass**

**File**: `smarthunt/src/passes/detection/pattern/tx_origin.rs`

```rust
use crate::engine::pass::{Pass, BugDetectionPass, PassId, PassLevel};
use crate::engine::context::AnalysisContext;
use crate::base::bug::{Bug, BugKind, RiskLevel, ConfidenceLevel};
use solidity::ast::utils::Visit;

#[derive(Debug)]
pub struct TxOriginDetectionPass;

impl Pass for TxOriginDetectionPass {
    fn id(&self) -> PassId {
        PassId::TxOrigin
    }

    fn name(&self) -> &'static str {
        "tx.origin Detector"
    }

    fn description(&self) -> &'static str {
        "Detects usage of tx.origin for authentication"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Expression
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable]
    }
}

impl BugDetectionPass for TxOriginDetectionPass {
    fn detect_bugs(&self, context: &AnalysisContext) -> Vec<Bug> {
        let symbol_table = context.require_pass_result::<SymbolTable>(PassId::SymbolTable)
            .expect("SymbolTable pass should have run");

        let mut visitor = TxOriginVisitor {
            bugs: Vec::new(),
            symbol_table,
        };

        for source_unit in &context.source_units {
            visitor.visit_source_unit(source_unit);
        }

        visitor.bugs
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Medium
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![477]  // CWE-477: Use of Obsolete Function
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![115]  // SWC-115: Authorization through tx.origin
    }
}

struct TxOriginVisitor<'a> {
    bugs: Vec<Bug>,
    symbol_table: &'a SymbolTable,
}

impl<'a, 'b> Visit<'b> for TxOriginVisitor<'a> {
    fn visit_expr(&mut self, expr: &'b Expr) {
        // Detection logic (same as before)
        if self.is_tx_origin_usage(expr) {
            self.bugs.push(Bug::new(
                "tx-origin",
                "Use of tx.origin for authentication",
                expr.span(),
                RiskLevel::Medium,
            ));
        }

        solidity::ast::utils::visit::default::visit_expr(self, expr);
    }
}
```

#### Task 6: IR Generation Pass (Weeks 6-7)

**Goal**: Transform AST to IR representation

**File**: `smarthunt/src/engine/ir_generation.rs`

```rust
use crate::engine::pass::{Pass, AnalysisPass, PassId, PassLevel, PassRepresentation, AnalysisError};
use crate::engine::context::AnalysisContext;
use smartir::{IrGenerator, SourceUnit as IrSourceUnit};

#[derive(Debug)]
pub struct IrGenerationPass;

impl Pass for IrGenerationPass {
    fn id(&self) -> PassId {
        PassId::IrGeneration
    }

    fn name(&self) -> &'static str {
        "IR Generation"
    }

    fn description(&self) -> &'static str {
        "Transforms AST to SmartIR intermediate representation"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Program
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast  // Takes AST, produces IR
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![PassId::SymbolTable]  // Need symbols for resolution
    }
}

impl AnalysisPass for IrGenerationPass {
    fn run_analysis(&self, context: &mut AnalysisContext) -> Result<(), AnalysisError> {
        let symbol_table = context.require_pass_result::<SymbolTable>(PassId::SymbolTable)?;

        // Generate IR from AST
        let mut ir_generator = IrGenerator::new(symbol_table);
        let mut ir_units = Vec::new();

        for source_unit in &context.source_units {
            let ir_unit = ir_generator.generate(source_unit)
                .map_err(|e| AnalysisError::ExecutionFailed(e.to_string()))?;
            ir_units.push(ir_unit);
        }

        // Store IR in context
        context.store_ir(ir_units);

        Ok(())
    }

    fn result_type_name(&self) -> &'static str {
        "Vec<ir::SourceUnit>"
    }
}
```

#### Task 7: IR-Based Analysis Pass Example (Week 8)

**Example**: Implement SSA Construction Pass

**File**: `smarthunt/src/passes/analysis/ir/ssa.rs`

```rust
use crate::engine::pass::{Pass, AnalysisPass, PassId, PassLevel, PassRepresentation, AnalysisError};
use crate::engine::context::AnalysisContext;
use smartir::ssa::{SsaBuilder, SsaForm};

#[derive(Debug)]
pub struct SsaConstructionPass;

impl Pass for SsaConstructionPass {
    fn id(&self) -> PassId {
        PassId::SsaConstruction
    }

    fn name(&self) -> &'static str {
        "SSA Construction"
    }

    fn description(&self) -> &'static str {
        "Converts IR to Static Single Assignment form"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Variable  // Operates on variable level
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ir  // IR-only pass
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![PassId::IrGeneration, PassId::IrCfg]
    }
}

impl AnalysisPass for SsaConstructionPass {
    fn run_analysis(&self, context: &mut AnalysisContext) -> Result<(), AnalysisError> {
        let ir_units = context.ir();
        let ir_cfg = context.require_pass_result::<IrCfgCollection>(PassId::IrCfg)?;

        let mut ssa_builder = SsaBuilder::new();
        let ssa_form = ssa_builder.build(ir_units, ir_cfg)
            .map_err(|e| AnalysisError::ExecutionFailed(e.to_string()))?;

        // Store SSA form in context
        context.store_pass_result(PassId::SsaConstruction, ssa_form);

        Ok(())
    }

    fn result_type_name(&self) -> &'static str {
        "SsaForm"
    }
}
```

#### Task 8: Hybrid Pass Example (Week 15)

**Example**: Reentrancy Detection using both AST and IR

**File**: `smarthunt/src/passes/detection/hybrid/reentrancy.rs`

```rust
use crate::engine::pass::{Pass, BugDetectionPass, PassId, PassLevel, PassRepresentation};
use crate::engine::context::AnalysisContext;
use crate::base::bug::{Bug, BugKind, RiskLevel, ConfidenceLevel};

#[derive(Debug)]
pub struct ReentrancyDetectionPass;

impl Pass for ReentrancyDetectionPass {
    fn id(&self) -> PassId {
        PassId::Reentrancy
    }

    fn name(&self) -> &'static str {
        "Reentrancy Detector"
    }

    fn description(&self) -> &'static str {
        "Detects reentrancy vulnerabilities using AST + IR analysis"
    }

    fn level(&self) -> PassLevel {
        PassLevel::Function
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Hybrid  // Uses both AST and IR
    }

    fn required_passes(&self) -> Vec<PassId> {
        vec![
            PassId::SymbolTable,
            PassId::ModifierAnalysis,  // AST: Identify reentrancy guards
            PassId::IrCfg,
            PassId::IrCallGraph,       // IR: Call graph analysis
            PassId::IrStateMutation,   // IR: State mutation tracking
        ]
    }
}

impl BugDetectionPass for ReentrancyDetectionPass {
    fn detect_bugs(&self, context: &AnalysisContext) -> Vec<Bug> {
        let mut bugs = Vec::new();

        // Step 1: Get AST-level information
        let symbol_table = context.symbol_table().unwrap();
        let modifier_analysis = context.require_pass_result::<ModifierAnalysis>(PassId::ModifierAnalysis).unwrap();

        // Identify functions with reentrancy guards (from AST modifiers)
        let guarded_functions = modifier_analysis.functions_with_nonreentrant_guard();

        // Step 2: Get IR-level information
        let ir_cfg = context.ir_cfg().unwrap();
        let ir_call_graph = context.require_pass_result::<IrCallGraph>(PassId::IrCallGraph).unwrap();
        let state_mutations = context.require_pass_result::<IrStateMutationMap>(PassId::IrStateMutation).unwrap();

        // Step 3: Analyze each function using precise IR flow
        for function in symbol_table.all_functions() {
            // Skip if function has reentrancy guard (AST info)
            if guarded_functions.contains(&function.id) {
                continue;
            }

            // Get function's CFG (IR info)
            let cfg = match ir_cfg.get_function_cfg(function.id) {
                Some(cfg) => cfg,
                None => continue,
            };

            // Check for reentrancy pattern: external call followed by state write
            if let Some(violation) = self.check_reentrancy_pattern(
                function,
                cfg,
                ir_call_graph,
                state_mutations,
                symbol_table,
            ) {
                bugs.push(Bug::new(
                    "reentrancy",
                    format!("Potential reentrancy in function '{}': state modification after external call", function.name),
                    violation.location,
                    RiskLevel::Critical,
                ));
            }
        }

        bugs
    }

    fn bug_kind(&self) -> BugKind {
        BugKind::Vulnerability
    }

    fn risk_level(&self) -> RiskLevel {
        RiskLevel::Critical
    }

    fn confidence(&self) -> ConfidenceLevel {
        ConfidenceLevel::High
    }

    fn cwe_ids(&self) -> Vec<usize> {
        vec![841]  // CWE-841: Improper Enforcement of Behavioral Workflow
    }

    fn swc_ids(&self) -> Vec<usize> {
        vec![107]  // SWC-107: Reentrancy
    }
}

impl ReentrancyDetectionPass {
    fn check_reentrancy_pattern(
        &self,
        function: &Function,
        cfg: &Cfg,
        call_graph: &IrCallGraph,
        state_mutations: &IrStateMutationMap,
        symbols: &SymbolTable,
    ) -> Option<ReentrancyViolation> {
        // Use IR CFG for precise flow analysis
        // 1. Find external calls in IR
        // 2. Check if state writes occur after external calls (IR flow)
        // 3. Use AST for better error messages and source locations

        // Implementation details...
        todo!()
    }
}
```

#### Task 9: Level-Based Traversal Optimization (Weeks 16-17)

**Goal**: Traverse AST once per level, not once per pass

**File**: `smarthunt/src/engine/level_executor.rs`

```rust
/// Execute all passes at a given level with a single AST traversal
pub struct LevelExecutor {
    level: PassLevel,
    passes: Vec<Box<dyn BugDetectionPass>>,
}

impl LevelExecutor {
    pub fn execute(&self, context: &AnalysisContext) -> Vec<Bug> {
        match self.level {
            PassLevel::Expression => self.execute_expression_level(context),
            PassLevel::Statement => self.execute_statement_level(context),
            PassLevel::Function => self.execute_function_level(context),
            PassLevel::Contract => self.execute_contract_level(context),
            // ... other levels
        }
    }

    fn execute_expression_level(&self, context: &AnalysisContext) -> Vec<Bug> {
        let mut visitor = MultiPassExpressionVisitor::new(&self.passes, context);

        for source_unit in &context.source_units {
            visitor.visit_source_unit(source_unit);
        }

        visitor.collect_bugs()
    }
}

/// Visitor that runs multiple expression-level passes in one traversal
struct MultiPassExpressionVisitor<'a> {
    passes: &'a [Box<dyn BugDetectionPass>],
    context: &'a AnalysisContext,
    bugs: Vec<Vec<Bug>>,  // One vec per pass
}

impl<'a, 'b> Visit<'b> for MultiPassExpressionVisitor<'a> {
    fn visit_expr(&mut self, expr: &'b Expr) {
        // Run all expression-level passes on this expression
        for (i, pass) in self.passes.iter().enumerate() {
            // Each pass checks this expression
            let pass_bugs = pass.detect_bugs_for_expr(expr, self.context);
            self.bugs[i].extend(pass_bugs);
        }

        // Continue traversal
        solidity::ast::utils::visit::default::visit_expr(self, expr);
    }
}
```

### 9.3 Testing Milestones

**Milestone 1: Infrastructure Tests** (End of Week 3)
- ✓ Pass trait implementations with representation support compile
- ✓ PassManager can register AST/IR/Hybrid passes
- ✓ Representation-aware DependencyGraph correctly sorts dependencies
- ✓ AnalysisContext stores and retrieves pass results
- ✓ Lazy IR generation works correctly

**Milestone 2: AST Analysis Pass Tests** (End of Week 5)
- ✓ All AST analysis passes produce identical results to old implementation
- ✓ AST passes run in correct dependency order
- ✓ Pass results correctly stored in context
- ✓ No IR generated when only AST passes are enabled

**Milestone 3: IR Infrastructure Tests** (End of Week 7)
- ✓ IR generation produces valid SmartIR from AST
- ✓ IR preserves semantic correctness
- ✓ IR stored correctly in AnalysisContext
- ✓ AST and IR coexist without conflicts

**Milestone 4: IR Analysis Pass Tests** (End of Week 10)
- ✓ All IR analysis passes produce correct results
- ✓ SSA form is valid
- ✓ CFG, def-use chains, taint analysis work correctly
- ✓ IR passes can access IR from context

**Milestone 5: Detection Pass Tests** (End of Week 15)
- ✓ AST-only detectors produce identical results
- ✓ IR-only detectors work correctly
- ✓ Hybrid detectors leverage both representations effectively
- ✓ No regressions in bug detection accuracy
- ✓ All CWE/SWC IDs preserved
- ✓ New IR-based detectors find additional vulnerabilities

**Milestone 6: Performance Tests** (End of Week 18)
- ✓ Cross-representation parallelism achieves 2-4x speedup
- ✓ Within-representation parallelism achieves 2-3x speedup
- ✓ Total speedup: 4-12x vs sequential execution
- ✓ Reduced traversal count improves performance
- ✓ Memory usage acceptable (AST + IR overhead < 30%)
- ✓ Lazy IR generation avoids overhead for AST-only analysis

---

## 10. Benefits & Trade-offs

### 10.1 Benefits

**Performance**:
- ✅ **Cross-Representation Parallelism**: AST and IR passes run simultaneously (2-4x speedup)
- ✅ **Within-Representation Parallelism**: Passes at same level run in parallel (2-3x speedup)
- ✅ **Total Performance Gain**: 4-12x speedup on multi-core systems vs sequential execution
- ✅ **Reduced Traversals**: From 16+ traversals to ~7 per representation
- ✅ **Lazy IR Generation**: IR only generated when needed (zero overhead for AST-only analysis)
- ✅ **Efficient Scheduling**: Dependencies respected across representations, parallelism maximized
- ✅ **Result Caching**: Analysis results computed once, reused by all detectors

**Detection Quality**:
- ✅ **Best of Both Worlds**: Leverage AST for source semantics, IR for precise flow analysis
- ✅ **Enhanced Precision**: IR-based detectors find bugs missed by AST-only analysis
- ✅ **Reduced False Positives**: Precise data flow tracking reduces false alarms
- ✅ **New Bug Classes**: IR enables detection of bugs invisible at AST level (uninitialized storage, SSA violations)
- ✅ **Hybrid Detectors**: Combine high-level semantics (AST) with low-level precision (IR)

**Maintainability**:
- ✅ **Representation Isolation**: AST and IR passes are independent modules
- ✅ **Separation of Concerns**: Analysis vs detection clearly separated; representation-specific logic isolated
- ✅ **Composability**: Passes can be mixed and matched across representations
- ✅ **Testability**: Each pass tested independently; IR correctness separately validated
- ✅ **Extensibility**: Easy to add new passes without modifying core engine

**Correctness**:
- ✅ **Explicit Dependencies**: Passes declare dependencies including representation requirements
- ✅ **Guaranteed Ordering**: Dependency graph ensures correct execution order across representations
- ✅ **IR Validation**: SmartIR validated for semantic correctness
- ✅ **Type Safety**: Pass results type-checked at compile time (with some runtime checks)

**Developer Experience**:
- ✅ **Clear Guidance**: Documentation explains when to use AST vs IR vs Hybrid
- ✅ **Representation Choice**: Developers choose best representation for their detector
- ✅ **Reusable Components**: Analysis passes shared across detectors and representations
- ✅ **Better Error Messages**: Pass failures clearly attributed with representation context
- ✅ **Incremental Adoption**: Can implement AST passes first, add IR later

### 10.2 Trade-offs

**Complexity**:
- ⚠️ **Dual Representation**: Managing both AST and IR adds conceptual complexity
- ⚠️ **Representation Choice**: Developers must choose appropriate representation (mitigated by clear guidelines)
- ⚠️ **More Abstraction**: PassManager adds abstraction layer vs direct detector calls
- ⚠️ **Learning Curve**: Contributors need to understand pass system AND AST vs IR trade-offs
- ⚠️ **Boilerplate**: Each pass needs trait implementations plus representation specification

**Migration Cost**:
- ⚠️ **Extended Timeline**: 22 weeks vs 12 weeks (AST-only migration)
- ⚠️ **IR Development**: Building IR generation and analysis infrastructure
- ⚠️ **Refactoring Effort**: More extensive changes to accommodate dual representation
- ⚠️ **Testing Burden**: Regression tests for AST passes + new tests for IR correctness
- ⚠️ **Potential Bugs**: Migration may introduce new bugs (mitigated by compatibility layer + phased approach)

**Runtime Overhead**:
- ⚠️ **IR Generation Cost**: ~20-30% overhead to generate IR (one-time cost, amortized across IR passes)
- ⚠️ **Memory Overhead**: AST + IR increases memory usage by ~20-30%
- ⚠️ **Scheduling Cost**: More complex scheduling with representation awareness (negligible)
- ⚠️ **Synchronization**: Parallel execution requires Arc/RwLock (minimal contention)

**Mitigation Strategies**:
- ✅ **Lazy IR Generation**: IR only generated when IR passes are enabled (zero overhead for AST-only)
- ✅ **Clear Documentation**: Comprehensive guide on when to use AST vs IR vs Hybrid
- ✅ **Phased Migration**: Incremental rollout reduces risk
- ✅ **Compatibility Layer**: Old detectors continue working during migration
- ✅ **Performance Monitoring**: Detailed profiling identifies bottlenecks

### 10.3 Performance Estimates

**Expected Speedup with Dual Representation** (rough estimates):

**Scenario 1: AST-Only Analysis** (when no IR passes enabled)
- **IR Generation**: Skipped (lazy generation)
- **Analysis Phase**: 1.5-2x faster (from AST pass parallelization)
- **Detection Phase**: 2-3x faster (from reduced traversals + parallelization)
- **Overall**: 2-2.5x faster vs sequential baseline
- **Memory**: No overhead (IR not generated)

**Scenario 2: Full AST + IR Analysis** (all passes enabled)
- **IR Generation**: +30% time cost (one-time, amortized)
- **Analysis Phase**: 3-5x faster (AST + IR passes run in parallel)
- **Detection Phase**: 4-6x faster (AST + IR + hybrid detectors all parallel)
- **Overall**: 4-12x faster vs sequential baseline (despite IR generation overhead)
- **Memory**: +20-30% (both AST and IR in memory)

**Detailed Breakdown**:

| Phase | Sequential (Old) | AST+IR Parallel (New) | Speedup |
|-------|-----------------|----------------------|---------|
| AST Foundation | 1.0s | 0.5s (parallel) | 2x |
| IR Generation | N/A | 0.3s (new cost) | - |
| AST Analysis | 2.0s | 0.8s (parallel) | 2.5x |
| IR Analysis | N/A | 0.9s (parallel with AST) | - |
| AST Detection | 4.0s | 1.5s (parallel + reduced traversals) | 2.7x |
| IR Detection | N/A | 0.8s (parallel with AST) | - |
| Hybrid Detection | 3.0s | 1.2s (parallel + IR precision) | 2.5x |
| **Total** | **10.0s** | **2.0s** | **5x** |

**Performance Factors**:
- **CPU Cores**: More cores → better parallelism (8+ cores recommended)
- **Contract Size**: Larger contracts → more benefit from reduced traversals and parallel analysis
- **Detector Mix**: More IR/hybrid detectors → better amortization of IR generation cost
- **Memory**: Sufficient RAM for AST + IR (typically not a constraint)

**Real-World Expectations**:
- **Small contracts** (< 500 LOC): 2-3x speedup (overhead dominates)
- **Medium contracts** (500-2000 LOC): 4-6x speedup (sweet spot)
- **Large contracts** (> 2000 LOC): 6-12x speedup (maximum benefit)
- **Contract suites**: 8-15x speedup (amortization + caching)

---

## 11. Future Extensions

### 11.1 Incremental Analysis (Post-MVP)

**Concept**: Only re-run passes affected by code changes (AST or IR)

**Benefits**:
- Near-instant analysis for small edits (IDE integration)
- Efficient continuous analysis in development

**Implementation**:
- Track which AST nodes changed
- Invalidate AST passes that depend on changed nodes
- Invalidate IR (forces regeneration if IR passes are needed)
- Re-run only invalidated passes
- Smart IR caching: Only regenerate affected functions

**Representation-Specific Optimizations**:
- AST changes → Invalidate AST passes + IR
- IR regeneration → Incremental (only affected functions)
- Detection passes → Re-run based on invalidated analysis passes

### 11.2 IR-Based Optimizations (Post-MVP)

**Concept**: Use IR for code optimization suggestions

**Applications**:
- **Gas Optimization**: Identify redundant operations, inefficient patterns at IR level
- **Code Simplification**: Suggest simplifications based on IR analysis
- **Dead Code Elimination**: Precise identification of unreachable code
- **Constant Propagation**: Track constant values through IR SSA form

**Example**:
```rust
// Detector finds this in IR:
let x = SLOAD(slot);  // Load from storage
let y = ADD(x, 0);    // Add zero (no-op)
SSTORE(slot, y);      // Store back

// Suggest optimization:
// Remove ADD(x, 0) - identity operation
// Result: Same behavior, less gas
```

### 11.3 Cross-Contract Analysis (Post-MVP)

**Concept**: Extend IR-based analysis across contract boundaries

**Benefits**:
- Detect vulnerabilities in contract interactions
- Analyze delegatecall chains
- Track taint flow across contracts

**Implementation**:
- Build inter-contract call graph using IR
- Extend taint analysis across contract boundaries
- Detect reentrancy in complex multi-contract systems

### 11.4 Formal Verification Integration (Long-term)

**Concept**: Use IR as bridge to formal verification tools

**Benefits**:
- IR is closer to verification tool input formats
- SMT solver integration for path constraints
- Symbolic execution on IR

**Potential**:
- Generate SMT constraints from IR CFG
- Prove absence of bugs formally
- Exhaustive path exploration

### 11.5 Custom Pass Pipelines (Post-MVP)

**Concept**: Users define custom analysis pipelines

**Example**:
```rust
let mut manager = PassManager::new(config);
manager.register_default_passes();

// Custom pipeline: only run reentrancy detection
let pipeline = Pipeline::builder()
    .add_pass(PassId::SymbolTable)
    .add_pass(PassId::CFG)
    .add_pass(PassId::CallGraph)
    .add_pass(PassId::StateMutation)
    .add_pass(PassId::Reentrancy)
    .build();

let report = manager.run_pipeline(pipeline, &mut context)?;
```

**Representation-Specific Pipelines**:
```rust
// AST-only pipeline (fast, no IR overhead)
let ast_pipeline = Pipeline::builder()
    .add_ast_passes()
    .build();

// IR-only pipeline (for specific bug classes)
let ir_pipeline = Pipeline::builder()
    .add_pass(PassId::IrGeneration)
    .add_ir_analysis_passes()
    .add_pass(PassId::UninitializedStorage)
    .add_pass(PassId::DeadCode)
    .build();

// Full pipeline (all detectors)
let full_pipeline = Pipeline::default();
```

### 11.6 Pass Profiling & Optimization (Post-MVP)

**Concept**: Detailed profiling to optimize slow passes

**Metrics**:
- Pass execution time (broken down by AST/IR/Hybrid)
- Memory usage per pass (AST vs IR overhead)
- Cache hit/miss rates
- Parallelization efficiency (cross-representation vs within-representation)
- IR generation time
- Representation-specific statistics

**Tool**:
```bash
smarthunt analyze --profile contract.sol

# Output: Detailed timing report
# ========================================
# Pass Execution Report
# ========================================
# AST Passes (3.2s total):
#   SymbolTable: 0.8s
#   TypeIndex: 0.4s
#   Inheritance: 2.0s (BOTTLENECK)
#
# IR Generation: 1.5s
#
# IR Passes (2.1s total, 40% parallel overlap with AST):
#   IrCfg: 0.5s
#   SsaConstruction: 0.8s
#   TaintAnalysis: 0.8s
#
# Detection Passes (4.2s total, 95% parallel):
#   [... detector timings ...]
#
# Cross-Representation Parallelism: 2.3x speedup
# Total Analysis Time: 6.8s (vs 28.3s sequential)
# Overall Speedup: 4.2x
```

### 11.7 Language Server Protocol (LSP) Integration (Post-MVP)

**Concept**: Integrate pass-based analysis into IDE with incremental updates

**Features**:
- Real-time bug detection as you type
- Incremental analysis on file save (AST changes → smart IR updates)
- Code actions (quick fixes) from detectors
- Representation-aware caching (AST cached separately from IR)
- Hover information showing bug details and fixes

**Performance**:
- Sub-100ms response for AST-only detectors
- Background IR regeneration for IR-based detectors
- Smart prioritization: Run fast AST detectors first, queue IR detectors

### 11.8 Multi-Language Support (Long-term)

**Concept**: Extend pass framework to support Vyper, Yul, Cairo, etc.

**Implementation**:
- Abstract AST types behind traits
- Language-specific AST passes
- **Shared IR**: All languages compile to common SmartIR
- Unified IR analysis passes (work across all languages)
- Language-specific detectors use language AST; universal detectors use IR

**Architecture**:
```
Solidity AST ─→ SmartIR ─┐
Vyper AST ───→ SmartIR ─┤→ Unified IR Analysis → Universal Detectors
Yul AST ─────→ SmartIR ─┘

Language-specific AST detectors ──→ Language-specific bugs
Unified IR detectors ──────────────→ Universal vulnerabilities
```

**Benefits**:
- Write IR-based detectors once, work for all languages
- Consistent bug detection across language ecosystems
- Reduced maintenance burden (one IR vs multiple ASTs)

---

## Conclusion

This plan outlines a comprehensive refactoring of SmartHunt to a **unified LLVM-inspired pass-based architecture operating on both AST and IR representations**, with a critical architectural decision to **separate analysis infrastructure (solidity crate) from bug detection (smarthunt crate)** for maximum reusability.

### Key Outcomes

The migration will deliver:

1. **Dramatically Improve Performance**: 4-12x speedup from cross-representation parallelism, within-representation parallelism, and reduced traversals
2. **Enhance Bug Detection Quality**: Leverage AST for high-level semantics and IR for precise flow analysis; detect new bug classes invisible at AST level
3. **Enable Reusability Across Tools**: Analysis infrastructure in solidity crate shared by smarthunt (bug detection), smartproof (verification), and future tools
4. **Improve Maintainability**: Clear separation of concerns - analysis (solidity) vs application (smarthunt/smartproof)
5. **Enable Future Extensions**: Incremental analysis, custom pipelines, IDE integration, advanced IR-based optimizations
6. **Preserve Correctness**: Comprehensive regression testing ensures no loss of detection accuracy; IR validated for semantic correctness

### Key Innovations

**Architectural**:
- **Crate Separation**: Analysis framework (solidity) separate from applications (smarthunt, smartproof)
- **Dual Representation**: First smart contract analyzer to systematically leverage both AST and IR
- **Hybrid Passes**: Unique ability to combine high-level source semantics with low-level flow precision

**Technical**:
- **Lazy IR Generation**: Zero overhead when IR not needed
- **Representation-Aware Scheduling**: Optimized parallel execution across representations
- **Pass Infrastructure**: Reusable by any tool needing Solidity analysis

### Crate Organization Summary

```
┌─────────────────────────────────────────┐
│     solidity Crate                      │
│  (Analysis Framework - Reusable)        │
│  - Pass infrastructure                  │
│  - AST & IR representations             │
│  - Analysis passes (AST, IR, Hybrid)    │
└────────────┬────────────────────────────┘
             │ provides analysis to
             │
    ┌────────┴──────────┬─────────────────┐
    │                   │                 │
┌───▼──────────┐  ┌─────▼────────┐  ┌────▼────────┐
│  smarthunt   │  │ smartproof   │  │Future Tools │
│(Bug Detect)  │  │(Verification)│  │             │
└──────────────┘  └──────────────┘  └─────────────┘
```

### Timeline

**22 weeks** for complete migration with crate separation:
- **Weeks 1-5**: solidity crate foundation + AST infrastructure
- **Weeks 6-7**: solidity crate IR infrastructure (complete IR generation)
- **Weeks 8-10**: solidity crate IR analysis passes
- **Weeks 11-15**: smarthunt bug detection migration
- **Weeks 16-22**: Optimization + documentation

### Recommendation

Proceed with phased migration, starting with:
1. **Weeks 1-3**: Build solidity crate foundation to validate architecture
2. **Week 7**: Critical milestone - complete IR generation in solidity crate
3. **Week 10**: Validate solidity crate can support smarthunt needs

**Key Validation Points**:
- Week 3: solidity pass infrastructure works
- Week 7: IR generation complete and validated
- Week 10: IR analysis passes functional
- Week 15: smarthunt fully migrated to use solidity

### Success Metrics

**Performance**:
- ✅ 4-12x performance improvement (measured on benchmark suite)
- ✅ Parallel AST + IR analysis achieves theoretical maximum

**Detection Quality**:
- ✅ Detect 20-30% more vulnerabilities (from IR-based detectors)
- ✅ Zero regression in existing AST-based detectors

**Reusability**:
- ✅ smartproof can use solidity analysis (zero duplication)
- ✅ Future tools can leverage solidity framework
- ✅ Analysis improvements benefit all tools automatically

**Code Quality**:
- ✅ < 30% memory overhead
- ✅ Clean, well-documented API for external contributors
- ✅ Clear crate boundaries and responsibilities
