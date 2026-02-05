# SmartHunt: AST-Based Smart Contract Bug Detection Framework

## Executive Summary

SmartHunt is a modular, extensible, and high-performance bug detection framework for Solidity smart contracts. This plan outlines the architecture for building a pass-based analysis system that can efficiently detect vulnerabilities, code quality issues, and optimization opportunities through pattern matching on the Abstract Syntax Tree (AST).

**Design Principles:**
- **Modular:** Independent detector modules that can be composed
- **Extensible:** Easy to add new detectors without modifying core framework
- **Customizable:** Configurable rules, severity levels, and analysis scope
- **Parallelizable:** Concurrent execution of independent analysis passes
- **Efficient:** Minimal redundant work through information sharing and caching

---

## 1. Architecture Overview

### 1.1 Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                      SmartHunt CLI                          │
│  (Input files, config, flags, output format selection)     │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│          solidity crate (soljc infrastructure)              │
│  • Compile Solidity via solc → JSON AST                    │
│  • Parse JSON AST to internal Rust AST                     │
│  • Run 15 normalization passes (already implemented!)      │
│  • Generate IR (optional, already implemented!)            │
│  • Provides: SourceUnit, AST, IR, visitor patterns         │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│                   Analysis Orchestrator                      │
│  • Pass scheduler and dependency resolver                   │
│  • Context management and caching                           │
│  • Parallel execution coordinator                           │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│                    Analysis Passes                           │
│  Phase 1: Fast Local Analysis (per-function, per-contract)  │
│  Phase 2: Inter-procedural Analysis (call graph, CFG)       │
│  Phase 3: Semantic Analysis (data flow, symbolic)           │
│  Phase 4: Cross-contract Analysis (interactions)            │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│                    Bug Detectors                             │
│  • Pattern-based matchers (AST traversal)                   │
│  • Rule-based detectors (declarative patterns)              │
│  • Semantic analyzers (data/control flow)                   │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│                    Results Aggregator                        │
│  • Deduplication and filtering                              │
│  • Severity classification                                  │
│  • Confidence scoring                                       │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│                    Output Formatter                          │
│  • JSON (machine-readable)                                  │
│  • Markdown (human-readable reports)                        │
│  • SARIF (static analysis results interchange format)       │
│  • GitHub Actions annotations                               │
│  • IDE integration (LSP-style diagnostics)                  │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Key Architectural Decisions

**From Slither:**
- Leverage IR for high-precision semantic analysis
- Organize detectors by severity (Critical, High, Medium, Low, Informational) and confidence
- Target < 1 second analysis time per contract
- Provide both API and CLI interfaces

**From Aderyn:**
- Use Rust for performance and safety
- Modular crate structure (already established in verazt)
- Multiple output formats for different use cases
- Focus on foundry/hardhat integration

**From Semgrep:**
- Multi-pass architecture with increasing precision
- Pattern-based rule system for easy extensibility
- Clear separation between fast local analysis and expensive cross-file analysis
- Community-driven detector registry

