# Plan: Merge Analysis Module into SmartHunt

**Date**: 2026-02-06
**Status**: Planning
**Author**: Architecture Review

## Executive Summary

This document outlines the plan to migrate the `solidity/src/analysis` module into SmartHunt and establish a comprehensive data flow analysis framework. The goal is to enable bug detection passes that leverage both **data flow analysis** (taint tracking, SSA, def-use chains) and **AST/IR pattern matching** (structural detection).

## 1. Current State Analysis

### 1.1 SmartHunt Architecture (Current)

**Location**: `smarthunt/src/`

**Key Components**:
- **Detection Framework**: Pass-based bug detection using the solidity crate's `Pass` trait
- **16 AST-based Detectors**: Mostly stubs, only `TxOriginDetector` and partial `CeiViolationDetector` implemented
- **DetectorRegistry**: Registration and lookup of bug detectors
- **DetectionManager**: Orchestrates detector execution (sequential/parallel)
- **Output Formatters**: JSON, Markdown, SARIF, Text

**Detection Pattern**:
```rust
pub trait BugDetectionPass: Pass {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>>;
    // metadata methods...
}
```

**Current Limitations**:
- ❌ No data flow analysis infrastructure
- ❌ Most detectors are stubs without implementation
- ❌ IR-based detection not yet enabled
- ❌ No taint analysis, SSA, or def-use chains
- ❌ Limited cross-procedural analysis
- ❌ No control flow graph construction
- ✅ Good foundation with pass-based architecture
- ✅ Comprehensive metadata (CWE, SWC, confidence, risk)

### 1.2 Analysis Module Architecture (Current)

**Location**: `solidity/src/analysis/`

**Key Components**:
- **Pass Framework**: Generic pass infrastructure with dependency resolution
- **PassManager**: Orchestrates pass registration, scheduling, and execution
- **PassScheduler**: Computes execution order via topological sort
- **PassExecutor**: Sequential/parallel execution engine
- **Analysis Passes**:
  - ✅ **SymbolTable**: Fast entity lookup (contracts, functions, variables)
  - ✅ **TypeIndex**: Type information indexing
  - ✅ **CallGraph**: Function call relationships (direct, external, delegate)
  - ✅ **InheritanceGraph**: Contract inheritance with C3 linearization
  - ✅ **ModifierAnalysis**: Modifier usage and access control patterns
  - ⏳ **IR Generation**: AST → IR conversion (planned)
  - ⏳ **Control Flow**: CFG, dominators, loops (planned)
  - ⏳ **Data Flow**: SSA, def-use, liveness, taint (planned)

**Capabilities**:
- ✅ Dependency-based scheduling (topological sort)
- ✅ Lazy IR generation (on-demand)
- ✅ Artifact storage system (type-safe, dynamic)
- ✅ Pass completion tracking
- ✅ Parallel execution support
- ✅ Multi-representation (AST/IR/Hybrid)

## 2. Integration Strategy

### 2.1 High-Level Goals

1. **Merge Analysis Infrastructure**: Bring `PassManager`, `PassScheduler`, `PassExecutor` into SmartHunt
2. **Preserve Bug Detection Interface**: Keep `BugDetectionPass` trait as the primary detector API
3. **Enable Data Flow Analysis**: Implement IR-based data flow passes
4. **Unified Pass System**: Bug detectors become specialized analysis passes
5. **Two-Layer Architecture**:
   - **Layer 1**: Analysis passes (symbol table, call graph, CFG, data flow)
   - **Layer 2**: Bug detection passes (depend on Layer 1 artifacts)

### 2.2 Directory Structure (Target)

```
smarthunt/
├── src/
│   ├── lib.rs
│   ├── main.rs
│   ├── config.rs
│   │
│   ├── analysis/                    # NEW: Core analysis framework
│   │   ├── mod.rs
│   │   ├── context.rs               # Migrated from solidity
│   │   ├── pass.rs                  # Migrated & extended
│   │   ├── pass_id.rs               # Migrated & extended
│   │   ├── manager.rs               # Migrated: PassManager
│   │   ├── scheduler.rs             # Migrated: PassScheduler
│   │   ├── executor.rs              # Migrated: PassExecutor
│   │   ├── dependency.rs            # Migrated: DependencyGraph
│   │   │
│   │   └── passes/                  # Analysis passes
│   │       ├── ast/
│   │       │   ├── symbol_table.rs  # Migrated
│   │       │   ├── type_index.rs    # Migrated
│   │       │   ├── call_graph.rs    # Migrated
│   │       │   ├── inheritance.rs   # Migrated
│   │       │   └── modifier.rs      # Migrated
│   │       │
│   │       ├── ir/
│   │       │   ├── cfg.rs           # NEW: Control flow graph
│   │       │   ├── dominator.rs     # NEW: Dominator tree
│   │       │   ├── loops.rs         # NEW: Loop detection
│   │       │   └── ssa.rs           # NEW: SSA construction
│   │       │
│   │       └── dataflow/            # NEW: Data flow analysis framework
│   │           ├── mod.rs           # Module exports
│   │           ├── framework.rs     # Core DFA framework
│   │           │                    #   - DataFlowFact trait
│   │           │                    #   - TransferFunction trait
│   │           │                    #   - DataFlowSolver (worklist algorithm)
│   │           │                    #   - AnalysisDirection enum
│   │           ├── def_use.rs       # Reaching definitions analysis
│   │           ├── liveness.rs      # Variable liveness analysis
│   │           ├── taint.rs         # Taint propagation analysis
│   │           ├── state_mutation.rs # State change tracking
│   │           └── call_context.rs  # Call context sensitivity (future)
│   │
│   ├── detection/                   # UPDATED: Bug detection framework
│   │   ├── mod.rs
│   │   ├── pass.rs                  # BugDetectionPass trait (extends Pass)
│   │   ├── manager.rs               # DetectionManager (uses PassManager)
│   │   ├── registry.rs              # DetectorRegistry
│   │   │
│   │   └── detectors/
│   │       ├── ast/                 # AST-based detectors (existing)
│   │       │   ├── tx_origin.rs
│   │       │   ├── cei_violation.rs
│   │       │   └── ...
│   │       │
│   │       ├── ir/                  # NEW: IR-based detectors
│   │       │   ├── dead_code.rs
│   │       │   ├── uninitialized_storage.rs
│   │       │   └── integer_overflow.rs
│   │       │
│   │       └── hybrid/              # NEW: Hybrid detectors
│   │           ├── reentrancy.rs    # AST + CFG + Taint
│   │           ├── access_control.rs # AST + CallGraph + Modifiers
│   │           └── state_consistency.rs
│   │
│   └── output/
│       └── ...
```

