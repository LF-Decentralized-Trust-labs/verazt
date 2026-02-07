# Plan: Restructure Data Flow Analysis and AST Pattern Matching Frameworks

## Executive Summary

This plan proposes a restructured analysis architecture for SmartHunt with two distinct frameworks:

1. **Data Flow Analysis (DFA) Framework** - For analyzing Solidity IR with control flow sensitivity
2. **AST Pattern Matching Framework** - For efficient pattern-based detection on Solidity AST

Both frameworks are designed to be efficient, extensible, and practical for implementing bug detection passes.

---

## Part 1: Data Flow Analysis Framework for Solidity IR (`irdfa`)

### 1.1 Current State Analysis

**Existing Infrastructure:**
- Basic `DataFlowFact` trait with lattice operations (`bottom`, `top`, `meet`, `less_or_equal`)
- Generic `DataFlowSolver` with worklist algorithm
- `TransferFunction` trait for statement semantics
- CFG with `BasicBlock`, `Terminator`, and `ControlFlowGraph`

**Current Limitations:**
1. No SSA form - makes def-use analysis expensive
2. Limited inter-procedural analysis support
3. No distinction between intra-block and inter-block analysis
4. Missing efficient fact propagation for sparse analyses
5. No support for context-sensitive analysis
6. CFG doesn't track variable definitions/uses per block

### 1.2 Proposed DFA Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         DFA Framework                                │
├─────────────────────────────────────────────────────────────────────┤
│  Layer 4: Bug Detection Passes                                       │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │ Reentrancy  │ │ Uninitialized│ │ Integer     │ │ Taint Flow  │   │
│  │ Detector    │ │ Storage     │ │ Overflow    │ │ Detector    │   │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │
├─────────────────────────────────────────────────────────────────────┤
│  Layer 3: Composed Analyses                                          │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │ State       │ │ Taint       │ │ Alias       │ │ Value Range │   │
│  │ Dependency  │ │ Analysis    │ │ Analysis    │ │ Analysis    │   │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │
├─────────────────────────────────────────────────────────────────────┤
│  Layer 2: Core Data Flow Analyses                                    │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │ Reaching    │ │ Live        │ │ Available   │ │ Def-Use     │   │
│  │ Definitions │ │ Variables   │ │ Expressions │ │ Chains      │   │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │
├─────────────────────────────────────────────────────────────────────┤
│  Layer 1: Foundation (CFG + Framework)                               │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ Enhanced CFG │ Lattice Framework │ Worklist Solver │ SSA (opt) ││
│  └─────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
```

### 1.3 Layer 1: Foundation Components

#### 1.3.1 Enhanced CFG Representation

**File**: `smarthunt/src/irdfa/cfg.rs`

```rust
/// Enhanced basic block with def/use information
pub struct BasicBlock {
    pub id: BasicBlockId,
    pub stmts: Vec<Stmt>,
    pub terminator: Terminator,
    pub predecessors: Vec<BasicBlockId>,
    pub successors: Vec<BasicBlockId>,

    // New: Precomputed def/use sets for efficiency
    pub defs: HashSet<VarId>,      // Variables defined in this block
    pub uses: HashSet<VarId>,      // Variables used in this block
    pub upward_exposed: HashSet<VarId>, // Used before defined in block
}

/// Variable identifier for tracking
#[derive(Clone, Hash, Eq, PartialEq)]
pub struct VarId {
    pub name: String,
    pub scope: VarScope,
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum VarScope {
    Local,                          // Function local variable
    State { contract: String },     // Contract state variable
    Memory,                         // Memory location
    Storage,                        // Storage slot
}

/// Enhanced CFG with additional metadata
pub struct ControlFlowGraph {
    pub function_id: FunctionId,
    pub blocks: HashMap<BasicBlockId, BasicBlock>,
    pub entry: BasicBlockId,
    pub exit_blocks: Vec<BasicBlockId>,  // Multiple exits possible

    // Precomputed for efficiency
    pub reverse_postorder: Vec<BasicBlockId>,
    pub dominators: Option<DominatorTree>,
    pub loop_info: Option<LoopInfo>,
}
```

**Key Improvements:**
- Precomputed def/use sets per block (avoids repeated traversal)
- Reverse postorder for efficient iteration
- Support for multiple exit blocks
- Variable scoping (local vs. state vs. memory vs. storage)

#### 1.3.2 Improved Lattice Framework

**File**: `smarthunt/src/irdfa/lattice.rs`

```rust
/// A lattice for data flow analysis with clearer semantics
pub trait Lattice: Clone + Eq + Debug + Send + Sync {
    /// Bottom element (⊥) - no information / initial state
    fn bottom() -> Self;

    /// Top element (⊤) - all possible information
    fn top() -> Self;

    /// Join operation (⊔) - combines information from multiple paths
    /// For forward analysis: information at merge points
    /// For backward analysis: information from multiple successors
    fn join(&self, other: &Self) -> Self;

    /// Meet operation (⊓) - intersection of information
    fn meet(&self, other: &Self) -> Self;

    /// Partial order check: self ⊑ other
    fn less_or_equal(&self, other: &Self) -> bool;

    /// Check if this is the bottom element
    fn is_bottom(&self) -> bool {
        self == &Self::bottom()
    }

    /// Check if this is the top element
    fn is_top(&self) -> bool {
        self == &Self::top()
    }
}

/// Common lattice implementations
pub mod lattices {
    /// PowerSet lattice for set-based analyses
    #[derive(Clone, Eq, PartialEq, Debug)]
    pub struct PowerSetLattice<T: Clone + Eq + Hash> {
        pub elements: HashSet<T>,
    }

    impl<T: Clone + Eq + Hash + Send + Sync + Debug> Lattice for PowerSetLattice<T> {
        fn bottom() -> Self { Self { elements: HashSet::new() } }
        fn top() -> Self { panic!("PowerSet top requires universe") }
        fn join(&self, other: &Self) -> Self {
            Self { elements: self.elements.union(&other.elements).cloned().collect() }
        }
        fn meet(&self, other: &Self) -> Self {
            Self { elements: self.elements.intersection(&other.elements).cloned().collect() }
        }
        fn less_or_equal(&self, other: &Self) -> bool {
            self.elements.is_subset(&other.elements)
        }
    }

    /// Map lattice for variable-to-value analyses
    #[derive(Clone, Eq, PartialEq, Debug)]
    pub struct MapLattice<K: Clone + Eq + Hash, V: Lattice> {
        pub map: HashMap<K, V>,
    }

