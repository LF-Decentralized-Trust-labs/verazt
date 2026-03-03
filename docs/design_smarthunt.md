# SmartHunt Design and Architecture

## Overview

SmartHunt is a static analysis tool for detecting vulnerabilities in Solidity smart contracts. It features a sophisticated two-phase pipeline architecture that combines data flow analysis (DFA) on intermediate representation (IR) with pattern matching on abstract syntax trees (AST).

## Architecture Principles

### Core Design Patterns

1. **Two-Level Representation**
   - AST for syntactic/semantic analysis (pattern matching)
   - IR for dataflow analysis (control/data flow)
   - Each representation optimized for different analysis types

2. **Pass-Based Architecture**
   - Extensible: new passes easily added
   - Composable: passes depend on each other
   - Traceable: all passes tracked in context

3. **Lazy Evaluation**
   - IR generated only when needed
   - Passes scheduled dynamically based on detector dependencies
   - Analysis context grows with required passes

4. **Immutable Context for Detectors**
   - Analysis phase populates context (mutable)
   - Detection phase reads context (immutable)
   - Enables safe parallel detector execution

## Pipeline Architecture

### Two-Phase Execution Model

The pipeline follows a two-phase design orchestrated by `PipelineEngine`:

```
Input (Solidity) → Analysis Phase → Detection Phase → Output
```

#### Phase 1: Analysis Phase (Sequential/Parallel by Dependency Level)

**Purpose**: Compute analysis artifacts that detectors depend on

**Key Features**:
- Only required passes are scheduled (lazy evaluation)
- Passes at same dependency level run in parallel via rayon
- Dependency graph automatically resolved by PassScheduler

**Example Passes**:
- Foundation: SymbolTable, TypeIndex
- AST Analysis: CallGraph, InheritanceGraph, ModifierAnalysis
- IR Generation: CFG construction, data flow analysis passes

#### Phase 2: Detection Phase (Fully Parallel)

**Purpose**: Run all enabled detectors to find vulnerabilities

**Key Features**:
- Detectors are read-only, accessing immutable AnalysisContext
- All detectors run fully in parallel (via rayon)
- Results aggregated into single report

### Pipeline Configuration

```rust
PipelineConfig {
    parallel: bool,              // Enable parallel execution
    num_threads: usize,          // Worker threads (0 = auto-detect)
    enabled: Vec<String>,        // Detectors to enable (empty = all)
    disabled: Vec<String>,       // Detectors to disable
}
```

## Analysis Framework

### Pass System

**Pass Trait Hierarchy**:

```
Pass (base trait)
  ├─ id() -> PassId
  ├─ name() -> &str
  ├─ description() -> &str
  ├─ level() -> PassLevel
  ├─ representation() -> PassRepresentation
  └─ dependencies() -> Vec<PassId>

BugDetectionPass extends Pass
  ├─ detect() -> Result<Vec<Bug>>
  ├─ bug_kind() -> BugKind
  ├─ bug_category() -> BugCategory
  ├─ risk_level() -> RiskLevel
  ├─ confidence() -> ConfidenceLevel
  ├─ cwe_ids() -> Vec<usize>
  ├─ swc_ids() -> Vec<usize>
  ├─ recommendation() -> &str
  └─ references() -> Vec<&str>
```

**Pass Levels** (granularity):
- Program (multi-contract)
- Contract (single contract)
- Function (single function)
- Block (basic block)
- Statement (individual statement)
- Expression (individual expression)
- Variable (variable tracking)

**Pass Representations**:
- AST: Abstract Syntax Tree based passes
- IR: Intermediate Representation based passes
- Hybrid: Both AST and IR

### AnalysisContext - Central Artifact Storage

The `AnalysisContext` serves as the central repository for all analysis artifacts:

```rust
AnalysisContext {
    source_units: Vec<SourceUnit>,              // Original AST
    ir_units: Option<Vec<ir::SourceUnit>>,      // Generated IR
    artifacts: HashMap<String, Arc<dyn Any>>,   // Type-erased storage
    completed_passes: HashSet<PassId>,          // Pass tracking
    stats: AnalysisStats,                       // Execution metrics
}
```

**Key Methods**:
- `store_artifact()` / `get_artifact()` - Type-safe artifact storage
- `mark_pass_completed()` / `is_pass_completed()` - Pass tracking
- Convenience accessors: `contracts()`, `functions()`, etc.

### PassManager - Orchestrator

Responsibilities:
- Registers analysis passes
- Computes execution schedule using dependency graph
- Executes passes (sequential or parallel)
- Tracks completion status in AnalysisContext
- Supports lazy IR generation

## Detector System

### Two Detector Categories

#### 1. DFA-Based Detectors (Data Flow Analysis on IR)