## 3. Data Flow Analysis Framework Design

### 3.0 Framework Overview

The data flow analysis (DFA) framework provides a **generic, reusable infrastructure** for implementing various data flow analyses. It follows the classic **worklist algorithm** approach with lattice-based abstract interpretation.

**Key Design Principles**:
- **Generic Framework**: One solver implementation works for all analyses
- **Lattice-Based**: Data flow facts form a lattice (bottom, top, meet, partial order)
- **Pluggable Transfer Functions**: Each analysis defines how statements transform facts
- **Direction Agnostic**: Supports both forward and backward analysis
- **CFG-Based**: Operates on control flow graphs from IR
- **Fixpoint Iteration**: Worklist algorithm ensures convergence

**Framework Layers**:
1. **Core Framework** (`framework.rs`): Generic solver and trait definitions
2. **Specific Analyses** (`taint.rs`, `def_use.rs`, etc.): Concrete implementations
3. **Analysis Passes**: Integration with PassManager and artifact storage
4. **Bug Detectors**: Consumers of data flow analysis results

**Integration with Pass System**:
```
┌─────────────────────────────────────────────────────────────┐
│ PassManager (orchestrates all passes)                      │
└─────────────────────────────────────────────────────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        ▼                  ▼                  ▼
┌───────────────┐  ┌────────────────┐  ┌──────────────────┐
│ AST Passes    │  │ IR Passes      │  │ Data Flow Passes │
│               │  │                │  │                  │
│ • SymbolTable │  │ • CFG          │  │ • TaintAnalysis  │
│ • CallGraph   │  │ • Dominators   │  │ • DefUseChain    │
│ • TypeIndex   │  │ • SSA          │  │ • Liveness       │
│ • Inheritance │  │ • Loops        │  │ • StateMutation  │
└───────────────┘  └────────────────┘  └──────────────────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           ▼
                  ┌──────────────────┐
                  │ AnalysisContext  │
                  │ (artifact store) │
                  └──────────────────┘
                           │
                           ▼
                  ┌──────────────────┐
                  │ Bug Detectors    │
                  │                  │
                  │ • Reentrancy     │
                  │ • AccessControl  │
                  │ • UninitStorage  │
                  │ • DeadCode       │
                  └──────────────────┘
```

**Supported Analyses**:

| Analysis | Direction | Use Case | Bug Detectors |
|----------|-----------|----------|---------------|
| **Taint Analysis** | Forward | Track untrusted data sources to security-critical sinks | Reentrancy, Unsafe delegatecall, Unchecked input |
| **Def-Use Chains** | Forward | Link variable definitions to their uses | Uninitialized storage, Dead assignments |
| **Liveness Analysis** | Backward | Identify live variables at each program point | Dead code, Unused variables |
| **State Mutation** | Hybrid | Track state reads/writes inter-procedurally | CEI violations, Access control, View/pure validation |
| **SSA Construction** | Transform | Convert to Static Single Assignment form | Enables more precise analyses (future) |
| **Constant Propagation** | Forward | Track compile-time constant values | Integer overflow, Division by zero (future) |
| **Alias Analysis** | Forward | Track pointer aliasing relationships | Complex storage bugs (future) |

### 3.1 Framework Architecture

The data flow analysis framework will support **forward** and **backward** analysis with **meet operators** for combining data flow facts.

**Module Location**: `smarthunt/src/analysis/passes/dataflow/`

**Architecture Overview**:
```
dataflow/
├── mod.rs              # Public exports and module documentation
├── framework.rs        # Core generic DFA framework
│                       #   - DataFlowFact trait (lattice elements)
│                       #   - TransferFunction trait (analysis-specific logic)
│                       #   - DataFlowSolver (worklist algorithm)
│                       #   - AnalysisDirection (Forward/Backward)
│
├── def_use.rs          # Reaching definitions analysis
│                       #   - DefUseChainPass
│                       #   - Definition, Use structs
│                       #   - DefUseChain artifact
│
├── liveness.rs         # Variable liveness analysis
│                       #   - LivenessAnalysisPass
│                       #   - LiveSet (DataFlowFact implementation)
│                       #   - Backward data flow
│
├── taint.rs            # Taint propagation analysis
│                       #   - TaintAnalysisPass
│                       #   - TaintSource, TaintSink enums
│                       #   - TaintStatus (DataFlowFact implementation)
│                       #   - TaintMap artifact
│
├── state_mutation.rs   # State mutation tracking
│                       #   - StateMutationPass
│                       #   - StateMutation, StateWrite, StateRead
│                       #   - Inter-procedural analysis
│
└── call_context.rs     # Call context sensitivity (future enhancement)
                        #   - Context-sensitive analysis
                        #   - Call string approach
```