    /// Product lattice for combining multiple analyses
    #[derive(Clone, Eq, PartialEq, Debug)]
    pub struct ProductLattice<A: Lattice, B: Lattice> {
        pub first: A,
        pub second: B,
    }

    /// Flat lattice for constant propagation
    #[derive(Clone, Eq, PartialEq, Debug)]
    pub enum FlatLattice<T: Clone + Eq> {
        Bottom,
        Value(T),
        Top,
    }
}
```

#### 1.3.3 Generic Data Flow Solver

**File**: `smarthunt/src/irdfa/solver.rs`

```rust
/// Analysis direction
#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Forward,   // Information flows with control flow
    Backward,  // Information flows against control flow
}

/// Transfer function for statements
pub trait Transfer<L: Lattice>: Send + Sync {
    /// Apply transfer function to a single statement
    fn transfer_stmt(&self, stmt: &Stmt, fact: &L) -> L;

    /// Apply transfer function to a block (default: sequential)
    fn transfer_block(&self, block: &BasicBlock, fact: L, direction: Direction) -> L {
        let stmts = match direction {
            Direction::Forward => block.stmts.iter().collect::<Vec<_>>(),
            Direction::Backward => block.stmts.iter().rev().collect::<Vec<_>>(),
        };
        stmts.into_iter().fold(fact, |acc, stmt| self.transfer_stmt(stmt, &acc))
    }

    /// Handle terminator (optional, for analyses that care about branches)
    fn transfer_terminator(&self, terminator: &Terminator, fact: &L) -> L {
        fact.clone()
    }
}

/// Result of data flow analysis
pub struct DataFlowResult<L: Lattice> {
    /// Fact at entry of each block
    pub block_entry: HashMap<BasicBlockId, L>,
    /// Fact at exit of each block
    pub block_exit: HashMap<BasicBlockId, L>,
    /// Number of iterations until fixpoint
    pub iterations: usize,
    /// Whether fixpoint was reached
    pub converged: bool,
}

impl<L: Lattice> DataFlowResult<L> {
    /// Get fact before a specific statement in a block
    pub fn fact_before_stmt(&self, block_id: BasicBlockId, stmt_index: usize,
                           transfer: &impl Transfer<L>, block: &BasicBlock,
                           direction: Direction) -> L {
        // Compute intermediate facts
        todo!()
    }
}

/// Generic worklist-based solver
pub struct DataFlowSolver<L: Lattice, T: Transfer<L>> {
    direction: Direction,
    transfer: T,
    max_iterations: usize,
    _phantom: PhantomData<L>,
}

impl<L: Lattice, T: Transfer<L>> DataFlowSolver<L, T> {
    pub fn new(direction: Direction, transfer: T) -> Self {
        Self {
            direction,
            transfer,
            max_iterations: 1000,
            _phantom: PhantomData,
        }
    }

    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Solve data flow equations on CFG
    pub fn solve(&self, cfg: &ControlFlowGraph, initial: L) -> DataFlowResult<L> {
        match self.direction {
            Direction::Forward => self.solve_forward(cfg, initial),
            Direction::Backward => self.solve_backward(cfg, initial),
        }
    }

    fn solve_forward(&self, cfg: &ControlFlowGraph, initial: L) -> DataFlowResult<L> {
        let mut block_entry: HashMap<BasicBlockId, L> = HashMap::new();
        let mut block_exit: HashMap<BasicBlockId, L> = HashMap::new();

        // Initialize
        for &id in cfg.blocks.keys() {
            block_entry.insert(id, L::bottom());
            block_exit.insert(id, L::bottom());
        }
        block_entry.insert(cfg.entry, initial);

        // Worklist algorithm using reverse postorder
        let mut worklist: VecDeque<BasicBlockId> =
            cfg.reverse_postorder.iter().copied().collect();
        let mut iterations = 0;

        while let Some(block_id) = worklist.pop_front() {
            iterations += 1;
            if iterations > self.max_iterations {
                return DataFlowResult {
                    block_entry,
                    block_exit,
                    iterations,
                    converged: false,
                };
            }

            let block = cfg.blocks.get(&block_id).unwrap();

            // Compute entry fact by joining predecessors' exit facts
            let entry = if block_id == cfg.entry {
                block_entry.get(&block_id).unwrap().clone()
            } else {
                block.predecessors.iter()
                    .filter_map(|pred| block_exit.get(pred))
                    .fold(L::bottom(), |acc, fact| acc.join(fact))
            };

            // Apply transfer function
            let exit = self.transfer.transfer_block(block, entry.clone(), Direction::Forward);

            // Check for changes
            let old_exit = block_exit.get(&block_id).unwrap();
            if &exit != old_exit {
                block_entry.insert(block_id, entry);
                block_exit.insert(block_id, exit);

                // Add successors to worklist
                for &succ in &block.successors {
                    if !worklist.contains(&succ) {
                        worklist.push_back(succ);
                    }
                }
            }
        }

        DataFlowResult {
            block_entry,
            block_exit,
            iterations,
            converged: true,
        }
    }

    fn solve_backward(&self, cfg: &ControlFlowGraph, initial: L) -> DataFlowResult<L> {
        // Similar but iterate in reverse order and use successors for join
        todo!()
    }
}
```

### 1.4 Layer 2: Core Data Flow Analyses

#### 1.4.1 Reaching Definitions Analysis

**File**: `smarthunt/src/irdfa/analyses/reaching_defs.rs`

```rust
/// A definition point
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Definition {
    pub var: VarId,
    pub block: BasicBlockId,
    pub stmt_index: usize,
    pub loc: Option<Loc>,
}

/// Reaching definitions lattice (set of definitions)
pub type ReachingDefsLattice = PowerSetLattice<Definition>;

/// Transfer function for reaching definitions
pub struct ReachingDefsTransfer;

impl Transfer<ReachingDefsLattice> for ReachingDefsTransfer {
    fn transfer_stmt(&self, stmt: &Stmt, fact: &ReachingDefsLattice) -> ReachingDefsLattice {
        let mut result = fact.clone();

        // Kill: remove old definitions of assigned variables
        // Gen: add new definitions
        if let Some(assigned_var) = get_assigned_var(stmt) {
            result.elements.retain(|def| def.var != assigned_var);
            result.elements.insert(Definition {
                var: assigned_var,
                block: /* current block */,
                stmt_index: /* current index */,
                loc: stmt.loc(),
            });
        }

        result
    }
}

/// Reaching definitions analysis pass
pub struct ReachingDefsPass;