**Leveraging Existing verazt Infrastructure:**
- **solidity crate** already provides complete parsing and normalization pipeline:
  - Compilation via solc with version negotiation
  - JSON AST parsing to internal Rust representation (includes full type information from solc!)
  - 15 normalization passes (inheritance resolution, modifier expansion, expression flattening, etc.)
  - Optional IR generation for formal verification
  - Visitor/Fold/Map patterns for AST/IR traversal
  - **Complete type information** for all expressions (from solc's semantic analysis)
- **bugs crate** already provides bug/issue reporting data structures:
  - `Bug` struct with all necessary fields (name, description, location, kind, risk level, CWE/SWC IDs)
  - `BugKind` enum (Vulnerability, Refactoring, Optimization)
  - `RiskLevel` enum (Critical, High, Medium, Low, Informational/No)
- SmartHunt builds on top of this solid foundation, focusing on analysis passes and detectors

---

## 2. Pass-Based Analysis Architecture

### 2.1 Pass Scheduling and Dependencies

Analysis is organized into **passes** that execute in a specific order. Each pass:
- Has explicit dependencies on other passes
- Produces analysis artifacts consumed by later passes
- Can be skipped if not required by enabled detectors
- Can run in parallel with other passes when dependencies are met

### 2.2 Pass Phases

**Note:** Compilation, normalization (15 passes), and type information are already provided by the **solidity crate**. The passes below are what SmartHunt needs to implement.

**Legend:**
- 🌳 **AST-based pass** - operates on the normalized Abstract Syntax Tree
- ⚙️ **IR-based pass** - operates on the Intermediate Representation (optional, more precise)
- 🔄 **Hybrid pass** - can use both AST and IR depending on configuration

#### Phase 1: Context Building (Sequential)

**Pass 1.1: Symbol Table Extraction** 🌳 **AST-based**
- **Purpose:** Build fast lookup structures for program entities
- **Operations:**
  - Extract all contracts, functions, variables from normalized AST
  - Build name → definition mappings
  - Extract inheritance hierarchy (already linearized by solidity crate)
- **Artifacts Produced:**
  - `SymbolTable`: Fast lookup for all definitions
  - `InheritanceGraph`: Contract inheritance relationships
- **Consumers:** All subsequent passes

**Pass 1.2: Type Index Construction** 🌳 **AST-based**
- **Purpose:** Build fast type lookup structures
- **Operations:**
  - Index type information (already in AST from solc)
  - Create type relationship graph
  - Add convenience methods for common type checks
- **Artifacts Produced:**
  - `TypeIndex`: Fast type queries
  - `TypeGraph`: Type relationships (inheritance, conversions)
- **Consumers:** All detection passes requiring type information

**Pass 1.3: Control Flow Graph (CFG) Construction** 🔄 **Hybrid (AST or IR)**
- **Purpose:** Build control flow graphs for all functions
- **Operations:**
  - Create basic blocks for each function
  - Identify entry/exit points
  - Handle loops, conditionals, try-catch
  - Link function calls
- **Input:** Can work on either AST (normalized, flattened expressions) or IR (atomic operations)
- **Recommendation:** Use IR when available for more precise CFG (atomic operations are easier to analyze)
- **Artifacts Produced:**
  - `ControlFlowGraph`: Per-function CFG
  - `BasicBlock`: List of basic blocks per function
- **Consumers:** Data flow analysis, reachability analysis, reentrancy detection

**Pass 1.4: Call Graph Construction** 🌳 **AST-based**
- **Purpose:** Build inter-procedural call relationships
- **Operations:**
  - Track direct function calls
  - Resolve virtual calls (interface/inheritance)
  - Identify external calls (cross-contract)
  - Detect recursive calls
- **Artifacts Produced:**
  - `CallGraph`: Global call graph
  - `ExternalCallSites`: Locations of external calls
  - `RecursiveFunctions`: Set of recursive functions
- **Consumers:** Inter-procedural analysis, reentrancy detection, gas analysis

---

#### Phase 2: Local Analysis (Parallelizable)

These passes analyze individual functions or contracts in isolation and can run concurrently.

**Pass 2.1: Local Pattern Matching** 🌳 **AST-based**
- **Purpose:** Fast AST pattern detection
- **Detectors:**
  - Deprecated constructs (`tx.origin`, `block.timestamp` misuse)
  - Dangerous patterns (`selfdestruct`, `delegatecall`)
  - Style violations (visibility, naming conventions)
  - Low-level calls without checks
  - Unchecked return values
  - Floating pragma
  - Missing SPDX license
- **Parallelization:** One thread per contract or function
- **Estimated Time:** < 50ms per contract

**Pass 2.2: Local Semantic Analysis** 🌳 **AST-based**
- **Purpose:** Function-level semantic checks
- **Detectors:**
  - Uninitialized local variables
  - Dead code detection
  - Redundant statements
  - Type confusion bugs
  - Integer overflow/underflow (pre-0.8.0)
  - Division by zero
  - Array out of bounds (static)
- **Parallelization:** One thread per function
- **Estimated Time:** 50-100ms per contract

**Pass 2.3: State Variable Analysis** 🌳 **AST-based**
- **Purpose:** Analyze state variable usage
- **Detectors:**
  - Uninitialized state variables
  - Constant/immutable candidates
  - Storage packing opportunities
  - Unused state variables
  - Shadow state variables
- **Parallelization:** One thread per contract
- **Estimated Time:** 20-50ms per contract

---

#### Phase 3: Inter-procedural Analysis (Partially Parallelizable)

These passes require call graph or CFG and analyze interactions between functions.

**Pass 3.1: Data Flow Analysis** 🔄 **Hybrid (better on IR for precision)**
- **Purpose:** Track how data flows through functions
- **Operations:**
  - Reaching definitions analysis
  - Live variable analysis
  - Taint analysis (user input → sensitive operations)
  - Constant propagation
- **Input:** Can work on AST CFG or IR CFG
- **Recommendation:** Use IR for higher precision (atomic operations make tracking clearer)
- **Artifacts Produced:**
  - `TaintGraph`: Sources (user input) → sinks (sensitive ops)
  - `DefUseChains`: Variable definition-use relationships
- **Detectors:**
  - Reentrancy vulnerabilities
  - Unchecked external call return values
  - Unvalidated user input
  - Access control bypass
- **Parallelization:** Parallel analysis per contract (separate CFGs)
- **Estimated Time:** 100-300ms per contract

**Pass 3.2: State Mutation Analysis** 🔄 **Hybrid (AST or IR)**
- **Purpose:** Track state changes across function calls
- **Operations:**
  - Identify read/write operations on state variables
  - Track state dependencies
  - Detect state inconsistencies
- **Artifacts Produced:**
  - `StateMutationMap`: Which functions read/write which state vars
  - `StateEffects`: Side effects of each function
- **Detectors:**
  - State variable shadowing
  - Write-after-write bugs
  - Check-effect-interaction violations
  - Race conditions
- **Parallelization:** Per-contract analysis
- **Estimated Time:** 100-200ms per contract

**Pass 3.3: Access Control Analysis** 🌳 **AST-based**
- **Purpose:** Verify access control patterns
- **Operations:**
  - Identify privileged functions (onlyOwner, onlyAdmin, etc.)
  - Track authentication checks
  - Verify authorization before sensitive operations
- **Detectors:**
  - Missing access control
  - Centralization risks
  - Privilege escalation
  - Authorization bypass
- **Parallelization:** Per-contract analysis
- **Estimated Time:** 50-150ms per contract

**Pass 3.4: Reentrancy Analysis** 🔄 **Hybrid (AST for basic, IR for deep analysis)**
- **Purpose:** Deep reentrancy detection
- **Operations:**
  - Identify external calls (untrusted)
  - Track state mutations before/after external calls
  - Analyze call chains
  - Check CEI (Checks-Effects-Interactions) pattern
- **Detectors:**
  - Classic reentrancy
  - Cross-function reentrancy
  - Read-only reentrancy
  - Cross-contract reentrancy
- **Parallelization:** Per-contract with inter-contract edge analysis
- **Estimated Time:** 200-500ms per contract

---

#### Phase 4: Cross-Contract Analysis (Limited Parallelization)

These passes analyze interactions between multiple contracts.

**Pass 4.1: External Interaction Analysis** 🌳 **AST-based**
- **Purpose:** Analyze cross-contract calls
- **Operations:**
  - Map external call dependencies
  - Track interface usage
  - Identify proxy patterns
  - Detect upgrade mechanisms
- **Detectors:**
  - Unsafe external calls
  - Interface mismatches
  - Proxy storage collision
  - Upgrade vulnerabilities
- **Parallelization:** Grouped by contract clusters
- **Estimated Time:** 100-500ms per contract cluster

**Pass 4.2: Economic/Game Theory Analysis** 🌳 **AST-based**
- **Purpose:** Detect economic vulnerabilities
- **Operations:**
  - Track value flows (ETH, tokens)
  - Identify incentive structures
  - Analyze fee mechanisms
- **Detectors:**
  - Front-running vulnerabilities
  - MEV extraction opportunities
  - Economic exploits (price manipulation)
  - Flash loan attack vectors
- **Parallelization:** Per transaction flow
- **Estimated Time:** 200-1000ms per contract

---

#### Phase 5: Advanced Semantic Analysis (Optional, Expensive)

These passes are opt-in for deep analysis.

**Pass 5.1: Symbolic Execution (Limited)** ⚙️ **IR-based (requires IR)**
- **Purpose:** Explore execution paths symbolically
- **Operations:**
  - Bounded symbolic execution (limited depth)
  - Path condition collection
  - SMT solver integration (Z3)
- **Input:** **Requires IR** (atomic operations essential for symbolic execution)
- **Why IR:** Complex expressions must be broken down into atomic operations for symbolic analysis
- **Detectors:**
  - Unreachable code
  - Assert violations
  - Overflow/underflow (all paths)
  - Logic errors
- **Parallelization:** Per function with path limit
- **Estimated Time:** 1-10s per function (depth-limited)

**Pass 5.2: Formal Verification (Limited)** ⚙️ **IR-based (requires IR)**
- **Purpose:** Prove properties using formal methods
- **Operations:**
  - Convert to verification conditions
  - Use SMT solvers for property checking
- **Input:** **Requires IR** (verification requires normalized, atomic operations)
- **Why IR:** Formal methods need simplified, normalized representation
- **Detectors:**
  - Invariant violations
  - Specification mismatches
- **Note:** This bridges to `smartproof` crate
- **Estimated Time:** Seconds to minutes (opt-in only)

---

### 2.3 Pass Dependency Graph

**Legend:** 🌳 AST-based | ⚙️ IR-based | 🔄 Hybrid

```
solidity crate (Compilation + Normalization + Types)
    │  ✅ Already provided!
    │  Produces: AST (normalized) + optional IR
    │
    ▼
Pass 1.1 🌳 (Symbol Table Extraction)
    │
    ├─→ Pass 1.2 🌳 (Type Index)
    │       │
    │       └─→ Pass 2.2 🌳 (Local Semantic)
    │
    ├─→ Pass 1.3 🔄 (CFG Construction) - works on AST or IR
    │       │
    │       ├─→ Pass 3.1 🔄 (Data Flow) - better on IR
    │       │       │
    │       │       └─→ Pass 3.4 🔄 (Reentrancy)
    │       │
    │       └─→ Pass 5.1 ⚙️ (Symbolic Execution) - requires IR
    │
    ├─→ Pass 1.4 🌳 (Call Graph)
    │       │
    │       ├─→ Pass 3.1 🔄 (Data Flow)
    │       ├─→ Pass 3.2 🔄 (State Mutation)
    │       ├─→ Pass 3.4 🔄 (Reentrancy)
    │       └─→ Pass 4.1 🌳 (External Interaction)
    │
    └─→ Pass 2.1 🌳 (Local Patterns) ─┐
                                      ├─→ Results Aggregation
    Pass 2.3 🌳 (State Variables) ────┤
                                      │
    Pass 3.3 🌳 (Access Control) ─────┤
                                      │
    Pass 4.2 🌳 (Economic Analysis) ──┘
```

---

## 3. Detector Framework

### 3.1 Detector Taxonomy

Detectors are organized by **category** and **complexity**:

#### Category 1: Vulnerability Detectors
- **Critical:** Reentrancy, access control bypass, integer overflow
- **High:** Unchecked external calls, delegatecall to untrusted, tx.origin
- **Medium:** Front-running, block timestamp manipulation, DoS vectors
- **Low:** Race conditions, missing events, improper validation

#### Category 2: Code Quality Detectors
- **Informational:** Dead code, unused variables, naming conventions
- **Refactoring:** Duplicate code, complex functions, style violations

#### Category 3: Gas Optimization Detectors
- **Optimization:** Storage packing, constant candidates, loop optimizations
- **Cost:** Expensive operations in loops, redundant storage reads

#### Category 4: Best Practices Detectors
- **Informational:** Missing NatSpec, floating pragma, SPDX license
- **Style:** Visibility, ordering, naming conventions

### 3.2 Detector Interface

All detectors implement a unified trait:

```rust
use bugs::{Bug, BugKind, RiskLevel};  // Reuse existing bugs crate!

pub trait Detector: Send + Sync {
    /// Unique detector identifier
    fn id(&self) -> &'static str;

    /// Human-readable detector name
    fn name(&self) -> &'static str;

    /// Detailed description of what this detector finds
    fn description(&self) -> &'static str;

    /// Which passes must run before this detector
    fn required_passes(&self) -> Vec<PassId>;

    /// Bug category (Vulnerability, Refactoring, Optimization) from bugs crate
    fn bug_kind(&self) -> BugKind;

    /// Risk level (Critical, High, Medium, Low, No/Informational) from bugs crate
    fn risk_level(&self) -> RiskLevel;

    /// Confidence level (High, Medium, Low)
    fn confidence(&self) -> ConfidenceLevel;

    /// Associated CWE IDs
    fn cwe_ids(&self) -> Vec<usize>;

    /// Associated SWC IDs (Smart Contract Weakness Classification)
    fn swc_ids(&self) -> Vec<usize>;

    /// Run the detector and return findings using Bug struct from bugs crate
    fn detect(&self, context: &AnalysisContext) -> Vec<Bug>;
}
```

**Note:** We reuse the `Bug`, `BugKind`, and `RiskLevel` types from the existing **bugs crate**, ensuring consistency across all verazt tools (smarthunt, smartproof, etc.).

### 3.3 Detector Registration

Detectors are registered in a central registry:

```rust
pub struct DetectorRegistry {
    detectors: HashMap<String, Box<dyn Detector>>,
    by_category: HashMap<BugKind, Vec<String>>,
    by_severity: HashMap<RiskLevel, Vec<String>>,
}

impl DetectorRegistry {
    pub fn register(&mut self, detector: Box<dyn Detector>) {
        let id = detector.id().to_string();
        self.by_category
            .entry(detector.bug_kind())
            .or_default()
            .push(id.clone());
        self.by_severity
            .entry(detector.risk_level())
            .or_default()
            .push(id.clone());
        self.detectors.insert(id, detector);
    }

    pub fn get_enabled(&self, config: &Config) -> Vec<&dyn Detector> {
        // Filter based on config (severity threshold, categories, etc.)
    }
}
```

### 3.4 Pattern-Based Detector DSL

For simple pattern matching, provide a declarative DSL:

```rust
use bugs::{Bug, BugKind, RiskLevel};  // From bugs crate

// Example: Detect tx.origin usage
detector! {
    id: "tx-origin-usage",
    name: "Dangerous use of tx.origin",
    description: "tx.origin can be manipulated by intermediate contracts",
    bug_kind: BugKind::Vulnerability,  // From bugs crate
    risk_level: RiskLevel::High,       // From bugs crate
    confidence: High,
    cwe: [345],
    swc: [115],

    pattern: {
        // Match any expression that is a member access to tx.origin
        Expr::Member(MemberExpr {
            base: Expr::Ident(Ident { name: "tx", .. }),
            member: "origin",
            ..
        })
    },

    suggest: "Use msg.sender instead of tx.origin for authentication"
}
```

This macro expands to a full Detector implementation that creates `Bug` instances from the bugs crate.

---

## 4. Analysis Context and Caching

### 4.1 Analysis Context

The `AnalysisContext` holds all analysis artifacts and provides a unified interface:

```rust
pub struct AnalysisContext {
    /// Original AST (includes type info from solc!)
    pub source_units: Vec<SourceUnit>,

    /// Optional IR
    pub ir_units: Option<Vec<ir::SourceUnit>>,

    /// Symbol tables
    pub symbols: SymbolTable,
    pub inheritance: InheritanceGraph,

    /// Type system (types already in AST, this is just an index for fast lookup)
    pub type_index: TypeIndex,

    /// Control flow
    pub cfgs: HashMap<FunctionId, ControlFlowGraph>,
    pub call_graph: CallGraph,

    /// Data flow
    pub taint_graph: Option<TaintGraph>,
    pub def_use_chains: Option<DefUseChains>,

    /// State analysis
    pub state_mutations: Option<StateMutationMap>,

    /// Configuration
    pub config: Config,
}

impl AnalysisContext {
    /// Get CFG for a function
    pub fn cfg(&self, func: &FuncDef) -> Option<&ControlFlowGraph> {
        self.cfgs.get(&func.id())
    }

    /// Get all external calls in a function
    pub fn external_calls(&self, func: &FuncDef) -> Vec<&CallExpr> {
        // Query call graph
    }

    /// Check if a function modifies state
    pub fn modifies_state(&self, func: &FuncDef) -> bool {
        // Query state mutation map
    }

    /// Get taint sources for a variable
    pub fn taint_sources(&self, var: &VarDecl) -> Vec<TaintSource> {
        // Query taint graph
    }
}
```

### 4.2 Caching Strategy

To avoid redundant computation:

1. **Inter-pass caching:**
   - Store analysis artifacts in AnalysisContext
   - Passes mark their outputs as cached
   - Subsequent passes query cache first

2. **Incremental analysis:**
   - Hash AST nodes to detect changes
   - Only re-analyze changed functions/contracts
   - Cache results from previous runs

3. **Persistent caching:**
   - Serialize analysis results to disk (optional)
   - Use file hashes as cache keys
   - Useful for large codebases with CI/CD

---

## 5. Parallelization Strategy

### 5.1 Parallel Pass Execution

Passes are scheduled based on dependencies:

```rust
pub struct PassScheduler {
    passes: Vec<Box<dyn AnalysisPass>>,
    dependency_graph: PassDependencyGraph,
}

impl PassScheduler {
    /// Execute passes in topological order, parallelizing when possible
    pub async fn execute(&self, context: &mut AnalysisContext) {
        let sorted = self.dependency_graph.topological_sort();

        for level in sorted.levels() {
            // All passes in a level can run in parallel
            let futures: Vec<_> = level
                .iter()
                .map(|pass| pass.run(context))
                .collect();

            // Wait for all passes in this level to complete
            futures::future::join_all(futures).await;
        }
    }
}
```

### 5.2 Parallel Detector Execution

Detectors within the same pass can run concurrently:

```rust
pub async fn run_detectors(
    detectors: Vec<&dyn Detector>,
    context: &AnalysisContext,
) -> Vec<Bug> {
    use rayon::prelude::*;

    detectors
        .par_iter()
        .flat_map(|detector| detector.detect(context))
        .collect()
}
```

### 5.3 Parallel Contract Analysis

For large projects with many contracts:

```rust
pub fn analyze_contracts_parallel(
    contracts: Vec<Contract>,
    context: &AnalysisContext,
) -> Vec<Bug> {
    use rayon::prelude::*;

    contracts
        .par_iter()
        .flat_map(|contract| {
            // Analyze each contract independently
            let contract_context = context.scope_to_contract(contract);
            run_all_detectors(&contract_context)
        })
        .collect()
}
```

### 5.4 Thread Pool Configuration

```rust
pub struct ParallelConfig {
    /// Number of worker threads (default: num CPUs)
    pub num_threads: usize,

    /// Minimum work size for parallelization
    pub min_parallel_size: usize,

    /// Enable work stealing
    pub work_stealing: bool,
}
```

---

## 6. Configuration and Customization

### 6.1 Configuration File Format

Support TOML configuration:

```toml
[smarthunt]
# Analysis scope
enable_ir_analysis = true
enable_symbolic_execution = false
max_symbolic_depth = 5

# Performance
num_threads = 8
cache_enabled = true
cache_dir = ".smarthunt-cache"

# Output
output_format = "json"  # json, markdown, sarif
output_file = "smarthunt-report.json"
verbose = true

# Filtering
min_severity = "medium"  # critical, high, medium, low, informational
exclude_informational = false
exclude_optimization = false

# Detector selection
[detectors]
# Enable/disable by category
vulnerabilities = true
refactoring = true
optimization = true
best_practices = true

# Enable/disable individual detectors
reentrancy = true
tx-origin = true
unchecked-call = true
centralization-risk = false  # Disable specific detector

# Custom detector configuration
[detectors.reentrancy]
check_read_only = true
max_call_depth = 3

[detectors.gas-optimization]
threshold_loops = 10
suggest_packing = true

# Ignore patterns
[ignore]
# Ignore by file path
files = [
    "contracts/test/**",
    "contracts/mocks/**",
]

# Ignore by detector + location
[[ignore.rules]]
detector = "centralization-risk"
file = "contracts/Ownable.sol"
reason = "Intentional design pattern"

[[ignore.rules]]
detector = "tx-origin"
function = "emergencyWithdraw"
reason = "Reviewed and accepted"
```

### 6.2 Command-Line Interface

```bash
smarthunt [OPTIONS] <INPUT_FILES>

OPTIONS:
    --config <FILE>              Configuration file (TOML)
    --base-path <PATH>           Base path for imports
    --include-path <PATH>        Additional include paths (repeatable)
    --solc-version <VERSION>     Solc version to use

    # Analysis options
    --no-ir                      Skip IR generation
    --enable-symbolic            Enable symbolic execution
    --max-depth <N>              Max symbolic execution depth

    # Performance
    --threads <N>                Number of threads
    --cache / --no-cache         Enable/disable caching
    --cache-dir <PATH>           Cache directory

    # Output
    --format <FORMAT>            Output format (json, markdown, sarif)
    --output <FILE>              Output file (default: stdout)
    --verbose / --quiet          Verbosity level

    # Filtering
    --severity <LEVEL>           Minimum severity (critical, high, medium, low, info)
    --only-vulnerabilities       Only show vulnerabilities
    --only-high-confidence       Only show high confidence findings

    # Detector selection
    --detector <ID>              Enable specific detector (repeatable)
    --exclude <ID>               Disable specific detector (repeatable)
    --list-detectors             List all available detectors

    # Ignore
    --ignore-file <FILE>         Ignore file (TOML format)

    # Profiling
    --profile-time               Profile execution time per pass
    --profile-memory             Profile memory usage

EXAMPLES:
    # Analyze all contracts in src/
    smarthunt src/**/*.sol

    # Analyze with custom config
    smarthunt --config smarthunt.toml src/

    # Only high-severity vulnerabilities
    smarthunt --severity high --only-vulnerabilities src/

    # Enable specific detectors
    smarthunt --detector reentrancy --detector unchecked-call src/

    # Generate JSON report
    smarthunt --format json --output report.json src/

    # Fast analysis (skip expensive passes)
    smarthunt --no-ir --threads 16 src/
```

### 6.3 Detector Registry and Discovery

Support dynamic detector loading:

```rust
// Built-in detectors are automatically registered
pub fn register_builtin_detectors(registry: &mut DetectorRegistry) {
    registry.register(Box::new(ReentrancyDetector::default()));
    registry.register(Box::new(TxOriginDetector::default()));
    registry.register(Box::new(UncheckedCallDetector::default()));
    // ... 50+ more
}

// Load custom detectors from plugins (future)
pub fn load_plugin_detectors(
    registry: &mut DetectorRegistry,
    plugin_dir: &Path,
) -> Result<()> {
    // Scan for .so/.dylib files
    // Load via dlopen
    // Call registration function
}
```

---

## 7. Output Formats

### 7.1 JSON Format

```json
{
  "version": "1.0.0",
  "smarthunt_version": "0.1.0",
  "timestamp": "2026-02-05T10:30:00Z",
  "analysis_time_ms": 1234,
  "files_analyzed": 15,
  "contracts_analyzed": 42,

  "summary": {
    "total_issues": 23,
    "by_severity": {
      "critical": 2,
      "high": 5,
      "medium": 8,
      "low": 5,
      "informational": 3
    },
    "by_category": {
      "vulnerability": 15,
      "refactoring": 3,
      "optimization": 5
    }
  },

  "issues": [
    {
      "id": "vuln-001",
      "detector": "reentrancy",
      "detector_name": "Reentrancy Vulnerability",
      "description": "Function is vulnerable to reentrancy attack",
      "severity": "critical",
      "confidence": "high",
      "category": "vulnerability",
      "cwe": [841],
      "swc": [107],

      "location": {
        "file": "contracts/Bank.sol",
        "line": 42,
        "column": 5,
        "function": "withdraw",
        "contract": "Bank"
      },

      "code_snippet": "function withdraw() public {\n    msg.sender.call{value: balance}(\"\");\n    balance = 0;\n}",

      "explanation": "The external call to msg.sender occurs before the state variable 'balance' is updated. An attacker can re-enter this function and drain funds.",

      "recommendation": "Follow the Checks-Effects-Interactions pattern: update state before making external calls.",

      "suggested_fix": "function withdraw() public {\n    uint amount = balance;\n    balance = 0;\n    msg.sender.call{value: amount}(\"\");\n}",

      "references": [
        "https://swcregistry.io/docs/SWC-107",
        "https://consensys.github.io/smart-contract-best-practices/attacks/reentrancy/"
      ]
    }
  ],

  "performance": {
    "passes": [
      {"name": "AST Preprocessing", "time_ms": 45},
      {"name": "CFG Construction", "time_ms": 123},
      {"name": "Data Flow Analysis", "time_ms": 456},
      {"name": "Reentrancy Detection", "time_ms": 234}
    ]
  }
}
```

### 7.2 Markdown Format

```markdown
# SmartHunt Analysis Report

**Generated:** 2026-02-05 10:30:00
**Analysis Time:** 1.234s
**Files Analyzed:** 15
**Contracts Analyzed:** 42

## Summary

| Severity | Count |
|----------|-------|
| Critical | 2     |
| High     | 5     |
| Medium   | 8     |
| Low      | 5     |
| Info     | 3     |
| **Total**| **23**|

---

## Critical Issues

### 1. Reentrancy Vulnerability

**File:** `contracts/Bank.sol:42`
**Function:** `withdraw`
**Detector:** `reentrancy`
**Confidence:** High

**Description:**
The external call to `msg.sender` occurs before the state variable `balance` is updated. An attacker can re-enter this function and drain funds.

**Code:**
```solidity
function withdraw() public {
    msg.sender.call{value: balance}("");
    balance = 0;
}
```

**Recommendation:**
Follow the Checks-Effects-Interactions pattern: update state before making external calls.

**Suggested Fix:**
```solidity
function withdraw() public {
    uint amount = balance;
    balance = 0;
    msg.sender.call{value: amount}("");
}
```

**References:**
- [SWC-107](https://swcregistry.io/docs/SWC-107)
- [Reentrancy Attacks](https://consensys.github.io/smart-contract-best-practices/attacks/reentrancy/)

---
```

### 7.3 SARIF Format

Support Static Analysis Results Interchange Format for IDE integration:

```json
{
  "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
  "version": "2.1.0",
  "runs": [
    {
      "tool": {
        "driver": {
          "name": "SmartHunt",
          "version": "0.1.0",
          "informationUri": "https://github.com/verazt/smarthunt",
          "rules": [
            {
              "id": "reentrancy",
              "name": "ReentrancyVulnerability",
              "shortDescription": {
                "text": "Reentrancy vulnerability detected"
              },
              "fullDescription": {
                "text": "Function is vulnerable to reentrancy attack..."
              },
              "helpUri": "https://swcregistry.io/docs/SWC-107",
              "properties": {
                "category": "vulnerability",
                "severity": "critical",
                "cwe": [841],
                "swc": [107]
              }
            }
          ]
        }
      },
      "results": [
        {
          "ruleId": "reentrancy",
          "level": "error",
          "message": {
            "text": "Reentrancy vulnerability: external call before state update"
          },
          "locations": [
            {
              "physicalLocation": {
                "artifactLocation": {
                  "uri": "contracts/Bank.sol"
                },
                "region": {
                  "startLine": 42,
                  "startColumn": 5
                }
              }
            }
          ]
        }
      ]
    }
  ]
}
```

---

## 8. Priority Detectors to Implement

### Phase 1: Core Vulnerability Detectors (Critical/High)

1. **Reentrancy Detection** (SWC-107)
   - Classic reentrancy
   - Cross-function reentrancy
   - Read-only reentrancy

2. **Access Control Issues** (SWC-105, SWC-106)
   - Missing access control modifiers
   - Unprotected initialization
   - Privilege escalation

3. **Unchecked External Calls** (SWC-104)
   - Unchecked low-level calls
   - Unchecked ERC20 transfers
   - Send/transfer vs call

4. **Integer Overflow/Underflow** (SWC-101)
   - Pre-0.8.0 contracts without SafeMath
   - Unchecked arithmetic

5. **Delegatecall to Untrusted** (SWC-112)
   - Delegatecall to user-controlled address
   - Proxy storage collision

6. **Tx.Origin Authentication** (SWC-115)
   - Using tx.origin for auth

7. **Uninitialized Storage** (SWC-109)
   - Uninitialized storage pointers
   - Uninitialized state variables

8. **Denial of Service** (SWC-113, SWC-128)
   - Gas limit DoS (unbounded loops)
   - Revert DoS (failed transfers)

### Phase 2: Medium Severity Detectors

9. **Block Timestamp Manipulation** (SWC-116)
10. **Weak Randomness** (SWC-120)
11. **Front-Running** (Transaction Ordering Dependence)
12. **Signature Malleability** (SWC-117, SWC-121, SWC-122)
13. **Missing Event Emission**
14. **Shadowing State Variables** (SWC-119)
15. **Incorrect Inheritance Order**

### Phase 3: Code Quality & Best Practices

16. **Dead Code Detection**
17. **Unused Variables**
18. **Redundant Statements**
19. **Style Violations** (Solidity style guide)
20. **Missing NatSpec Documentation**
21. **Floating Pragma**
22. **Outdated Solidity Version**

### Phase 4: Gas Optimization

23. **Storage Packing Opportunities**
24. **Constant/Immutable Candidates**
25. **Redundant Storage Reads**
26. **Expensive Operations in Loops**
27. **Public vs External Functions**
28. **Short-Circuit Optimization**

### Phase 5: Advanced Semantic Detectors

29. **Price Manipulation** (Oracle attacks)
30. **Flash Loan Vulnerabilities**
31. **MEV Extraction Opportunities**
32. **Cross-Contract Reentrancy**
33. **Unsafe External Call Chains**

---

## 9. Implementation Roadmap

### Milestone 1: Core Infrastructure (Weeks 1-2)

**Goals:**
- Refactor existing TaskGenerator into PassScheduler
- Implement AnalysisContext and artifact caching
- Implement Detector trait and registry
- Implement Pass trait and dependency system
- **Integrate with existing solidity crate APIs**

**Deliverables:**
- `smarthunt/src/engine/pass.rs` - Pass trait
- `smarthunt/src/engine/scheduler.rs` - Pass scheduler
- `smarthunt/src/engine/context.rs` - Analysis context
  - Use `solidity::compile::compile_input_file()` for compilation
  - Use `solidity::ast::normalize::run_passes()` for normalization
  - Use `solidity::codegen::lower_source_unit()` for IR generation
- `smarthunt/src/detector/mod.rs` - Detector trait and registry
- `smarthunt/src/detector/registry.rs` - Detector registration

**Testing:**
- Unit tests for pass scheduling
- Integration test with 2-3 simple passes
- Test integration with solidity crate compilation/normalization

---

### Milestone 2: Context Building & Graph Construction (Weeks 3-4)

**Goals:**
- Extract symbol tables from normalized AST (provided by solidity crate)
- Build type index for fast lookups (types already in AST from solc)
- Implement CFG construction
- Implement call graph construction

**Deliverables:**
- `smarthunt/src/passes/symbol_table.rs` - Extract symbols from normalized AST
- `smarthunt/src/passes/type_index.rs` - Index types for fast queries
- `smarthunt/src/passes/cfg.rs` - CFG construction
- `smarthunt/src/passes/call_graph.rs` - Call graph construction
- `smarthunt/src/graph/cfg.rs` - CFG data structures
- `smarthunt/src/graph/call_graph.rs` - Call graph data structures
- `smarthunt/src/graph/symbol_table.rs` - Symbol table data structures
- `smarthunt/src/graph/type_index.rs` - Type index data structures

**Testing:**
- Test symbol table extraction from normalized AST
- Test type index queries
- Test CFG construction on various control flow patterns
- Test call graph on inheritance and interfaces

---

### Milestone 3: Local Analysis Detectors (Weeks 5-6)

**Goals:**
- Implement 10 pattern-based detectors
- Implement local semantic analysis pass

**Deliverables:**
- `smarthunt/src/detectors/tx_origin.rs`
- `smarthunt/src/detectors/floating_pragma.rs`
- `smarthunt/src/detectors/low_level_call.rs`
- `smarthunt/src/detectors/unchecked_return.rs`
- `smarthunt/src/detectors/deprecated.rs`
- `smarthunt/src/detectors/visibility.rs`
- `smarthunt/src/detectors/shadowing.rs`
- `smarthunt/src/detectors/uninitialized_local.rs`
- `smarthunt/src/detectors/dead_code.rs`
- `smarthunt/src/detectors/style.rs`
- `smarthunt/src/passes/local_semantic.rs`

**Testing:**
- Test each detector with positive and negative cases
- Create test contract suite

---

### Milestone 4: Data Flow Analysis (Weeks 7-8)

**Goals:**
- Implement data flow analysis pass
- Implement taint analysis
- Implement reentrancy detector

**Deliverables:**
- `smarthunt/src/passes/data_flow.rs`
- `smarthunt/src/analysis/taint.rs`
- `smarthunt/src/analysis/reaching_defs.rs`
- `smarthunt/src/detectors/reentrancy.rs`
- `smarthunt/src/detectors/unchecked_call.rs`
- `smarthunt/src/detectors/unvalidated_input.rs`

**Testing:**
- Test reentrancy detection on classic cases
- Test cross-function reentrancy
- Test read-only reentrancy

---

### Milestone 5: State & Access Control Analysis (Weeks 9-10)

**Goals:**
- Implement state mutation analysis
- Implement access control analysis
- Implement related detectors

**Deliverables:**
- `smarthunt/src/passes/state_mutation.rs`
- `smarthunt/src/passes/access_control.rs`
- `smarthunt/src/detectors/missing_access_control.rs`
- `smarthunt/src/detectors/centralization_risk.rs`
- `smarthunt/src/detectors/cei_violation.rs`
- `smarthunt/src/detectors/state_shadowing.rs`

**Testing:**
- Test access control on OpenZeppelin patterns
- Test CEI (Checks-Effects-Interactions) violations

---

### Milestone 6: Parallelization & Performance (Weeks 11-12)

**Goals:**
- Implement parallel pass execution
- Implement parallel detector execution
- Add caching and incremental analysis
- Performance benchmarking

**Deliverables:**
- Parallel scheduler implementation
- Thread pool configuration
- Cache layer (in-memory + persistent)
- Benchmark suite
- Performance profiling

**Target Performance:**
- < 1 second per contract (local analysis)
- < 5 seconds per contract (full analysis)
- Linear scaling with number of cores

---

### Milestone 7: Output & Integration (Weeks 13-14)

**Goals:**
- Implement all output formats
- Implement CLI
- Implement configuration system
- Create documentation

**Deliverables:**
- `smarthunt/src/output/json.rs`
- `smarthunt/src/output/markdown.rs`
- `smarthunt/src/output/sarif.rs`
- `smarthunt/src/config.rs` - Config file parsing
- `smarthunt/src/cli.rs` - Command-line interface
- `README.md` - User documentation
- `docs/detectors.md` - Detector reference
- `docs/configuration.md` - Config reference

---

### Milestone 8: Advanced Detectors (Weeks 15-16)

**Goals:**
- Implement cross-contract analysis
- Implement economic/game theory detectors
- Implement gas optimization detectors

**Deliverables:**
- `smarthunt/src/passes/cross_contract.rs`
- `smarthunt/src/passes/economic.rs`
- `smarthunt/src/detectors/unsafe_external.rs`
- `smarthunt/src/detectors/frontrunning.rs`
- `smarthunt/src/detectors/price_manipulation.rs`
- `smarthunt/src/detectors/gas_optimization/` (multiple)

---

### Milestone 9: Polish & Testing (Weeks 17-18)

**Goals:**
- Comprehensive test suite
- Benchmarking against Slither/Aderyn
- Bug fixes and optimizations
- Documentation refinement

**Deliverables:**
- 100+ test contracts covering all detectors
- Benchmark comparison report
- CI/CD integration (GitHub Actions)
- VSCode extension integration
- Release 0.1.0

---

## 10. Testing Strategy

### 10.1 Unit Testing

- Test each detector independently with minimal contracts
- Test each pass with isolated AST fragments
- Test utility functions (CFG construction, taint analysis, etc.)

### 10.2 Integration Testing

- Test complete analysis pipeline on real contracts
- Test pass scheduling and dependencies
- Test parallelization correctness

### 10.3 Regression Testing

- Maintain suite of vulnerable contracts from:
  - SWC Registry examples
  - Historical exploits (DAO hack, Parity wallet, etc.)
  - CTF challenges (Ethernaut, Damn Vulnerable DeFi)
- Ensure all known vulnerabilities are detected

### 10.4 False Positive Testing

- Test on well-audited contracts (OpenZeppelin, Uniswap, AAVE)
- Measure false positive rate
- Tune confidence levels

### 10.5 Performance Testing

- Benchmark on large codebases (100+ contracts)
- Measure memory usage
- Test scalability with parallel execution

---

## 11. Future Extensions

### 11.1 Rule Engine

Allow users to define custom detectors using a DSL:

```yaml
rules:
  - id: custom-reentrancy
    name: Custom Reentrancy Pattern
    severity: high
    patterns:
      - pattern: |
          function $FUNC(...) {
            ...
            $CALL(...);
            ...
            $STATE = ...;
          }
        where:
          - $CALL is external call
          - $STATE is state variable
```

### 11.2 Machine Learning Integration

- Train ML models on labeled vulnerabilities
- Use embeddings for similarity-based detection
- Anomaly detection for unusual patterns

### 11.3 Symbolic Execution Integration

- Deep integration with smartproof for full formal verification
- Bounded model checking
- Invariant inference

### 11.4 IDE Integration

- VSCode extension with real-time analysis
- LSP (Language Server Protocol) support
- Quick fixes and refactoring suggestions

### 11.5 CI/CD Integration

- GitHub Actions for PR analysis
- GitLab CI integration
- Slack/Discord notifications
- Diff-based analysis (only analyze changed code)

### 11.6 Web Interface

- Upload contracts for analysis
- Interactive reports with code highlighting
- Comparison across multiple tools
- Historical tracking of vulnerabilities

---

## 12. Comparison with Existing Tools

### SmartHunt vs Slither

| Feature | Slither | SmartHunt |
|---------|---------|-----------|
| Language | Python | Rust |
| Performance | ~1s per contract | Target: <1s per contract |
| Parallelization | Limited | Full parallel pass execution |
| IR | SlithIR | verazt IR |
| Extensibility | Python API | Rust trait + DSL |
| Output | JSON, Markdown | JSON, Markdown, SARIF |
| Detectors | 100+ | Target: 50+ (v0.1) |

**SmartHunt Advantages:**
- Better performance (Rust)
- More parallelization
- Structured pass system
- Better caching

**Slither Advantages:**
- Mature ecosystem
- More detectors
- Python ecosystem integration

### SmartHunt vs Aderyn

| Feature | Aderyn | SmartHunt |
|---------|--------|-----------|
| Language | Rust | Rust |
| AST | Custom | verazt |
| Architecture | Unknown | Pass-based |
| Extensibility | Custom detectors | Trait + DSL |
| Output | JSON, Markdown, SARIF | JSON, Markdown, SARIF |

**SmartHunt Advantages:**
- Explicit pass system
- Better parallelization architecture
- Shared IR with formal verification

### SmartHunt vs Semgrep

| Feature | Semgrep | SmartHunt |
|---------|---------|-----------|
| Language | OCaml | Rust |
| Scope | Multi-language | Solidity-specific |
| Patterns | Code-like patterns | AST patterns + semantic |
| Analysis | Local (free), Cross-file (paid) | Full semantic analysis |
| Rules | 2000+ registry | Detector registry |

**SmartHunt Advantages:**
- Deep Solidity semantic analysis
- Smart contract-specific detectors
- Free cross-contract analysis

**Semgrep Advantages:**
- Multi-language support
- Massive rule registry
- Easy pattern syntax

---

## 13. Success Metrics

### Performance Metrics
- **Analysis Time:** < 1 second per contract (local analysis)
- **Memory Usage:** < 2GB for 100+ contract projects
- **Parallel Speedup:** 4-8x with 8 cores

### Quality Metrics
- **Detector Coverage:** 50+ detectors in v0.1, 100+ in v1.0
- **True Positive Rate:** > 90% on SWC test cases
- **False Positive Rate:** < 10% on well-audited contracts
- **SWC Coverage:** 80%+ of critical SWC entries

### Usability Metrics
- **Setup Time:** < 5 minutes from install to first analysis
- **Configuration:** Works out-of-box with zero config
- **Documentation:** Complete reference for all detectors

---

## 14. Risks and Mitigations

### Risk 1: False Positives
**Impact:** Users lose trust if too many false alarms
**Mitigation:**
- Confidence levels for each finding
- Extensive testing on real contracts
- User feedback loop for refinement

### Risk 2: Performance Regression
**Impact:** Slow analysis hurts adoption
**Mitigation:**
- Continuous benchmarking in CI
- Performance budgets for each pass
- Optional expensive passes (opt-in)

### Risk 3: Complexity Creep
**Impact:** System becomes hard to maintain
**Mitigation:**
- Clear separation of concerns (passes, detectors)
- Documentation for each component
- Code review process

### Risk 4: Incompleteness
**Impact:** Missing critical vulnerabilities
**Mitigation:**
- Prioritize high-impact detectors first
- Compare against Slither/Aderyn on same corpus
- Continuous detector additions

---

## 15. Conclusion

SmartHunt will be a state-of-the-art bug detection framework for Solidity smart contracts, leveraging:

1. **Complete parsing and normalization infrastructure** from the solidity crate (soljc)
   - Compilation via solc with version negotiation
   - 15-pass normalization (inheritance, modifiers, expression flattening, etc.)
   - Optional IR generation
   - Visitor/Fold/Map patterns for traversal
2. **Bug reporting infrastructure** from the bugs crate
   - Standardized Bug struct with all necessary metadata
   - BugKind and RiskLevel enumerations
   - CWE/SWC ID tracking
3. **Pass-based analysis architecture** for modularity and efficiency
4. **Parallelization** for performance on large codebases
5. **Extensibility** through traits and DSL
6. **Rich output formats** for various use cases

By building on the solidity and bugs crates' foundation, SmartHunt can focus entirely on high-value analysis and detection work rather than reimplementing basic compilation and bug reporting infrastructure.

The phased implementation plan ensures we deliver value incrementally, starting with critical vulnerability detectors and expanding to comprehensive analysis.

**Next Steps:**
1. Review and approve this plan
2. Set up project board with milestones
3. Begin Milestone 1: Core Infrastructure
4. Establish CI/CD and testing infrastructure
5. Start weekly progress reviews

---

**Document Version:** 2.1 (Streamlined - focus on work to be done)
**Author:** SmartHunt Planning Team
**Date:** 2026-02-05
**Status:** Draft - Awaiting Review

**Changelog:**
- v2.1: Removed detailed descriptions of already-completed features (solidity/bugs crates)
- v2.0: Updated to leverage existing solidity crate (soljc) for parsing and normalization
- v1.0: Initial plan