#### 3.1.1 Core Framework (framework.rs)

**Purpose**: Provide generic infrastructure for all data flow analyses

```rust
// Generic data flow analysis framework

/// Lattice element for data flow facts
pub trait DataFlowFact: Clone + Eq + Debug + Send + Sync {
    /// Bottom element (initial state)
    fn bottom() -> Self;

    /// Top element (unknown/any state)
    fn top() -> Self;

    /// Meet operator (join in lattice)
    fn meet(&self, other: &Self) -> Self;

    /// Partial order check
    fn less_or_equal(&self, other: &Self) -> bool;
}

/// Transfer function for data flow analysis
pub trait TransferFunction<F: DataFlowFact>: Send + Sync {
    /// Apply transfer function to a statement/instruction
    fn transfer(&self, stmt: &IrStmt, fact: &F) -> F;
}

/// Data flow analysis direction
pub enum AnalysisDirection {
    Forward,   // Gen-kill: flow from entry to exit
    Backward,  // Use-def: flow from exit to entry
}

/// Generic data flow solver
pub struct DataFlowSolver<F: DataFlowFact, T: TransferFunction<F>> {
    direction: AnalysisDirection,
    transfer: T,
    cfg: Arc<ControlFlowGraph>,
    worklist: VecDeque<BasicBlockId>,
    facts: HashMap<BasicBlockId, F>,
    phantom: PhantomData<F>,
}

impl<F: DataFlowFact, T: TransferFunction<F>> DataFlowSolver<F, T> {
    /// Run iterative worklist algorithm until fixpoint
    pub fn solve(&mut self) -> DataFlowResult<HashMap<BasicBlockId, F>> {
        // Worklist algorithm implementation
    }
}
```

#### 3.1.2 Framework Design Patterns

**How to implement a new data flow analysis**:

1. **Define your data flow fact** (lattice element):
   ```rust
   #[derive(Clone, Eq, PartialEq, Debug)]
   pub struct MyFact {
       // your data
   }

   impl DataFlowFact for MyFact {
       fn bottom() -> Self { /* initial state */ }
       fn top() -> Self { /* unknown state */ }
       fn meet(&self, other: &Self) -> Self { /* combine facts */ }
       fn less_or_equal(&self, other: &Self) -> bool { /* partial order */ }
   }
   ```

2. **Define your transfer function**:
   ```rust
   pub struct MyTransfer {
       // analysis configuration
   }

   impl TransferFunction<MyFact> for MyTransfer {
       fn transfer(&self, stmt: &IrStmt, fact: &MyFact) -> MyFact {
           // Analyze statement and update fact
           match stmt {
               IrStmt::Assign(var, expr) => { /* handle assignment */ },
               IrStmt::Call(func, args) => { /* handle call */ },
               // ...
           }
       }
   }
   ```

3. **Create analysis pass**:
   ```rust
   pub struct MyAnalysisPass;

   impl AnalysisPass for MyAnalysisPass {
       fn dependencies(&self) -> Vec<PassId> {
           vec![PassId::IrGeneration, PassId::IrCfg]
       }

       fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
           for function in context.functions() {
               let cfg = context.get_artifact::<ControlFlowGraph>(&format!("cfg_{}", function.id))?;

               // Create solver
               let transfer = MyTransfer::new();
               let mut solver = DataFlowSolver::new(
                   AnalysisDirection::Forward,
                   transfer,
                   cfg,
               );

               // Solve
               let results = solver.solve()?;

               // Store artifact
               context.store_artifact(&format!("my_analysis_{}", function.id), results);
           }
           Ok(())
       }
   }
   ```

4. **Use in bug detectors**:
   ```rust
   impl BugDetectionPass for MyDetector {
       fn dependencies(&self) -> Vec<PassId> {
           vec![PassId::MyAnalysis]
       }

       fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
           let my_results = context.get_artifact::<MyResults>("my_analysis_...")?;
           // Use results to detect bugs
       }
   }
   ```

#### 3.1.3 Framework Component Relationships

**Dependency Flow**:
```
ControlFlowGraph (from IrCfg pass)
        ↓
DataFlowSolver<F, T>
    ├── uses: DataFlowFact trait (F)
    ├── uses: TransferFunction trait (T)
    └── uses: AnalysisDirection
        ↓
    Worklist Algorithm
    (iterative fixpoint)
        ↓
Results (HashMap<BasicBlockId, F>)
        ↓
Analysis Pass
(stores artifact in AnalysisContext)
        ↓
Bug Detection Pass
(retrieves artifact, detects patterns)
```

**Component Interaction**:
```
┌─────────────────────────────────────────────────────────┐
│ DataFlowSolver<F, T>                                    │
│                                                         │
│  ┌─────────────┐    ┌──────────────┐                   │
│  │  Worklist   │───▶│  Visit Block │                   │
│  │ (BasicBlock)│    └──────────────┘                   │
│  └─────────────┘           │                           │
│         ▲                  ▼                            │
│         │         ┌─────────────────┐                  │
│         │         │ For each stmt:  │                  │
│         │         │ fact = transfer │                  │
│         │         │   .transfer()   │                  │
│         │         └─────────────────┘                  │
│         │                  │                           │
│         │                  ▼                            │
│         │         ┌─────────────────┐                  │
│         │         │ fact.meet()     │                  │
│         │         │ with old fact   │                  │
│         │         └─────────────────┘                  │
│         │                  │                           │
│         │                  ▼                            │
│         │         ┌─────────────────┐                  │
│         │         │ Changed?        │                  │
│         └─────────│ Add successors  │                  │
│                   │ to worklist     │                  │
│                   └─────────────────┘                  │
└─────────────────────────────────────────────────────────┘
```