impl AnalysisPass for ReachingDefsPass {
    fn run(&self, context: &mut AnalysisContext) -> Result<(), PassError> {
        let cfgs = context.get_artifact::<CfgMap>("cfg_map")?;
        let mut results = HashMap::new();

        for (func_id, cfg) in cfgs.iter() {
            let solver = DataFlowSolver::new(Direction::Forward, ReachingDefsTransfer);
            let result = solver.solve(cfg, ReachingDefsLattice::bottom());
            results.insert(func_id.clone(), result);
        }

        context.set_artifact("reaching_defs", Arc::new(results));
        Ok(())
    }
}
```

#### 1.4.2 Live Variables Analysis

**File**: `smarthunt/src/irdfa/analyses/liveness.rs`

```rust
/// Live variables lattice (set of live variables)
pub type LiveVarsLattice = PowerSetLattice<VarId>;

/// Transfer function for liveness (backward analysis)
pub struct LiveVarsTransfer;

impl Transfer<LiveVarsLattice> for LiveVarsTransfer {
    fn transfer_stmt(&self, stmt: &Stmt, fact: &LiveVarsLattice) -> LiveVarsLattice {
        let mut result = fact.clone();

        // Gen: variables used in statement become live
        for used in get_used_vars(stmt) {
            result.elements.insert(used);
        }

        // Kill: variables defined become not live (before the def)
        if let Some(defined) = get_assigned_var(stmt) {
            result.elements.remove(&defined);
        }

        result
    }
}

/// Live variables analysis pass
pub struct LivenessPass;

impl AnalysisPass for LivenessPass {
    fn run(&self, context: &mut AnalysisContext) -> Result<(), PassError> {
        let cfgs = context.get_artifact::<CfgMap>("cfg_map")?;
        let mut results = HashMap::new();

        for (func_id, cfg) in cfgs.iter() {
            let solver = DataFlowSolver::new(Direction::Backward, LiveVarsTransfer);
            let result = solver.solve(cfg, LiveVarsLattice::bottom());
            results.insert(func_id.clone(), result);
        }

        context.set_artifact("liveness", Arc::new(results));
        Ok(())
    }
}
```

#### 1.4.3 Def-Use Chains

**File**: `smarthunt/src/irdfa/analyses/def_use.rs`

```rust
/// A use point
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Use {
    pub var: VarId,
    pub block: BasicBlockId,
    pub stmt_index: usize,
    pub loc: Option<Loc>,
}

/// Def-use chain: maps definitions to their uses
pub struct DefUseChains {
    /// Definition -> Uses
    pub def_to_uses: HashMap<Definition, HashSet<Use>>,
    /// Use -> Reaching definitions
    pub use_to_defs: HashMap<Use, HashSet<Definition>>,
}

/// Build def-use chains from reaching definitions
pub struct DefUseChainsPass;

impl AnalysisPass for DefUseChainsPass {
    fn run(&self, context: &mut AnalysisContext) -> Result<(), PassError> {
        let reaching_defs = context.get_artifact::<ReachingDefsResults>("reaching_defs")?;
        let cfgs = context.get_artifact::<CfgMap>("cfg_map")?;

        let mut chains = DefUseChains::new();

        for (func_id, cfg) in cfgs.iter() {
            let defs_result = reaching_defs.get(func_id)?;

            for (block_id, block) in &cfg.blocks {
                let mut current_defs = defs_result.block_entry.get(block_id).cloned()
                    .unwrap_or_default();

                for (idx, stmt) in block.stmts.iter().enumerate() {
                    // For each use in this statement
                    for used_var in get_used_vars(stmt) {
                        let use_point = Use {
                            var: used_var.clone(),
                            block: *block_id,
                            stmt_index: idx,
                            loc: stmt.loc(),
                        };

                        // Find reaching definitions for this variable
                        for def in &current_defs.elements {
                            if def.var == used_var {
                                chains.add_def_use(def.clone(), use_point.clone());
                            }
                        }
                    }

                    // Update reaching defs after this statement
                    current_defs = ReachingDefsTransfer.transfer_stmt(stmt, &current_defs);
                }
            }
        }

        context.set_artifact("def_use_chains", Arc::new(chains));
        Ok(())
    }
}
```

### 1.5 Layer 3: Composed Analyses

#### 1.5.1 Taint Analysis

**File**: `smarthunt/src/irdfa/analyses/taint.rs`

```rust
/// Taint sources specific to Solidity
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum TaintSource {
    MsgSender,           // msg.sender
    MsgValue,            // msg.value
    MsgData,             // msg.data
    TxOrigin,            // tx.origin
    BlockTimestamp,      // block.timestamp
    BlockNumber,         // block.number
    ExternalCallResult,  // Result of external call
    CallDataLoad,        // Direct calldata access
    FunctionParameter,   // Function input parameter
    StorageRead,         // Value read from storage
}

/// Taint sinks (sensitive operations)
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum TaintSink {
    EtherTransfer,       // .transfer(), .send(), .call{value:}()
    Delegatecall,        // .delegatecall()
    Selfdestruct,        // selfdestruct()
    StorageWrite,        // State variable assignment
    ExternalCallAddress, // Address in external call
    ArrayIndex,          // Array/mapping index (for DoS)
    LoopBound,           // Loop iteration bound
}

/// Taint state for a variable
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TaintState {
    Untainted,
    Tainted {
        sources: HashSet<TaintSource>,
        propagation_path: Vec<Loc>,
    },
}

/// Taint lattice for variables
pub type TaintLattice = MapLattice<VarId, TaintState>;

/// Transfer function for taint propagation
pub struct TaintTransfer {
    // Configurable: which sources to track
    tracked_sources: HashSet<TaintSource>,
}

impl Transfer<TaintLattice> for TaintTransfer {
    fn transfer_stmt(&self, stmt: &Stmt, fact: &TaintLattice) -> TaintLattice {
        let mut result = fact.clone();

        match stmt {
            Stmt::VarDecl(decl) => {
                if let Some(value) = &decl.value {
                    let taint = self.compute_taint(value, fact);
                    result.set(decl.var_id(), taint);
                }
            }
            Stmt::Expr(ExprStmt { expr: Expr::Assign(assign), .. }) => {
                let taint = self.compute_taint(&assign.rhs, fact);
                if let Some(var_id) = get_var_id(&assign.lhs) {
                    result.set(var_id, taint);
                }
            }
            // ... other cases
        }

        result
    }
}

/// Taint analysis pass - finds taint flows from sources to sinks
pub struct TaintAnalysisPass;