These detectors operate on the intermediate representation and use control flow and data flow analysis:

- `CeiViolationDfaDetector` - Checks-Effects-Interactions pattern violations
- `ReentrancyDfaDetector` - Reentrancy vulnerabilities
- `DeadCodeDfaDetector` - Unreachable code detection
- `UncheckedCallDfaDetector` - Unchecked external calls
- `UninitializedDfaDetector` - Uninitialized variables/storage

#### 2. GREP-Based Detectors (Pattern Matching on AST)

These detectors use pattern matching on the abstract syntax tree:

- `TxOriginGrepDetector` - tx.origin usage
- `FloatingPragmaGrepDetector` - Unlocked compiler versions
- `VisibilityGrepDetector` - Visibility issues
- `DeprecatedGrepDetector` - Deprecated features
- `LowLevelCallGrepDetector` - Low-level calls (call, delegatecall, etc.)
- `ShadowingGrepDetector` - Variable shadowing
- `TimestampDependenceGrepDetector` - Block timestamp usage
- `DelegatecallGrepDetector` - Delegatecall patterns
- `MissingAccessControlGrepDetector` - Missing access checks
- `ConstantStateVarGrepDetector` - Non-immutable state constants
- `CentralizationRiskGrepDetector` - Centralization issues

### Detector Registry

The `DetectorRegistry` manages all available detectors:
- Registers detectors by name and ID
- Supports filtering based on configuration
- Collects dependencies for scheduling
- Provides lookup and enumeration capabilities

## Data Flow Analysis Framework

Location: [crates/smarthunt/src/dfa/](../crates/smarthunt/src/dfa/)

### Control Flow Graph (CFG)

- Converts IR into basic blocks
- Tracks control flow edges (branch, fall-through, etc.)
- Precomputes def/use sets for optimization
- Builds successor/predecessor relationships

### Lattice Framework - Abstract Domain

```rust
Lattice Trait {
    top() -> Self,           // Maximum value
    bottom() -> Self,        // Minimum value
    meet() -> Self,          // Join operation
    is_leq() -> bool,        // Partial order
    widen() -> Option<Self>, // Widening operator
}
```

**Built-in Implementations**:
- `FlatLattice<T>` - Two-level lattice (bottom, value, top)
- `PowerSetLattice<T>` - Set membership (powerset)
- `MapLattice<K, V>` - Map-based facts
- `ProductLattice` - Combined lattice

### Worklist Solver

Generic forward/backward analysis engine:
- Configurable transfer functions
- Fixpoint computation with convergence detection
- Returns per-block entry/exit facts

**Transfer Function Trait**:
```rust
Transfer<L: Lattice> {
    transfer_stmt(stmt: &Stmt, fact: &L) -> L,
    transfer_block(block: &BasicBlock, fact: L, dir: Direction) -> L,
}
```

## Pattern Matching Framework

Location: [crates/smarthunt/src/grep/](../crates/smarthunt/src/grep/)

### Pattern Trait

```rust
Pattern {
    match_expr(&Expr) -> Option<Match>,
    match_stmt(&Stmt) -> Option<Match>,
    name() -> &str,
    description() -> &str,
}
```

### Match Result

```rust
Match {
    loc: Option<Loc>,                      // Location in source
    captures: HashMap<String, CapturedNode>, // Captured values
    context: MatchContext,                 // Analysis context
}
```

### Pattern Combinators

- `AndPattern` - All sub-patterns must match
- `OrPattern` - Any sub-pattern matches
- `NotPattern` - Negation
- `ContainsPattern` - Recursive containment
- `WherePattern` - Conditional matching

### Primitive Patterns

- `IdentPattern` - Identifier matching
- `CallPattern` - Function/method calls
- `MemberAccessPattern` - Property access (e.g., tx.origin)
- `AnyExpr` / `AnyStmt` - Wildcards

### PatternBuilder DSL

Fluent API for constructing patterns:

```rust
PatternBuilder::tx_origin()
PatternBuilder::binary_eq(left, right)
PatternBuilder::call(func_name, args)
```

### PatternMatcher

Single-pass multi-pattern matcher:
- Collects all matches across AST in one traversal
- Returns grouped results by pattern name
- Operates on immutable AnalysisContext

Example:
```rust
let pattern = PatternBuilder::tx_origin();
let mut matcher = PatternMatcher::new();
matcher.add_pattern("tx_origin", pattern);
let results = matcher.match_all(&context.source_units, &ctx);
```

## Solidity Processing Pipeline

### 1. Parsing
- Uses `solidity::parser::parse_input_file()`
- Supports multiple files with import resolution
- Handles Solidity compiler selection via `--solc-version`