**Concrete Example (Taint Analysis)**:
```
TaintAnalysisPass
    └─ For each function:
        ├─ Get CFG from context
        ├─ Create TaintTransfer
        ├─ Create DataFlowSolver<TaintStatus, TaintTransfer>
        ├─ solver.solve() → HashMap<BasicBlockId, TaintStatus>
        └─ Store TaintMap artifact
```

#### 3.1.4 Control Flow Graph (CFG)

```rust
/// Basic block in control flow graph
pub struct BasicBlock {
    pub id: BasicBlockId,
    pub stmts: Vec<IrStmt>,
    pub terminator: Terminator,
    pub predecessors: Vec<BasicBlockId>,
    pub successors: Vec<BasicBlockId>,
}

/// Control flow graph for a function
pub struct ControlFlowGraph {
    pub function: FunctionId,
    pub blocks: HashMap<BasicBlockId, BasicBlock>,
    pub entry: BasicBlockId,
    pub exit: BasicBlockId,
}

/// CFG construction pass
pub struct CfgPass;

impl AnalysisPass for CfgPass {
    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        // Build CFG for each function in IR
        // Store as artifact: "cfg_<function_id>"
    }
}
```

### 3.2 Specific Data Flow Analyses

#### 3.2.1 Taint Analysis

**Purpose**: Track untrusted data flow from sources to sinks

```rust
/// Taint source (e.g., user input, external calls)
pub enum TaintSource {
    UserInput,           // msg.data, calldata parameters
    ExternalCall(Loc),   // Return value from external call
    StorageRead(Loc),    // Storage variable read (mutable state)
    Environment,         // block.timestamp, msg.sender, etc.
}

/// Taint sink (e.g., security-critical operations)
pub enum TaintSink {
    Send(Loc),           // send(), transfer(), call{value}()
    Delegatecall(Loc),   // delegatecall()
    Selfdestruct(Loc),   // selfdestruct()
    StorageWrite(Loc),   // State variable write
    ExternalCall(Loc),   // External contract call
}

/// Taint status for variables
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TaintStatus {
    Untainted,           // No taint
    Tainted {
        sources: Vec<TaintSource>,
        path: Vec<Loc>,  // Propagation path
    },
}

impl DataFlowFact for TaintStatus {
    fn bottom() -> Self { TaintStatus::Untainted }
    fn top() -> Self { TaintStatus::Tainted { sources: vec![], path: vec![] } }
    fn meet(&self, other: &Self) -> Self {
        // Union of taint sources
    }
}

/// Taint analysis pass
pub struct TaintAnalysisPass {
    sources: Vec<TaintSource>,
    sinks: Vec<TaintSink>,
}

impl AnalysisPass for TaintAnalysisPass {
    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::IrGeneration, PassId::IrCfg]
    }

    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        // For each function:
        //   1. Build CFG
        //   2. Identify taint sources and sinks
        //   3. Run forward data flow to propagate taints
        //   4. Store TaintMap artifact
    }
}

/// Result artifact
pub struct TaintMap {
    /// Variable → Taint status at each program point
    pub taints: HashMap<(BasicBlockId, VarId), TaintStatus>,

    /// Flows from sources to sinks
    pub flows: Vec<TaintFlow>,
}

pub struct TaintFlow {
    pub source: TaintSource,
    pub sink: TaintSink,
    pub path: Vec<Loc>,
}
```

#### 3.2.2 Def-Use Chains

**Purpose**: Track where variables are defined and used

```rust
/// Definition site
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Definition {
    pub var: VarId,
    pub block: BasicBlockId,
    pub stmt_index: usize,
    pub loc: Loc,
}

/// Use site
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Use {
    pub var: VarId,
    pub block: BasicBlockId,
    pub stmt_index: usize,
    pub loc: Loc,
}

/// Def-Use chain artifact
pub struct DefUseChain {
    /// Definition → Uses
    pub def_uses: HashMap<Definition, Vec<Use>>,

    /// Use → Definitions
    pub use_defs: HashMap<Use, Vec<Definition>>,

    /// All definitions per variable
    pub defs_by_var: HashMap<VarId, Vec<Definition>>,
}

/// Reaching definitions analysis
pub struct DefUseChainPass;

impl AnalysisPass for DefUseChainPass {
    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::IrGeneration, PassId::IrCfg]
    }

    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        // For each function:
        //   1. Compute reaching definitions (forward data flow)
        //   2. Build def-use and use-def maps
        //   3. Store DefUseChain artifact
    }
}
```

#### 3.2.3 State Mutation Tracking

**Purpose**: Track which functions modify contract state