impl AnalysisPass for TaintAnalysisPass {
    fn run(&self, context: &mut AnalysisContext) -> Result<(), PassError> {
        // 1. Run taint propagation
        // 2. Identify sink locations
        // 3. Report taint flows reaching sinks
        todo!()
    }
}
```

#### 1.5.2 State Mutation Analysis

**File**: `smarthunt/src/irdfa/analyses/state_mutation.rs`

```rust
/// State mutation event
#[derive(Clone, Debug)]
pub struct StateMutation {
    pub var: VarId,
    pub kind: MutationKind,
    pub loc: Loc,
    pub in_external_call_context: bool,
}

#[derive(Clone, Debug)]
pub enum MutationKind {
    Write,
    Increment,
    Decrement,
    MapUpdate,
    ArrayPush,
    ArrayPop,
}

/// State access (read or write)
#[derive(Clone, Debug)]
pub struct StateAccess {
    pub var: VarId,
    pub kind: AccessKind,
    pub loc: Loc,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessKind {
    Read,
    Write,
}

/// State access ordering for CEI analysis
pub struct StateAccessSequence {
    pub function: FunctionId,
    pub accesses: Vec<StateAccess>,
    pub external_calls: Vec<ExternalCallInfo>,
}

/// Analysis for Checks-Effects-Interactions pattern
pub struct StateMutationPass;

impl AnalysisPass for StateMutationPass {
    fn run(&self, context: &mut AnalysisContext) -> Result<(), PassError> {
        // Analyze state access ordering
        // Detect CEI violations: Effect after Interaction
        todo!()
    }
}
```

### 1.6 Layer 4: Bug Detection Using DFA

#### 1.6.1 Reentrancy Detector

```rust
/// Reentrancy detection using state mutation analysis
pub struct ReentrancyDetector;

impl BugDetectionPass for ReentrancyDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let state_access = context.get_artifact::<StateAccessResults>("state_access")?;
        let call_graph = context.get_artifact::<CallGraph>("call_graph")?;

        let mut bugs = Vec::new();

        for (func_id, sequence) in state_access.iter() {
            // Check for state writes after external calls
            for ext_call in &sequence.external_calls {
                let writes_after = sequence.accesses.iter()
                    .filter(|a| a.kind == AccessKind::Write && a.loc > ext_call.loc);

                for write in writes_after {
                    bugs.push(Bug {
                        kind: BugKind::Reentrancy,
                        message: format!(
                            "State variable '{}' written after external call",
                            write.var.name
                        ),
                        locations: vec![ext_call.loc, write.loc],
                        severity: RiskLevel::Critical,
                    });
                }
            }
        }

        Ok(bugs)
    }
}
```

---

## Part 2: AST Pattern Matching Framework (`astmatch`)

### 2.1 Design Goals

1. **Declarative patterns**: Define patterns as composable structures
2. **Efficient matching**: Single-pass traversal with multiple patterns
3. **Capture groups**: Extract matched sub-expressions for analysis
4. **Context awareness**: Access symbol table and type information during matching
5. **Extensibility**: Easy to add new pattern types

### 2.2 Proposed Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    AST Pattern Matching Framework                    │
├─────────────────────────────────────────────────────────────────────┤
│  Layer 3: Bug Detection Patterns                                     │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │
│  │ TxOrigin    │ │ Timestamp   │ │ Deprecated  │ │ Unchecked   │   │
│  │ Pattern     │ │ Pattern     │ │ Patterns    │ │ Call Pattern│   │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │
├─────────────────────────────────────────────────────────────────────┤
│  Layer 2: Composite Patterns                                         │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ And | Or | Not | Sequence | Contains | Where (predicate)        ││
│  └─────────────────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────────────────┤
│  Layer 1: Primitive Patterns                                         │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ Expr patterns │ Stmt patterns │ Type patterns │ Wildcards       ││
│  └─────────────────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────────────────┤
│  Layer 0: Pattern Engine                                             │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ PatternMatcher │ AstVisitor │ MatchContext │ CaptureGroups      ││
│  └─────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
```

### 2.3 Core Pattern Types

**File**: `smarthunt/src/astmatch/mod.rs`

```rust
/// Result of a pattern match
#[derive(Clone, Debug)]
pub struct Match {
    pub loc: Loc,
    pub captures: HashMap<String, CapturedNode>,
    pub context: MatchContext,
}

/// Captured AST node
#[derive(Clone, Debug)]
pub enum CapturedNode {
    Expr(Box<Expr>),
    Stmt(Box<Stmt>),
    Type(Box<TypeName>),
    Ident(Identifier),
    Lit(Lit),
}

/// Context available during matching
pub struct MatchContext {
    pub symbol_table: Arc<SymbolTable>,
    pub type_index: Arc<TypeIndex>,
    pub current_contract: Option<Name>,
    pub current_function: Option<FunctionId>,
    pub in_modifier: bool,
    pub in_loop: bool,
}

/// Core pattern trait
pub trait Pattern: Send + Sync {
    /// Try to match against an expression
    fn match_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match>;

    /// Try to match against a statement
    fn match_stmt(&self, stmt: &Stmt, ctx: &MatchContext) -> Option<Match>;

    /// Name for debugging
    fn name(&self) -> &str;

    /// Description for documentation
    fn description(&self) -> &str;
}
```

### 2.4 Primitive Patterns

**File**: `smarthunt/src/astmatch/primitives.rs`