### 2. AST Representation
- `SourceUnit` - Top-level compilation unit
- `ContractDef` - Contract definitions
- `FuncDef` - Function definitions
- `Expr` / `Stmt` - Expressions and statements
- `Type` - Type information

### 3. IR Generation (Optional)
- Converts AST to IR representation
- Creates control flow graphs
- Builds dataflow facts
- Enables DFA-based detectors

### 4. Analysis Passes
- Decorate AST with symbol tables
- Build call graphs and inheritance graphs
- Analyze modifiers and access control
- Track state mutations

## Output Formats

Location: [crates/smarthunt/src/output/](../crates/smarthunt/src/output/)

### Supported Formats

1. **Text** - Human-readable console output
2. **JSON** - Machine-parseable JSON with metadata, summary, and findings
3. **Markdown** - Formatted report for documentation
4. **SARIF** - Standard Analysis Results Format for tool integration

### Report Structure

```rust
AnalysisReport {
    bugs: Vec<Bug>,
    files_analyzed: Vec<String>,
    duration: Duration,
    stats: BugStats {
        bugs_by_severity,
        bugs_by_category,
    },
}
```

### Bug Structure

```rust
Bug {
    name: String,
    description: Option<String>,
    loc: Loc,                  // Line, column
    kind: BugKind,
    category: BugCategory,
    risk_level: RiskLevel,
    cwe_ids: Vec<usize>,
    swc_ids: Vec<usize>,
}
```

## CLI Interface

Location: [crates/smarthunt/src/main.rs](../crates/smarthunt/src/main.rs)

### Commands

- `analyze <files>` - Main analysis command (default)
- `list-detectors` - Show all available detectors with metadata
- `show-detector <id>` - Display detailed info on a specific detector
- `init-config <output>` - Generate default configuration file

### Key Options

- Input: Solidity files with optional base path and include paths
- `--solc-version` - Compiler version specification
- `--format` - Output format (text, json, markdown, sarif)
- `--parallel` - Enable parallel execution
- `--num-threads` - Number of worker threads
- `--enable` / `--disable` - Detector filtering
- `--severity` - Filter by severity (info, low, medium, high, critical)
- `--output` - Output file path
- `--debug` - Enable debug mode with AST printing

### Execution Flow

1. Parse Solidity source files into AST
2. Create `AnalysisContext` with source units
3. Instantiate `PipelineEngine` with configuration
4. Execute two-phase pipeline (Analysis → Detection)
5. Format output and write results

## Parallelization Strategy

### Analysis Phase
- Parallel by dependency level via PassScheduler
- Passes at same level run concurrently using rayon
- Respects pass dependencies for correctness

### Detection Phase
- Fully parallel execution using rayon
- All detectors run concurrently (read-only access)
- Thread pool sized based on available cores

### Configuration
- Default: Single-threaded
- Enable via `--parallel` flag
- Configure threads with `--num-threads`

## Extension Points

1. **Add New Detectors**: Implement `BugDetectionPass` trait
2. **Add Analysis Passes**: Implement `AnalysisPass` trait
3. **Add Patterns**: Implement `Pattern` trait
4. **Add Transfer Functions**: Implement `Transfer` trait for DFA
5. **Add Output Formatters**: Implement `OutputFormatter` trait

## Performance Characteristics

### Optimizations

- Lazy IR generation minimizes overhead for AST-only detectors
- Detector registry caches lookups
- Pass completion status prevents redundant computation
- Rayon thread pool reused across parallel sections
- Type erasure via `Arc<dyn Any>` enables efficient artifact sharing

### Error Handling

- `PassError` for pass execution failures
- `DetectorError` for detector-specific errors
- `PassResult<T>` for structured error propagation
- Fail-fast option to stop on first error

## Implementation Statistics

- **Total Detectors**: 16 vulnerability detectors
  - 5 DFA-based detectors
  - 11 GREP-based detectors
- **Analysis Passes**: Symbol table, call graph, CFG, etc.
- **Output Formats**: 4 formats (text, JSON, markdown, SARIF)

## Key Files and Locations

- Main entry: [crates/smarthunt/src/main.rs](../crates/smarthunt/src/main.rs)
- Pipeline engine: [crates/smarthunt/src/pipeline/engine.rs](../crates/smarthunt/src/pipeline/engine.rs)
- DFA framework: [crates/smarthunt/src/dfa/](../crates/smarthunt/src/dfa/)
- Pattern matching: [crates/smarthunt/src/grep/](../crates/smarthunt/src/grep/)
- Analysis framework: [crates/smarthunt/src/analysis/](../crates/smarthunt/src/analysis/)
- Output formatters: [crates/smarthunt/src/output/](../crates/smarthunt/src/output/)