```rust
/// State mutation information
#[derive(Clone, Debug)]
pub struct StateMutation {
    pub function: FunctionId,
    pub writes: Vec<StateWrite>,
    pub reads: Vec<StateRead>,
    pub is_pure: bool,
    pub is_view: bool,
}

pub struct StateWrite {
    pub variable: VarId,
    pub loc: Loc,
    pub is_conditional: bool,  // Inside if/loop
}

pub struct StateRead {
    pub variable: VarId,
    pub loc: Loc,
}

/// Inter-procedural state mutation analysis
pub struct StateMutationPass;

impl AnalysisPass for StateMutationPass {
    fn dependencies(&self) -> Vec<PassId> {
        vec![
            PassId::SymbolTable,
            PassId::CallGraph,
            PassId::IrGeneration,
        ]
    }

    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        // 1. Intra-procedural: identify direct state reads/writes
        // 2. Inter-procedural: propagate through call graph
        // 3. Validate view/pure modifiers
        // 4. Store StateMutationMap artifact
    }
}
```

#### 3.2.4 SSA Construction

**Purpose**: Convert IR to Static Single Assignment form for precise data flow

```rust
/// SSA variable (original variable + version)
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SsaVar {
    pub original: VarId,
    pub version: usize,
}

/// Phi function (merge point)
pub struct PhiFunction {
    pub dest: SsaVar,
    pub sources: Vec<(BasicBlockId, SsaVar)>,  // (predecessor, value)
}

/// SSA form of a function
pub struct SsaFunction {
    pub function: FunctionId,
    pub blocks: HashMap<BasicBlockId, SsaBasicBlock>,
    pub phi_functions: HashMap<BasicBlockId, Vec<PhiFunction>>,
}

/// SSA construction pass
pub struct SsaConstructionPass;

impl AnalysisPass for SsaConstructionPass {
    fn dependencies(&self) -> Vec<PassId> {
        vec![
            PassId::IrGeneration,
            PassId::IrCfg,
            PassId::DominatorTree,
        ]
    }

    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        // 1. Compute dominance frontiers
        // 2. Insert phi functions
        // 3. Rename variables
        // 4. Store SsaMap artifact
    }
}
```

#### 3.2.5 Liveness Analysis

**Purpose**: Determine which variables are live at each program point

```rust
/// Liveness information
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct LiveSet {
    pub live_in: HashSet<VarId>,   // Live before block
    pub live_out: HashSet<VarId>,  // Live after block
}

impl DataFlowFact for LiveSet {
    fn bottom() -> Self {
        LiveSet {
            live_in: HashSet::new(),
            live_out: HashSet::new(),
        }
    }

    fn meet(&self, other: &Self) -> Self {
        LiveSet {
            live_in: &self.live_in | &other.live_in,
            live_out: &self.live_out | &other.live_out,
        }
    }
}

/// Liveness analysis pass (backward analysis)
pub struct LivenessAnalysisPass;

impl AnalysisPass for LivenessAnalysisPass {
    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::IrGeneration, PassId::IrCfg]
    }

    fn run(&self, context: &mut AnalysisContext) -> PassResult<()> {
        // Backward data flow analysis:
        //   live_in[B] = use[B] ∪ (live_out[B] - def[B])
        //   live_out[B] = ∪(live_in[S]) for all successors S
    }
}
```

### 3.3 Data Flow Analysis Integration Pattern

**How bug detectors use data flow analysis:**

```rust
pub struct ReentrancyDetector {
    // detector configuration
}

impl BugDetectionPass for ReentrancyDetector {
    fn dependencies(&self) -> Vec<PassId> {
        vec![
            PassId::SymbolTable,
            PassId::CallGraph,
            PassId::IrCfg,
            PassId::TaintAnalysis,
            PassId::StateMutation,
        ]
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let call_graph = context.call_graph()?;
        let taint_map = context.get_artifact::<TaintMap>("taint_map")?;
        let state_mutations = context.get_artifact::<StateMutationMap>("state_mutations")?;

        let mut bugs = Vec::new();

        // Pattern: External call → State write
        for function in context.functions() {
            let cfg = context.get_artifact::<ControlFlowGraph>(&format!("cfg_{}", function.id))?;

            for block in cfg.blocks.values() {
                // Look for pattern:
                //   1. External call (tainted)
                //   2. State write after the call
                //   3. No reentrancy guard

                if let Some(bug) = self.check_reentrancy_pattern(
                    block,
                    taint_map,
                    state_mutations,
                    context,
                ) {
                    bugs.push(bug);
                }
            }
        }

        Ok(bugs)
    }
}
```

## 4. Migration Phases

### Phase 1: Infrastructure Migration (Week 1-2)

**Goal**: Move core analysis framework from `solidity` to `smarthunt`

**Tasks**:
1. Create `smarthunt/src/analysis/` directory structure
2. Migrate core framework files:
   - [context.rs](smarthunt/src/analysis/context.rs) - Analysis context
   - [pass.rs](smarthunt/src/analysis/pass.rs) - Pass trait and types
   - [pass_id.rs](smarthunt/src/analysis/pass_id.rs) - Pass identifiers
   - [manager.rs](smarthunt/src/analysis/manager.rs) - PassManager
   - [scheduler.rs](smarthunt/src/analysis/scheduler.rs) - PassScheduler
   - [executor.rs](smarthunt/src/analysis/executor.rs) - PassExecutor
   - [dependency.rs](smarthunt/src/analysis/dependency.rs) - DependencyGraph
3. Update imports to reference smarthunt's analysis module
4. Ensure `BugDetectionPass` extends `Pass` trait
5. Update `DetectionManager` to use `PassManager` internally
6. Run integration tests

**Success Criteria**:
- ✅ All analysis framework code compiles in smarthunt
- ✅ Existing TxOriginDetector still works
- ✅ PassManager can run analysis passes
- ✅ No regression in existing functionality

### Phase 2: AST Analysis Passes Migration (Week 2-3)

**Goal**: Migrate existing AST passes from solidity module