```rust
/// Match a specific identifier
pub struct IdentPattern {
    pub name: String,
    pub capture_as: Option<String>,
}

impl Pattern for IdentPattern {
    fn match_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match> {
        if let Expr::Ident(id) = expr {
            if id.name == self.name {
                let mut captures = HashMap::new();
                if let Some(key) = &self.capture_as {
                    captures.insert(key.clone(), CapturedNode::Ident(id.clone()));
                }
                return Some(Match { loc: id.loc?, captures, context: ctx.clone() });
            }
        }
        None
    }
    // ...
}

/// Match a member access: object.member
pub struct MemberAccessPattern {
    pub object: Box<dyn Pattern>,
    pub member: String,
    pub capture_object_as: Option<String>,
}

impl Pattern for MemberAccessPattern {
    fn match_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match> {
        if let Expr::Member(m) = expr {
            if m.member.name == self.member {
                if let Some(obj_match) = self.object.match_expr(&m.object, ctx) {
                    let mut captures = obj_match.captures;
                    if let Some(key) = &self.capture_object_as {
                        captures.insert(key.clone(), CapturedNode::Expr(m.object.clone()));
                    }
                    return Some(Match { loc: m.loc?, captures, context: ctx.clone() });
                }
            }
        }
        None
    }
    // ...
}

/// Match a function call with specific callee
pub struct CallPattern {
    pub callee: Box<dyn Pattern>,
    pub args: Option<ArgsPattern>,
    pub capture_as: Option<String>,
}

/// Match any expression (wildcard)
pub struct AnyExpr {
    pub capture_as: Option<String>,
}

impl Pattern for AnyExpr {
    fn match_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match> {
        let mut captures = HashMap::new();
        if let Some(key) = &self.capture_as {
            captures.insert(key.clone(), CapturedNode::Expr(Box::new(expr.clone())));
        }
        Some(Match { loc: expr.loc()?, captures, context: ctx.clone() })
    }
    // ...
}

/// Match expression with specific type
pub struct TypedExprPattern {
    pub inner: Box<dyn Pattern>,
    pub expected_type: TypePattern,
}

/// Type patterns
pub enum TypePattern {
    Exact(String),           // Exact type name
    Address,                 // address or address payable
    Integer,                 // any int/uint
    Bytes,                   // bytes or bytesN
    String,                  // string
    Array(Box<TypePattern>), // T[]
    Mapping { key: Box<TypePattern>, value: Box<TypePattern> },
    Any,
}
```

### 2.5 Composite Patterns

**File**: `smarthunt/src/astmatch/composite.rs`

```rust
/// Match if all patterns match
pub struct AndPattern {
    pub patterns: Vec<Box<dyn Pattern>>,
}

impl Pattern for AndPattern {
    fn match_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match> {
        let mut combined_captures = HashMap::new();

        for pattern in &self.patterns {
            let m = pattern.match_expr(expr, ctx)?;
            combined_captures.extend(m.captures);
        }

        Some(Match {
            loc: expr.loc()?,
            captures: combined_captures,
            context: ctx.clone()
        })
    }
    // ...
}

/// Match if any pattern matches
pub struct OrPattern {
    pub patterns: Vec<Box<dyn Pattern>>,
}

impl Pattern for OrPattern {
    fn match_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match> {
        for pattern in &self.patterns {
            if let Some(m) = pattern.match_expr(expr, ctx) {
                return Some(m);
            }
        }
        None
    }
    // ...
}

/// Match if pattern does NOT match
pub struct NotPattern {
    pub inner: Box<dyn Pattern>,
}

/// Match with additional predicate
pub struct WherePattern {
    pub inner: Box<dyn Pattern>,
    pub predicate: Box<dyn Fn(&Match) -> bool + Send + Sync>,
}

impl Pattern for WherePattern {
    fn match_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match> {
        let m = self.inner.match_expr(expr, ctx)?;
        if (self.predicate)(&m) {
            Some(m)
        } else {
            None
        }
    }
    // ...
}

/// Match if expression contains a sub-pattern
pub struct ContainsPattern {
    pub inner: Box<dyn Pattern>,
}

impl Pattern for ContainsPattern {
    fn match_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match> {
        // First try direct match
        if let Some(m) = self.inner.match_expr(expr, ctx) {
            return Some(m);
        }

        // Recursively check sub-expressions
        match expr {
            Expr::Binary(b) => {
                self.match_expr(&b.left, ctx)
                    .or_else(|| self.match_expr(&b.right, ctx))
            }
            Expr::Unary(u) => self.match_expr(&u.operand, ctx),
            Expr::Call(c) => {
                self.match_expr(&c.callee, ctx)
                    .or_else(|| c.args.iter().find_map(|a| self.match_expr(a, ctx)))
            }
            Expr::Member(m) => self.match_expr(&m.object, ctx),
            // ... other cases
            _ => None,
        }
    }
    // ...
}
```

### 2.6 Pattern Builder DSL

**File**: `smarthunt/src/astmatch/builder.rs`

```rust
/// Builder for creating patterns fluently
pub struct PatternBuilder;

impl PatternBuilder {
    /// Match identifier
    pub fn ident(name: &str) -> IdentPattern {
        IdentPattern { name: name.to_string(), capture_as: None }
    }

    /// Match member access: obj.member
    pub fn member(object: impl Pattern + 'static, member: &str) -> MemberAccessPattern {
        MemberAccessPattern {
            object: Box::new(object),
            member: member.to_string(),
            capture_object_as: None,
        }
    }

    /// Match tx.origin
    pub fn tx_origin() -> MemberAccessPattern {
        Self::member(Self::ident("tx"), "origin")
    }

    /// Match msg.sender
    pub fn msg_sender() -> MemberAccessPattern {
        Self::member(Self::ident("msg"), "sender")
    }

    /// Match block.timestamp
    pub fn block_timestamp() -> MemberAccessPattern {
        Self::member(Self::ident("block"), "timestamp")
    }

    /// Match a function call
    pub fn call(callee: impl Pattern + 'static) -> CallPatternBuilder {
        CallPatternBuilder { callee: Box::new(callee), args: None }
    }

    /// Match any expression
    pub fn any() -> AnyExpr {
        AnyExpr { capture_as: None }
    }

    /// Match any of the patterns
    pub fn any_of(patterns: Vec<Box<dyn Pattern>>) -> OrPattern {
        OrPattern { patterns }
    }

    /// Match all patterns
    pub fn all_of(patterns: Vec<Box<dyn Pattern>>) -> AndPattern {
        AndPattern { patterns }
    }

    /// Match expression containing pattern
    pub fn contains(inner: impl Pattern + 'static) -> ContainsPattern {
        ContainsPattern { inner: Box::new(inner) }
    }

    /// Binary comparison: left op right
    pub fn binary(left: impl Pattern + 'static, op: BinaryOp, right: impl Pattern + 'static)
        -> BinaryPattern
    {
        BinaryPattern {
            left: Box::new(left),
            op: Some(op),
            right: Box::new(right),
        }
    }

    /// Assignment pattern
    pub fn assign(target: impl Pattern + 'static, value: impl Pattern + 'static)
        -> AssignPattern
    {
        AssignPattern {
            target: Box::new(target),
            value: Box::new(value),
        }
    }
}

// Extension trait for patterns
pub trait PatternExt: Pattern + Sized {
    /// Capture matched node
    fn capture(self, name: &str) -> CapturePattern<Self>;

    /// Add predicate
    fn where_fn<F: Fn(&Match) -> bool + Send + Sync + 'static>(self, pred: F)
        -> WherePattern;

    /// Combine with AND
    fn and(self, other: impl Pattern + 'static) -> AndPattern;

    /// Combine with OR
    fn or(self, other: impl Pattern + 'static) -> OrPattern;
}
```