**Tasks**:
1. Create `smarthunt/src/analysis/passes/ast/` directory
2. Migrate AST passes:
   - [symbol_table.rs](smarthunt/src/analysis/passes/ast/symbol_table.rs)
   - [type_index.rs](smarthunt/src/analysis/passes/ast/type_index.rs)
   - [call_graph.rs](smarthunt/src/analysis/passes/ast/call_graph.rs)
   - [inheritance.rs](smarthunt/src/analysis/passes/ast/inheritance.rs)
   - [modifier.rs](smarthunt/src/analysis/passes/ast/modifier.rs)
3. Register passes in PassManager
4. Update detectors to declare dependencies on these passes
5. Test artifact storage and retrieval

**Success Criteria**:
- ✅ Symbol table, call graph, and other AST passes available
- ✅ Detectors can access pass artifacts via AnalysisContext
- ✅ Pass dependencies correctly resolved

### Phase 3: Control Flow Analysis (Week 3-4)

**Goal**: Implement IR-based control flow graph construction

**Tasks**:
1. Create `smarthunt/src/analysis/passes/ir/` directory
2. Implement CFG construction:
   - [cfg.rs](smarthunt/src/analysis/passes/ir/cfg.rs) - Basic block extraction and edge construction
   - [dominator.rs](smarthunt/src/analysis/passes/ir/dominator.rs) - Dominator tree (for SSA)
   - [loops.rs](smarthunt/src/analysis/passes/ir/loops.rs) - Natural loop detection
3. Add CFG visualization/debugging utilities
4. Write unit tests for CFG construction
5. Integrate with PassManager

**Success Criteria**:
- ✅ CFG correctly constructed for all functions
- ✅ Dominators computed accurately
- ✅ Loop headers and backedges identified

### Phase 4: Data Flow Framework (Week 4-5)

**Goal**: Implement generic data flow analysis framework

**Tasks**:
1. Create `smarthunt/src/analysis/passes/dataflow/` directory structure
2. Implement core DFA framework in [framework.rs](smarthunt/src/analysis/passes/dataflow/framework.rs):
   - `DataFlowFact` trait for lattice elements (bottom, top, meet, less_or_equal)
   - `TransferFunction` trait for analysis-specific logic
   - `AnalysisDirection` enum (Forward/Backward)
   - `DataFlowSolver<F, T>` with generic worklist algorithm
   - Fixpoint iteration with convergence detection
3. Implement specific data flow analyses as `AnalysisPass` implementations:
   - [def_use.rs](smarthunt/src/analysis/passes/dataflow/def_use.rs) - Reaching definitions (forward)
     - Definition and Use site tracking
     - DefUseChain artifact with def→use and use→def maps
   - [liveness.rs](smarthunt/src/analysis/passes/dataflow/liveness.rs) - Variable liveness (backward)
     - LiveSet as DataFlowFact
     - Gen-kill equations: live_in = use ∪ (live_out - def)
   - [taint.rs](smarthunt/src/analysis/passes/dataflow/taint.rs) - Taint propagation (forward)
     - TaintSource and TaintSink enums
     - TaintStatus as DataFlowFact
     - TaintMap artifact with source→sink flows
   - [state_mutation.rs](smarthunt/src/analysis/passes/dataflow/state_mutation.rs) - State tracking
     - Intra-procedural state read/write detection
     - Inter-procedural propagation via call graph
     - StateMutationMap artifact
4. Create [mod.rs](smarthunt/src/analysis/passes/dataflow/mod.rs) with public exports
5. Register all data flow passes with PassManager
6. Add unit tests for framework and each analysis
7. Integration testing with benchmark contracts

**Success Criteria**:
- ✅ Generic worklist algorithm correctly computes fixpoints
- ✅ Forward analysis (taint, def-use) propagates facts correctly
- ✅ Backward analysis (liveness) computes live variables accurately
- ✅ Taint analysis identifies flows from sources to sinks
- ✅ Def-use chains correctly link definitions to uses
- ✅ State mutation tracking works inter-procedurally
- ✅ All passes integrate cleanly with PassManager dependency system

### Phase 5: SSA Construction (Week 5-6)

**Goal**: Implement SSA form for precise analysis

**Tasks**:
1. Implement SSA construction algorithm:
   - [ssa.rs](smarthunt/src/analysis/passes/ir/ssa.rs)
   - Dominance frontier computation
   - Phi function insertion
   - Variable renaming
2. Update data flow analyses to work on SSA
3. Add SSA-based optimizations (constant propagation, dead code elimination)
4. Testing and validation

**Success Criteria**:
- ✅ Correct SSA form generated
- ✅ Phi functions inserted at merge points
- ✅ Data flow analyses more precise on SSA

### Phase 6: Hybrid Bug Detectors (Week 6-8)

**Goal**: Implement bug detectors using both AST patterns and data flow

**Tasks**:
1. Create `smarthunt/src/detection/detectors/hybrid/` directory
2. Implement sophisticated detectors:
   - [reentrancy.rs](smarthunt/src/detection/detectors/hybrid/reentrancy.rs)
     - Pattern: External call followed by state write
     - Uses: CallGraph + CFG + TaintAnalysis + StateMutation
   - [access_control.rs](smarthunt/src/detection/detectors/hybrid/access_control.rs)
     - Pattern: State-changing functions without modifiers
     - Uses: SymbolTable + ModifierAnalysis + StateMutation
   - [state_consistency.rs](smarthunt/src/detection/detectors/hybrid/state_consistency.rs)
     - Pattern: Inconsistent state updates
     - Uses: DefUseChain + StateMutation