### 2.7 Pattern Matcher Engine

**File**: `smarthunt/src/astmatch/matcher.rs`

```rust
/// Pattern matcher that runs multiple patterns in one traversal
pub struct PatternMatcher {
    patterns: Vec<(String, Box<dyn Pattern>)>,
}

impl PatternMatcher {
    pub fn new() -> Self {
        Self { patterns: Vec::new() }
    }

    pub fn add_pattern(&mut self, name: &str, pattern: impl Pattern + 'static) {
        self.patterns.push((name.to_string(), Box::new(pattern)));
    }

    /// Run all patterns on source units
    pub fn match_all(&self, units: &[SourceUnit], ctx: &MatchContext)
        -> HashMap<String, Vec<Match>>
    {
        let mut results: HashMap<String, Vec<Match>> = HashMap::new();

        for (name, _) in &self.patterns {
            results.insert(name.clone(), Vec::new());
        }

        let mut visitor = PatternVisitor {
            matcher: self,
            context: ctx.clone(),
            results: &mut results,
        };

        for unit in units {
            visitor.visit_source_unit(unit);
        }

        results
    }
}

struct PatternVisitor<'a> {
    matcher: &'a PatternMatcher,
    context: MatchContext,
    results: &'a mut HashMap<String, Vec<Match>>,
}

impl<'a> PatternVisitor<'a> {
    fn visit_expr(&mut self, expr: &Expr) {
        // Try all patterns on this expression
        for (name, pattern) in &self.matcher.patterns {
            if let Some(m) = pattern.match_expr(expr, &self.context) {
                self.results.get_mut(name).unwrap().push(m);
            }
        }

        // Recurse into sub-expressions
        match expr {
            Expr::Binary(b) => {
                self.visit_expr(&b.left);
                self.visit_expr(&b.right);
            }
            Expr::Unary(u) => self.visit_expr(&u.operand),
            Expr::Call(c) => {
                self.visit_expr(&c.callee);
                for arg in c.args.iter() {
                    self.visit_expr(arg);
                }
            }
            Expr::Member(m) => self.visit_expr(&m.object),
            Expr::Index(i) => {
                self.visit_expr(&i.object);
                self.visit_expr(&i.index);
            }
            Expr::Conditional(c) => {
                self.visit_expr(&c.condition);
                self.visit_expr(&c.true_expr);
                self.visit_expr(&c.false_expr);
            }
            // ... other cases
            _ => {}
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        // Try all patterns on this statement
        for (name, pattern) in &self.matcher.patterns {
            if let Some(m) = pattern.match_stmt(stmt, &self.context) {
                self.results.get_mut(name).unwrap().push(m);
            }
        }

        // Recurse and visit contained expressions
        match stmt {
            Stmt::Expr(e) => self.visit_expr(&e.expr),
            Stmt::If(i) => {
                self.visit_expr(&i.condition);
                self.visit_block(&i.true_branch);
                if let Some(fb) = &i.false_branch {
                    self.visit_block(fb);
                }
            }
            Stmt::Return(r) => {
                if let Some(e) = &r.value {
                    self.visit_expr(e);
                }
            }
            // ... other cases
            _ => {}
        }
    }

    fn visit_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.visit_stmt(stmt);
        }
    }

    fn visit_source_unit(&mut self, unit: &SourceUnit) {
        for elem in &unit.elems {
            match elem {
                SourceUnitElem::Contract(c) => self.visit_contract(c),
                SourceUnitElem::Func(f) => self.visit_function(f, None),
                _ => {}
            }
        }
    }

    fn visit_contract(&mut self, contract: &ContractDef) {
        let prev_contract = self.context.current_contract.clone();
        self.context.current_contract = Some(contract.name.clone());

        for elem in &contract.elems {
            match elem {
                ContractElem::FuncDef(f) => self.visit_function(f, Some(&contract.name)),
                ContractElem::VarDecl(v) => {
                    if let Some(init) = &v.value {
                        self.visit_expr(init);
                    }
                }
                _ => {}
            }
        }

        self.context.current_contract = prev_contract;
    }

    fn visit_function(&mut self, func: &FuncDef, contract: Option<&Name>) {
        let prev_function = self.context.current_function.clone();
        self.context.current_function = Some(FunctionId {
            contract: contract.cloned(),
            name: func.name.clone(),
            kind: func.kind,
        });

        if let Some(body) = &func.body {
            self.visit_block(body);
        }

        self.context.current_function = prev_function;
    }
}
```

### 2.8 Example: Bug Detector Using Patterns

```rust
/// TxOrigin detector using pattern matching
pub struct TxOriginDetector {
    pattern: Box<dyn Pattern>,
}

impl TxOriginDetector {
    pub fn new() -> Self {
        use PatternBuilder as P;

        // Pattern: tx.origin used in comparison (authentication check)
        let pattern = P::any_of(vec![
            // tx.origin == address
            Box::new(P::binary(P::tx_origin(), BinaryOp::Eq, P::any())),
            // address == tx.origin
            Box::new(P::binary(P::any(), BinaryOp::Eq, P::tx_origin())),
            // require(tx.origin == ...) - implicit via contains
            Box::new(P::contains(P::binary(P::tx_origin(), BinaryOp::Eq, P::any()))),
        ]);

        Self { pattern: Box::new(pattern) }
    }
}

impl BugDetectionPass for TxOriginDetector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let symbol_table = context.get_artifact::<SymbolTable>("symbol_table")?;
        let type_index = context.get_artifact::<TypeIndex>("type_index")?;

        let match_ctx = MatchContext {
            symbol_table,
            type_index,
            current_contract: None,
            current_function: None,
            in_modifier: false,
            in_loop: false,
        };

        let mut bugs = Vec::new();

        for unit in context.source_units() {
            // Use pattern visitor
            let visitor = PatternVisitor::new(&self.pattern, &match_ctx);
            let matches = visitor.visit(unit);

            for m in matches {
                bugs.push(Bug {
                    kind: BugKind::TxOrigin,
                    message: "Use of tx.origin for authentication".to_string(),
                    loc: m.loc,
                    severity: RiskLevel::Medium,
                    confidence: ConfidenceLevel::High,
                });
            }
        }

        Ok(bugs)
    }
}
```

---

## Part 3: Implementation Plan

### Phase 1: Create Top-Level Modules (Week 1-2)

**Task 1.1: Create `irdfa/` Module**
- Create `smarthunt/src/irdfa/mod.rs` with public API
- Create `smarthunt/src/irdfa/var.rs` with `VarId`, `VarScope`
- Create `smarthunt/src/irdfa/cfg.rs` with enhanced CFG
- Create `smarthunt/src/irdfa/lattice.rs` with `Lattice` trait
- Create `smarthunt/src/irdfa/solver.rs` with `DataFlowSolver`
- Add `mod irdfa;` to `lib.rs`

**Task 1.2: Create `astmatch/` Module**
- Create `smarthunt/src/astmatch/mod.rs` with public API
- Create `smarthunt/src/astmatch/core.rs` with `Pattern` trait, `Match`, `CapturedNode`
- Create `smarthunt/src/astmatch/primitives.rs` with basic patterns
- Create `smarthunt/src/astmatch/composite.rs` with combinators
- Create `smarthunt/src/astmatch/builder.rs` with fluent DSL
- Create `smarthunt/src/astmatch/matcher.rs` with engine
- Add `mod astmatch;` to `lib.rs`

**Task 1.3: Implement Lattice Framework**
- Implement `PowerSetLattice`, `MapLattice`, `FlatLattice`, `ProductLattice`
- Add unit tests for lattice operations in `smarthunt/src/irdfa/tests/`

### Phase 2: Core Data Flow Analyses (Week 3-4)

**Task 2.1: Create `irdfa/analyses/` Submodule**
- Create `smarthunt/src/irdfa/analyses/mod.rs`
- Implement `ReachingDefsTransfer` in `reaching_defs.rs`
- Implement `LiveVarsTransfer` in `liveness.rs`
- Implement `DefUseChains` in `def_use.rs`

**Task 2.2: Implement Forward Analyses**
- Complete reaching definitions analysis
- Add tests with sample IR

**Task 2.3: Implement Backward Analyses**
- Complete liveness analysis (backward solver)
- Verify with dead code examples

### Phase 3: Pattern Matching Implementation (Week 5-6)

**Task 3.1: Implement Primitive Patterns**
- `IdentPattern`, `MemberAccessPattern`, `CallPattern`
- `AnyExpr`, `AnyStmt`, `TypedExprPattern`
- Unit tests for each pattern type

**Task 3.2: Implement Composite Patterns**
- `AndPattern`, `OrPattern`, `NotPattern`
- `ContainsPattern`, `WherePattern`, `SequencePattern`
- Unit tests for pattern composition

**Task 3.3: Implement Pattern Matcher Engine**
- `PatternMatcher` for multi-pattern single-pass matching
- `PatternVisitor` for AST traversal
- Integration tests with real Solidity AST

### Phase 4: Migrate Detectors (Week 7-8)

**Task 4.1: Migrate AST Detectors to `astmatch/`**
- Move `TxOriginDetector` to use `PatternBuilder`
- Move `TimestampDependenceDetector` to use `PatternBuilder`
- Move `DeprecatedDetector`, `LowLevelCallDetector`
- Place in `detection/detectors/ast/`

**Task 4.2: Create IR Detectors Using `irdfa/`**
- Create `ReentrancyDetector` using state mutation analysis
- Create `UninitializedStorageDetector` using reaching defs
- Create `DeadCodeDetector` using liveness
- Place in `detection/detectors/ir/`

**Task 4.3: Implement Composed Analyses**
- Implement `TaintAnalysis` in `irdfa/analyses/taint.rs`
- Implement `StateMutationAnalysis` in `irdfa/analyses/state_mutation.rs`
- Create `CeiViolationDetector` combining both

### Phase 5: Integration & Testing (Week 9-10)

**Task 5.1: Update Analysis Infrastructure**
- Update `PassId` enum with new dataflow passes
- Create bridge passes in `analysis/passes/ir/` that use `irdfa/`
- Update `PassManager` dependencies

**Task 5.2: Testing**
- Unit tests in `smarthunt/src/irdfa/tests/`
- Unit tests in `smarthunt/src/astmatch/tests/`
- Integration tests with SWC benchmark contracts
- Performance profiling and optimization

**Task 5.3: Documentation**
- Document `irdfa/` public API with examples
- Document `astmatch/` DSL with examples
- Add examples for creating new detectors

---

## Part 4: File Structure

The DFA and Pattern Matching frameworks are **top-level modules** in `smarthunt/src/`, separate from the analysis module. This provides clear separation of concerns and makes each framework independently reusable.

```
smarthunt/src/
├── lib.rs                        # Crate root (add mod irdfa, mod astmatch)
├── main.rs                       # CLI entry point
├── config.rs                     # CLI configuration
│
├── irdfa/                     # NEW: Top-level DFA Framework
│   ├── mod.rs                    # Public API: Lattice, Solver, CFG
│   │
│   ├── cfg.rs                    # Enhanced CFG representation
│   ├── lattice.rs                # Lattice trait + common implementations
│   ├── solver.rs                 # Generic worklist solver
│   ├── var.rs                    # VarId, VarScope definitions
│   │
│   └── analyses/                 # Built-in data flow analyses
│       ├── mod.rs
│       ├── reaching_defs.rs      # Reaching definitions
│       ├── liveness.rs           # Live variables (backward)
│       ├── def_use.rs            # Def-use chains
│       ├── taint.rs              # Taint analysis
│       └── state_mutation.rs     # State mutation tracking
│
├── astmatch/                     # NEW: Top-level AST Pattern Matching
│   ├── mod.rs                    # Public API: Pattern, Match, PatternBuilder
│   │
│   ├── core.rs                   # Pattern trait, Match, CapturedNode
│   ├── primitives.rs             # IdentPattern, MemberAccessPattern, CallPattern
│   ├── composite.rs              # AndPattern, OrPattern, NotPattern, etc.
│   ├── builder.rs                # Fluent pattern builder DSL
│   ├── matcher.rs                # Multi-pattern matcher engine
│   └── visitor.rs                # AST traversal for pattern matching
│
├── analysis/                     # Analysis pass infrastructure (existing)
│   ├── mod.rs
│   ├── context.rs                # AnalysisContext (existing)
│   ├── pass.rs                   # Pass traits (existing)
│   ├── pass_id.rs                # PassId enum (existing, extend)
│   ├── manager.rs                # PassManager (existing)
│   │
│   └── passes/                   # Analysis passes using DFA/Patterns
│       ├── mod.rs
│       │
│       ├── ast/                  # AST-level passes
│       │   ├── mod.rs
│       │   ├── symbol_table.rs   # (existing)
│       │   ├── call_graph.rs     # (existing)
│       │   ├── type_index.rs     # (existing)
│       │   └── inheritance.rs    # (existing)
│       │
│       └── ir/                   # IR-level passes (use dataflow crate)
│           ├── mod.rs
│           ├── cfg_pass.rs       # CFG construction pass (uses irdfa::CFG)
│           ├── dominator.rs      # Dominator tree
│           └── loop_analysis.rs  # Loop detection
│
├── detection/                    # Bug detection (existing)
│   ├── mod.rs
│   ├── pass.rs                   # BugDetectionPass trait
│   ├── manager.rs                # DetectionManager
│   ├── registry.rs               # Detector registry
│   │
│   └── detectors/
│       ├── mod.rs
│       │
│       ├── ast/                  # AST pattern-based detectors
│       │   ├── mod.rs
│       │   ├── tx_origin.rs      # Uses astmatch::PatternBuilder
│       │   ├── timestamp.rs      # Uses astmatch::PatternBuilder
│       │   ├── deprecated.rs     # Uses astmatch::PatternBuilder
│       │   └── ...
│       │
│       └── ir/                   # DFA-based detectors
│           ├── mod.rs
│           ├── reentrancy.rs     # Uses irdfa::analyses::state_mutation
│           ├── uninitialized.rs  # Uses irdfa::analyses::reaching_defs
│           ├── dead_code.rs      # Uses irdfa::analyses::liveness
│           └── ...
│
└── output/                       # Output formatters (existing)
    ├── mod.rs
    ├── json.rs
    ├── markdown.rs
    └── sarif.rs
```