3. Implement IR-only detectors:
   - [dead_code.rs](smarthunt/src/detection/detectors/ir/dead_code.rs) - Uses LivenessAnalysis
   - [uninitialized_storage.rs](smarthunt/src/detection/detectors/ir/uninitialized_storage.rs) - Uses DefUseChain
4. Complete AST detector stubs
5. Testing with real-world vulnerable contracts

**Success Criteria**:
- ✅ Reentrancy detector catches known CVEs
- ✅ Access control detector identifies unprotected functions
- ✅ False positive rate acceptable (<20%)
- ✅ All 16+ detectors implemented

### Phase 7: Optimization & Performance (Week 8-9)

**Goal**: Optimize analysis and detection performance

**Tasks**:
1. Enable parallel pass execution
2. Implement incremental analysis (cache artifacts)
3. Profile and optimize hot paths
4. Add analysis statistics and reporting
5. Benchmark against test suite
6. Memory usage optimization

**Success Criteria**:
- ✅ Analysis completes in <5s for medium contracts
- ✅ Memory usage under control (<2GB for large contracts)
- ✅ Parallel execution scales with CPU cores

### Phase 8: Documentation & Testing (Week 9-10)

**Goal**: Comprehensive documentation and test coverage

**Tasks**:
1. Write architecture documentation
2. Add inline documentation for all public APIs
3. Create tutorial for adding new detectors
4. Expand test suite:
   - Unit tests for each analysis pass
   - Integration tests for detector combinations
   - Regression tests for known vulnerabilities
5. Add benchmarking suite
6. Update CLI documentation

**Success Criteria**:
- ✅ >80% test coverage
- ✅ All public APIs documented
- ✅ Tutorial covers detector development

## 5. Data Flow Analysis Usage Patterns

### 5.1 Pattern 1: Taint Tracking for Security

**Example**: Detect tainted data reaching sensitive sinks

```rust
impl BugDetectionPass for UnsafeDelegatecallDetector {
    fn dependencies(&self) -> Vec<PassId> {
        vec![PassId::TaintAnalysis, PassId::CallGraph]
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let taint_map = context.get_artifact::<TaintMap>("taint_map")?;
        let mut bugs = Vec::new();

        for flow in &taint_map.flows {
            if let TaintSink::Delegatecall(loc) = flow.sink {
                if flow.source.is_user_controlled() {
                    bugs.push(create_bug_with_details(
                        self,
                        "Delegatecall with user-controlled target",
                        Some(&format!("Tainted data flows from {:?} to delegatecall", flow.source)),
                        loc,
                    ));
                }
            }
        }

        Ok(bugs)
    }
}
```

### 5.2 Pattern 2: State Mutation + Control Flow

**Example**: Check-Effects-Interactions pattern

```rust
impl BugDetectionPass for CeiViolationDetector {
    fn dependencies(&self) -> Vec<PassId> {
        vec![
            PassId::IrCfg,
            PassId::StateMutation,
            PassId::CallGraph,
        ]
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let state_mutations = context.get_artifact::<StateMutationMap>("state_mutations")?;
        let mut bugs = Vec::new();

        for function in context.functions() {
            let cfg = context.get_artifact::<ControlFlowGraph>(&format!("cfg_{}", function.id))?;

            // Walk CFG to find: external_call followed by state_write
            for block in cfg.blocks.values() {
                let mut seen_external_call = false;
                let mut external_call_loc = None;

                for stmt in &block.stmts {
                    if self.is_external_call(stmt) {
                        seen_external_call = true;
                        external_call_loc = Some(stmt.loc);
                    }

                    if seen_external_call && self.is_state_write(stmt, state_mutations) {
                        bugs.push(create_bug_with_details(
                            self,
                            "State change after external call",
                            Some("Violates Checks-Effects-Interactions pattern"),
                            stmt.loc,
                        ));
                    }
                }
            }
        }

        Ok(bugs)
    }
}
```

### 5.3 Pattern 3: Def-Use + Liveness

**Example**: Uninitialized storage pointer

```rust
impl BugDetectionPass for UninitializedStorageDetector {
    fn dependencies(&self) -> Vec<PassId> {
        vec![
            PassId::DefUseChain,
            PassId::TypeIndex,
            PassId::SymbolTable,
        ]
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let def_use = context.get_artifact::<DefUseChain>("def_use_chain")?;
        let type_index = context.type_index()?;
        let mut bugs = Vec::new();

        for (use_site, defs) in &def_use.use_defs {
            // Check if variable is storage pointer
            let var_type = type_index.get_type(use_site.var)?;
            if !var_type.is_storage_pointer() {
                continue;
            }

            // Check if any reaching definition initializes it
            if defs.is_empty() || defs.iter().all(|d| !self.is_initialized(d)) {
                bugs.push(create_bug_with_details(
                    self,
                    "Uninitialized storage pointer",
                    Some("Storage pointer used without initialization"),
                    use_site.loc,
                ));
            }
        }

        Ok(bugs)
    }
}
```

### 5.4 Pattern 4: Inter-procedural Analysis

**Example**: Reentrancy with call graph

```rust
impl BugDetectionPass for ReentrancyDetector {
    fn dependencies(&self) -> Vec<PassId> {
        vec![
            PassId::CallGraph,
            PassId::StateMutation,
            PassId::IrCfg,
        ]
    }

    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let call_graph = context.call_graph()?;
        let state_mutations = context.get_artifact::<StateMutationMap>("state_mutations")?;
        let mut bugs = Vec::new();

        for function in context.functions() {
            // Skip if function doesn't write state
            if state_mutations.get(function.id).writes.is_empty() {
                continue;
            }

            let cfg = context.get_artifact::<ControlFlowGraph>(&format!("cfg_{}", function.id))?;

            for block in cfg.blocks.values() {
                // Check for pattern:
                //   1. State read (balance check)
                //   2. External call
                //   3. State write (balance update)

                if let Some(vulnerability) = self.check_reentrancy_vulnerability(
                    block,
                    call_graph,
                    state_mutations,
                ) {
                    bugs.push(vulnerability);
                }
            }
        }

        Ok(bugs)
    }
}
```

## 6. Testing Strategy

### 6.1 Unit Tests

**Per-Pass Testing**:
- Each analysis pass has dedicated unit tests
- Test with minimal input programs
- Verify artifact correctness

**Example**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_cfg_construction() {
        let source = "contract C { function f() { if (x) { y = 1; } else { y = 2; } } }";
        let ast = parse(source).unwrap();
        let cfg = build_cfg(&ast);

        assert_eq!(cfg.blocks.len(), 4); // entry, if, else, merge
        assert_eq!(cfg.entry.successors().len(), 2);
    }
}
```

### 6.2 Integration Tests

**Detector Testing**:
- Test with real vulnerable contracts
- Verify true positives (known vulnerabilities)
- Verify true negatives (safe patterns)
- Measure false positive rate

**Test Suite**:
```
tests/
├── vulnerable/
│   ├── reentrancy_dao.sol          # The DAO hack
│   ├── reentrancy_simple.sol       # Simple reentrancy
│   ├── unprotected_selfdestruct.sol
│   └── ...
├── safe/
│   ├── reentrancy_guard.sol        # Properly protected
│   ├── checks_effects.sol          # CEI pattern
│   └── ...
└── benchmarks/
    ├── uniswap_v2.sol
    ├── compound.sol
    └── ...
```

### 6.3 Regression Tests

- Maintain database of known CVEs
- Ensure detectors catch historical vulnerabilities
- Track false positive/negative rates over time

### 6.4 Performance Benchmarks

- Track analysis time per pass
- Memory usage profiling
- Scalability tests (large contracts)

## 7. Success Metrics

### 7.1 Functionality Metrics

- **Pass Coverage**: All planned analysis passes implemented
- **Detector Coverage**: 20+ bug detectors implemented
- **CWE Coverage**: Cover top 10 smart contract CWEs
- **Test Coverage**: >80% code coverage

### 7.2 Performance Metrics

- **Analysis Time**: <5s for 1000 LOC contract
- **Memory Usage**: <2GB for large contracts
- **Parallel Speedup**: 2-3x with 4 cores

### 7.3 Quality Metrics

- **True Positive Rate**: >80% (detects real bugs)
- **False Positive Rate**: <20% (minimizes noise)
- **Precision**: >75% (bugs reported are real)
- **Recall**: >70% (finds most bugs)

## 8. Risk Mitigation

### 8.1 Technical Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| IR generation bugs | High | Extensive testing, validation against AST |
| Data flow imprecision | Medium | Conservative analysis, tunable precision |
| Performance degradation | Medium | Profiling, optimization, parallel execution |
| False positives | High | Confidence levels, user configuration |
| Breaking existing detectors | High | Comprehensive regression tests |

### 8.2 Schedule Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Underestimated complexity | High | Phased approach, MVP for each phase |
| Dependency issues | Medium | Early integration testing |
| Resource constraints | Medium | Prioritize core features |

## 9. Dependencies

### 9.1 Internal Dependencies

- `solidity` crate: AST, IR, parser
- `bugs` crate: Bug data structures
- `smarthunt`: Existing detection framework

### 9.2 External Dependencies

- `petgraph`: Graph algorithms (call graph, CFG)
- `rayon`: Parallel execution
- `dashmap`: Concurrent hash maps
- `tracing`: Logging and diagnostics

## 10. Future Enhancements (Post-MVP)

### 10.1 Advanced Analyses

- **Symbolic Execution**: Path exploration for edge cases
- **Abstract Interpretation**: More precise value tracking
- **Points-to Analysis**: Heap analysis for complex data structures
- **Alias Analysis**: Track aliasing relationships
- **Range Analysis**: Integer overflow detection

### 10.2 Machine Learning Integration

- **Pattern Learning**: Learn bug patterns from labeled data
- **False Positive Filtering**: ML-based ranking
- **Anomaly Detection**: Unusual code patterns

### 10.3 Cross-Contract Analysis

- **Whole-Program Analysis**: Analyze contract systems
- **Compositional Analysis**: Modular verification
- **Property Checking**: Invariant verification

## 11. Conclusion

This plan outlines a comprehensive approach to:

1. **Migrate** the analysis module infrastructure into SmartHunt
2. **Implement** a robust data flow analysis framework
3. **Enable** sophisticated bug detection combining AST patterns and data flow
4. **Deliver** a production-ready static analyzer for Solidity

The phased approach ensures:
- ✅ Incremental progress with testable milestones
- ✅ No disruption to existing functionality
- ✅ Clear success criteria for each phase
- ✅ Extensibility for future enhancements

**Estimated Timeline**: 10 weeks for full implementation
**Team Size**: 2-3 developers
**Priority**: High (enables sophisticated bug detection)

---

**Next Steps**:
1. Review and approve this plan
2. Set up project tracking (issues, milestones)
3. Begin Phase 1: Infrastructure migration
4. Regular progress reviews (weekly)