### Module Dependencies

```
                    ┌─────────────────┐
                    │   solidity      │  (AST + IR definitions)
                    └────────┬────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
         ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│    irdfa     │ │    astmatch     │ │    analysis     │
│  (DFA on IR)    │ │ (AST matching)  │ │  (pass infra)   │
└────────┬────────┘ └────────┬────────┘ └────────┬────────┘
         │                   │                   │
         └───────────────────┼───────────────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │   detection     │  (bug detectors)
                    └─────────────────┘
```

### Key Design Decisions

1. **`irdfa` is independent**: No dependency on `analysis` or `detection`. Can be used standalone.

2. **`astmatch` is independent**: No dependency on `analysis` or `detection`. Can be used standalone.

3. **`analysis` uses both**: Analysis passes can use `irdfa` for IR analysis and `astmatch` for AST queries.

4. **`detection` uses all**: Detectors combine `analysis`, `irdfa`, and `astmatch` as needed.

5. **Clear public APIs**:
   - `irdfa`: exports `Lattice`, `DataFlowSolver`, `ControlFlowGraph`, `VarId`
   - `astmatch`: exports `Pattern`, `Match`, `PatternBuilder`, `PatternMatcher`

---

## Part 5: Migration Strategy

### 5.1 Backward Compatibility

- Keep existing detector implementations working
- Gradually migrate detectors to new frameworks
- Deprecate old patterns with warnings

### 5.2 Testing Strategy

```rust
// Test contracts for each vulnerability type
tests/
├── contracts/
│   ├── reentrancy/
│   │   ├── simple_reentrancy.sol
│   │   ├── cross_function_reentrancy.sol
│   │   └── safe_reentrancy.sol
│   ├── tx_origin/
│   │   ├── tx_origin_auth.sol
│   │   └── tx_origin_safe.sol
│   └── ...
├── irdfa/
│   ├── test_reaching_defs.rs
│   ├── test_liveness.rs
│   └── test_taint.rs
└── astmatch/
    ├── test_primitives.rs
    ├── test_composite.rs
    └── test_matcher.rs
```

### 5.3 Performance Considerations

1. **CFG Construction**: O(n) where n = statements
2. **Data Flow Fixpoint**: O(n * h) where h = lattice height
3. **Pattern Matching**: O(n * p) where p = patterns
4. **Memory**: Store facts per block, not per statement (when possible)

---

## Summary

This plan provides **two independent, top-level frameworks** in `smarthunt/src/`:

### 1. `irdfa/` - Data Flow Analysis Framework

Located at `smarthunt/src/irdfa/`, this is a **standalone module** for IR-based analysis:

- **Enhanced CFG** (`cfg.rs`): Precomputed def/use sets, reverse postorder, multiple exits
- **Generic Lattice** (`lattice.rs`): `PowerSetLattice`, `MapLattice`, `FlatLattice`, `ProductLattice`
- **Worklist Solver** (`solver.rs`): Forward/backward analysis with convergence detection
- **Built-in Analyses** (`analyses/`): Reaching definitions, liveness, def-use chains, taint, state mutation

**Usage**:
```rust
use smarthunt::irdfa::{ControlFlowGraph, DataFlowSolver, Direction};
use smarthunt::irdfa::analyses::ReachingDefsTransfer;
```

### 2. `astmatch/` - AST Pattern Matching Framework

Located at `smarthunt/src/astmatch/`, this is a **standalone module** for AST-based detection:

- **Pattern Trait** (`core.rs`): Declarative pattern definitions with captures
- **Primitives** (`primitives.rs`): `IdentPattern`, `MemberAccessPattern`, `CallPattern`, wildcards
- **Composites** (`composite.rs`): `And`, `Or`, `Not`, `Contains`, `Where` combinators
- **Builder DSL** (`builder.rs`): Fluent API for pattern construction
- **Matcher Engine** (`matcher.rs`): Multi-pattern single-pass matching

**Usage**:
```rust
use smarthunt::astmatch::{PatternBuilder as P, PatternMatcher};

let pattern = P::binary(P::tx_origin(), BinaryOp::Eq, P::any());
```

### 3. Integration with Bug Detection

Detectors in `detection/` use both frameworks:

| Detector Type | Framework | Location |
|---------------|-----------|----------|
| TxOrigin, Timestamp, Deprecated | `astmatch` | `detection/detectors/ast/` |
| Reentrancy, Uninitialized, DeadCode | `irdfa` | `detection/detectors/ir/` |
| CEI Violation, Taint Flow | Both | `detection/detectors/ir/` |

### Design Principles

- **Independence**: `irdfa` and `astmatch` have no dependencies on each other
- **Reusability**: Both frameworks can be used outside of `smarthunt` detection
- **Efficiency**: Single-pass traversal, precomputed metadata, worklist optimization
- **Extensibility**: Trait-based design allows adding new analyses/patterns easily
- **Practicality**: Focus on common Solidity vulnerability patterns
