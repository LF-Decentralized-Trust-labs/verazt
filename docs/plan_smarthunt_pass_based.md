# SmartHunt Pass-Based Architecture Redesign

## 1. Current State: What's Wrong

### 1.1 The Execution Flow Is Disconnected

The `main.rs` **bypasses the entire pass infrastructure** and runs detectors directly:

```
main.rs (line 395-448):
  registry = DetectorRegistry::new()
  register_all_detectors(&registry)
  for detector in enabled_detectors:
      detector.detect(&context)    // ← Directly calls detectors, no PassManager involved
```

The `PassManager`, `PassScheduler`, `PassExecutor`, and `DetectionManager` are **never used** in the actual execution path. They are dead infrastructure.

### 1.2 Detectors Don't Use Analysis Passes

Every single detector declares **empty dependencies**:

| Module | Detector | `dependencies()` |
|--------|----------|-------------------|
| dfa | ReentrancyDfaDetector | `vec![]` |
| dfa | CeiViolationDfaDetector | `vec![]` |
| dfa | DeadCodeDfaDetector | `vec![]` |
| dfa | UncheckedCallDfaDetector | `vec![]` |
| dfa | UninitializedDfaDetector | `vec![]` |
| grep | TxOriginGrepDetector | `vec![]` |
| grep | All 11 grep detectors | `vec![]` |

Meanwhile, the analysis passes (SymbolTable, TypeIndex, CallGraph, InheritanceGraph, ModifierAnalysis, CFG) **exist and work** but are never invoked because no detector requests them.

### 1.3 Naming Problem

`detection` is ambiguous - it contains both:
- The orchestration logic (manager, registry)
- The detector trait definition (pass.rs)

It doesn't clearly express its role as the **entry point and pipeline coordinator**.

---

## 2. Proposed Architecture

### 2.1 Rename `detection` → `pipeline`

The `pipeline` module becomes the **main entry point** that coordinates the full execution flow. The name communicates that this is the orchestration layer, not the detection itself.

| Current | Proposed | Role |
|---------|----------|------|
| `detection/` | `pipeline/` | Main entry point and orchestration |
| `detection/manager.rs` | `pipeline/engine.rs` | Pipeline engine (the orchestrator) |
| `detection/pass.rs` | `pipeline/detector.rs` | BugDetectionPass trait |
| `detection/registry.rs` | `pipeline/registry.rs` | Detector registration |
| `analysis/` | `analysis/` | Unchanged - collects AST/IR info |
| `dfa/` | `dfa/` | Unchanged - IR analysis framework + detectors |
| `grep/` | `grep/` | Unchanged - AST pattern matching + detectors |

### 2.2 Two-Phase Pipeline

The core design is a **two-phase pipeline** driven by the CLI's `--enable`/`--disable` flags:

```
┌─────────────────────────────────────────────────────────────┐
│                        CLI (main.rs)                        │
│  --enable reentrancy,tx-origin  --parallel  --format json   │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                   PIPELINE ENGINE                           │
│                   (pipeline/engine.rs)                       │
│                                                             │
│  1. Resolve enabled detectors from CLI flags                │
│  2. Collect required analysis passes from detector deps     │
│  3. Build dependency graph → topological sort → levels      │
│  4. Execute Phase 1 and Phase 2                             │
└──────────────────────────┬──────────────────────────────────┘
                           │
              ┌────────────┴────────────┐
              ▼                         ▼
┌──────────────────────┐   ┌──────────────────────────────────┐
│  PHASE 1: ANALYSIS   │   │  PHASE 2: DETECTION              │
│  (analysis/)         │   │  (dfa/detectors + grep/detectors) │
│                      │   │                                    │
│  Run required passes │──▶│  Run enabled detectors             │
│  in parallel by      │   │  in parallel (all read-only       │
│  dependency levels   │   │  access to AnalysisContext)        │
└──────────────────────┘   └──────────────────────────────────┘
```

### 2.3 Phase 1: Analysis Passes (Parallel by Dependency Level)

The `DependencyGraph.compute_levels()` already computes parallelization levels. This is the execution model:

```
Level 0: [SymbolTable, TypeIndex]           ← parallel (no deps)
Level 1: [CallGraph, InheritanceGraph,      ← parallel (all depend only on Level 0)
           ModifierAnalysis]
Level 2: [CfgPass]                          ← parallel (depends on Level 1)
```

Only the passes **actually required** by the enabled detectors are scheduled. If the user enables only grep-based detectors that need SymbolTable, then only SymbolTable runs.

```rust
/// Phase 1: Run analysis passes
fn run_analysis_phase(
    &self,
    enabled_detectors: &[&dyn BugDetectionPass],
    context: &mut AnalysisContext,
) -> PassResult<()> {
    // 1. Collect required passes from detector dependencies
    let required: HashSet<PassId> = enabled_detectors
        .iter()
        .flat_map(|d| d.dependencies())
        .collect();

    // 2. Register only required passes with the PassManager
    let mut pass_manager = PassManager::new(PassManagerConfig {
        enable_parallel: self.config.parallel,
        ..Default::default()
    });
    for pass_id in &required {
        if let Some(pass) = self.create_analysis_pass(*pass_id) {
            pass_manager.register_analysis_pass(pass);
        }
    }

    // 3. Scheduler computes dependency levels → executor runs them
    //    Level 0 passes run in parallel, then Level 1, etc.
    pass_manager.run(context)?;
    Ok(())
}
```

### 2.4 Phase 2: Detection (Fully Parallel)

After Phase 1, the `AnalysisContext` contains all computed artifacts. All detectors **read from** the context and **produce** `Vec<Bug>`. No detector modifies the context. This makes them **embarrassingly parallel**.

```rust
/// Phase 2: Run detectors
fn run_detection_phase(
    &self,
    enabled_detectors: &[&dyn BugDetectionPass],
    context: &AnalysisContext,  // ← immutable reference, safe for parallel access
) -> Vec<Bug> {
    use rayon::prelude::*;

    if self.config.parallel && enabled_detectors.len() > 1 {
        enabled_detectors
            .par_iter()
            .flat_map(|detector| {
                detector.detect(context).unwrap_or_default()
            })
            .collect()
    } else {
        enabled_detectors
            .iter()
            .flat_map(|detector| {
                detector.detect(context).unwrap_or_default()
            })
            .collect()
    }
}
```

### 2.5 Pipeline Engine (Full Design)

```rust
/// The main pipeline engine that orchestrates analysis and detection.
///
/// Execution flow:
///   CLI flags → resolve detectors → collect analysis deps
///   → Phase 1: run analysis passes (parallel by dependency level)
///   → Phase 2: run detectors (fully parallel)
///   → collect bugs → output
pub struct PipelineEngine {
    /// Detector registry
    registry: DetectorRegistry,
    /// Pipeline configuration
    config: PipelineConfig,
}

pub struct PipelineConfig {
    /// Enable parallel execution
    pub parallel: bool,
    /// Number of worker threads (0 = auto-detect)
    pub num_threads: usize,
    /// Enabled detector IDs (empty = all)
    pub enabled: Vec<String>,
    /// Disabled detector IDs
    pub disabled: Vec<String>,
}

pub struct PipelineResult {
    /// All detected bugs
    pub bugs: Vec<Bug>,
    /// Timing: analysis phase
    pub analysis_duration: Duration,
    /// Timing: detection phase
    pub detection_duration: Duration,
    /// Timing: total
    pub total_duration: Duration,
    /// Per-detector stats
    pub detector_stats: Vec<DetectorStats>,
}

impl PipelineEngine {
    pub fn new(config: PipelineConfig) -> Self {
        let mut registry = DetectorRegistry::new();
        register_all_detectors(&mut registry);
        Self { registry, config }
    }

    /// Main entry point: run the full pipeline
    pub fn run(&self, context: &mut AnalysisContext) -> PipelineResult {
        let start = Instant::now();

        // Step 1: Resolve which detectors to run
        let enabled_detectors = self.resolve_detectors();

        // Step 2: Phase 1 - Analysis
        let analysis_start = Instant::now();
        self.run_analysis_phase(&enabled_detectors, context);
        let analysis_duration = analysis_start.elapsed();

        // Step 3: Phase 2 - Detection (parallel)
        let detection_start = Instant::now();
        let (bugs, stats) = self.run_detection_phase(&enabled_detectors, context);
        let detection_duration = detection_start.elapsed();

        PipelineResult {
            bugs,
            analysis_duration,
            detection_duration,
            total_duration: start.elapsed(),
            detector_stats: stats,
        }
    }
}
```

### 2.6 Simplified `main.rs`

```rust
fn run_analysis(args: Arguments) {
    // Parse and compile source files...
    let context = AnalysisContext::new(all_source_units, AnalysisConfig::default());

    // Create and run the pipeline
    let engine = PipelineEngine::new(PipelineConfig {
        parallel: args.parallel,
        num_threads: config.num_threads,
        enabled: config.detectors.enabled,
        disabled: config.detectors.disabled,
    });

    let result = engine.run(&mut context);

    // Format and output
    let report = AnalysisReport::new(result.bugs, files_analyzed, result.total_duration);
    // ...
}
```

---

## 3. Wiring Detectors to Analysis Passes

The critical missing piece: detectors need to declare their real dependencies so Phase 1 knows what to run.

### 3.1 GREP Detectors (AST-based)

Most GREP detectors are self-contained pattern matchers that traverse the AST directly. However, some can benefit from analysis data:

| Detector | Should Depend On | Why |
|----------|-----------------|-----|
| `missing_access_control` | `SymbolTable`, `ModifierAnalysis` | Needs to know modifiers and function visibility |
| `shadowing` | `SymbolTable`, `InheritanceGraph` | Needs inherited symbols to detect shadowing |
| `centralization_risk` | `SymbolTable`, `ModifierAnalysis` | Needs owner/admin pattern recognition |
| `constant_state_var` | `SymbolTable` | Needs state variable declarations |
| `visibility` | `SymbolTable` | Needs function visibility info |
| `tx_origin` | (none) | Pure pattern match |
| `floating_pragma` | (none) | Pure pattern match |
| `deprecated` | (none) | Pure pattern match |
| `low_level_call` | (none) | Pure pattern match |
| `timestamp_dependence` | (none) | Pure pattern match |
| `delegatecall` | (none) | Pure pattern match |

### 3.2 DFA Detectors (IR-based)

DFA detectors need CFG and data flow information:

| Detector | Should Depend On | Why |
|----------|-----------------|-----|
| `reentrancy` | `SymbolTable`, `CallGraph`, `ModifierAnalysis` | Needs call graph for external calls, modifiers for nonReentrant check |
| `cei_violation` | `SymbolTable`, `CallGraph` | Needs call sequence analysis |
| `dead_code` | `SymbolTable`, `CallGraph` | Needs reachability from call graph |
| `unchecked_call` | `SymbolTable` | Needs to identify external call returns |
| `uninitialized` | `SymbolTable` | Needs state variable declarations |

### 3.3 Dependency Graph Visualization

```
                 ┌─ Level 0 ─────────────────────────┐
                 │  SymbolTable    TypeIndex          │  ← No deps, run in parallel
                 └─────────┬────────────┬─────────────┘
                           │            │
                 ┌─ Level 1 ┼────────────┼────────────┐
                 │  CallGraph  InheritanceGraph        │  ← Depend on SymbolTable
                 │  ModifierAnalysis                   │     Run in parallel
                 └─────────┬────────────┬─────────────┘
                           │            │
                 ┌─ Level 2 ┼────────────┼────────────┐
                 │  CfgPass (if IR needed)             │  ← Depend on Level 1
                 └─────────────────────────────────────┘
                           │
                           ▼
    ┌──────────────────────────────────────────────────────┐
    │              ALL DETECTORS (parallel)                 │
    │                                                      │
    │  GREP:                    DFA:                       │
    │  ├─ tx_origin             ├─ reentrancy              │
    │  ├─ floating_pragma       ├─ cei_violation            │
    │  ├─ deprecated            ├─ dead_code               │
    │  ├─ low_level_call        ├─ unchecked_call           │
    │  ├─ timestamp_dependence  └─ uninitialized           │
    │  ├─ delegatecall                                     │
    │  ├─ visibility                                       │
    │  ├─ shadowing                                        │
    │  ├─ missing_access_control                           │
    │  ├─ centralization_risk                              │
    │  └─ constant_state_var                               │
    └──────────────────────────────────────────────────────┘
```

---

## 4. Module Structure After Redesign

```
smarthunt/src/
├── main.rs                         # CLI entry point → calls PipelineEngine
├── config.rs                       # CLI configuration
├── lib.rs                          # Re-exports
│
├── pipeline/                       # RENAMED from detection/
│   ├── mod.rs                      # Module definition
│   ├── engine.rs                   # PipelineEngine (main orchestrator)
│   ├── detector.rs                 # BugDetectionPass trait (renamed from pass.rs)
│   └── registry.rs                 # DetectorRegistry + register_all_detectors
│
├── analysis/                       # UNCHANGED - collects AST/IR information
│   ├── mod.rs
│   ├── context.rs                  # AnalysisContext (central artifact storage)
│   ├── pass.rs                     # Pass, AnalysisPass traits
│   ├── pass_id.rs                  # PassId enum
│   ├── pass_level.rs               # PassLevel enum
│   ├── pass_representation.rs      # PassRepresentation enum
│   ├── dependency.rs               # DependencyGraph
│   ├── manager.rs                  # PassManager (analysis phase orchestrator)
│   ├── scheduler.rs                # PassScheduler
│   ├── executor.rs                 # PassExecutor
│   ├── ast/                        # AST analysis passes
│   │   ├── symbol_table.rs         # SymbolTablePass
│   │   ├── type_index.rs           # TypeIndexPass
│   │   ├── call_graph.rs           # CallGraphPass
│   │   ├── inheritance_graph.rs    # InheritanceGraphPass
│   │   └── modifier_analysis.rs    # ModifierAnalysisPass
│   └── ir/                         # IR analysis passes
│       └── cfg.rs                  # CfgPass
│
├── dfa/                            # UNCHANGED - IR data flow framework
│   ├── mod.rs
│   ├── cfg.rs, lattice.rs, solver.rs, var.rs, utils.rs
│   ├── analyses/                   # Reusable DFA analyses
│   │   ├── reaching_defs.rs, liveness.rs, taint.rs
│   │   ├── def_use.rs, state_mutation.rs
│   └── detectors/                  # DFA-based bug detectors
│       ├── reentrancy.rs           # deps: [SymbolTable, CallGraph, ModifierAnalysis]
│       ├── cei_violation.rs        # deps: [SymbolTable, CallGraph]
│       ├── dead_code.rs            # deps: [SymbolTable, CallGraph]
│       ├── unchecked_call.rs       # deps: [SymbolTable]
│       └── uninitialized.rs        # deps: [SymbolTable]
│
├── grep/                           # UNCHANGED - AST pattern matching framework
│   ├── mod.rs
│   ├── core.rs, builder.rs, matcher.rs, primitives.rs, composite.rs
│   └── detectors/                  # GREP-based bug detectors
│       ├── tx_origin.rs            # deps: [] (pure pattern match)
│       ├── floating_pragma.rs      # deps: []
│       ├── deprecated.rs           # deps: []
│       ├── shadowing.rs            # deps: [SymbolTable, InheritanceGraph]
│       ├── visibility.rs           # deps: [SymbolTable]
│       └── ...
│
└── output/                         # UNCHANGED - output formatters
    ├── formatter.rs, json.rs, markdown.rs, sarif.rs
```

---

## 5. What Changes, What Doesn't

### 5.1 Changes Required

| Change | Files | Effort |
|--------|-------|--------|
| Rename `detection/` → `pipeline/` | 4 files | Low |
| Rename `detection/pass.rs` → `pipeline/detector.rs` | 1 file + imports | Low |
| Rename `detection/manager.rs` → `pipeline/engine.rs` | 1 file + imports | Low |
| Rewrite `PipelineEngine.run()` with two-phase flow | `engine.rs` | Medium |
| Wire detectors to analysis passes (update `dependencies()`) | ~16 detector files | Medium |
| Simplify `main.rs` to call `PipelineEngine` | `main.rs` | Low |
| Update `lib.rs` re-exports | `lib.rs` | Low |

### 5.2 What Stays the Same

- `analysis/` module - unchanged, already well-designed
- `dfa/` framework (cfg, lattice, solver, analyses) - unchanged
- `grep/` framework (core, builder, matcher, primitives) - unchanged
- All detector logic inside `detect()` methods - unchanged
- `PassManager`, `PassScheduler`, `PassExecutor` - unchanged, finally used
- `DependencyGraph` - unchanged, already has `compute_levels()` for parallelism
- `AnalysisContext` - unchanged
- Output formatters - unchanged

### 5.3 What Gets Deleted

| Item | Reason |
|------|--------|
| `PassLevel` enum | Detectors don't need to declare granularity level (see Section 6) |
| `PassRepresentation` enum | Unnecessary - the pass itself knows what it operates on |
| `analysis/executor.rs:execute_level_parallel()` | Dead code, replace with proper parallel execution |
| `detection/manager.rs` parallel detector logic | Replaced by `PipelineEngine` |

---

## 6. Design Decision: Drop PassLevel from Detectors

### Why PassLevel Is Unnecessary for Detectors

In LLVM, `PassLevel` (Module/Function/Loop/BasicBlock) serves two purposes:
1. **Automatic iteration**: The pass manager iterates at the right granularity
2. **Parallelization**: Items at the same level can be processed in parallel

For SmartHunt, **neither purpose applies to detectors**:
- GREP detectors use `PatternMatcher` which does its own efficient single-pass AST traversal
- DFA detectors build their own CFGs and run their own solvers per-function internally
- Forcing detectors into a level-based iteration model would **fight against** their natural execution patterns

### Where PassLevel IS Useful

PassLevel should remain on **analysis passes** (SymbolTable, CallGraph, etc.) because:
- Analysis passes are composable building blocks
- Their level describes what they compute (program-wide vs per-function)
- Future analysis passes might benefit from level-based iteration

### What Detectors Need Instead

Detectors only need:
1. `dependencies() -> Vec<PassId>` — which analysis passes must run first
2. `detect(&AnalysisContext) -> Vec<Bug>` — the actual detection logic

The parallelization happens at the **detector level** (all detectors run in parallel), not at the granularity level within a detector. This is the right trade-off because:
- Detectors are already coarse-grained (each takes ~1-50ms)
- There are 16+ detectors → plenty of parallelism
- Internal parallelization within a detector adds complexity with diminishing returns

---

## 7. Implementation Plan

### Step 1: Rename `detection/` → `pipeline/`

```
git mv smarthunt/src/detection/ smarthunt/src/pipeline/
```

Rename files:
- `detection/pass.rs` → `pipeline/detector.rs`
- `detection/manager.rs` → `pipeline/engine.rs`
- `detection/registry.rs` → `pipeline/registry.rs`

Update all imports across the codebase.

### Step 2: Rewrite PipelineEngine

Replace `DetectionManager` with `PipelineEngine` implementing the two-phase design from Section 2.5. Key changes:
- `run()` method implements Phase 1 → Phase 2 flow
- Phase 1 uses `PassManager` to run analysis passes
- Phase 2 uses rayon for parallel detector execution
- `PipelineConfig` takes detector enable/disable from CLI

### Step 3: Wire Detector Dependencies

Update each detector's `dependencies()` method according to the table in Section 3. Example:

```rust
// Before (reentrancy.rs)
fn dependencies(&self) -> Vec<PassId> {
    vec![]  // ← Wrong: ignores available analysis
}

// After
fn dependencies(&self) -> Vec<PassId> {
    vec![PassId::SymbolTable, PassId::CallGraph, PassId::ModifierAnalysis]
}
```

Then update `detect()` to use analysis artifacts from context:

```rust
fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
    // Use pre-computed analysis data instead of re-computing
    let symbol_table = context.get_artifact::<SymbolTable>("symbol-table");
    let call_graph = context.get_artifact::<CallGraph>("call-graph");
    let modifiers = context.get_artifact::<ModifierAnalysis>("modifier-analysis");
    // ... detection logic using these artifacts
}
```

### Step 4: Simplify main.rs

Replace the manual detector loop with:

```rust
let engine = PipelineEngine::new(PipelineConfig::from(&config));
let result = engine.run(&mut context);
```

### Step 5: Clean Up Dead Code

- Remove `PassLevel` from detector `Pass` impl (keep in analysis passes)
- Remove `PassRepresentation` from detector `Pass` impl
- Remove unused `execute_level_parallel()` function
- Remove the `DetectionManager` type entirely

---

## 8. Parallel Execution Summary

### What Runs in Parallel

```
Phase 1 - Analysis (parallel within dependency levels):
  ┌───────────────────────────────────────┐
  │ Level 0: SymbolTable ∥ TypeIndex      │  2 passes in parallel
  │ Level 1: CallGraph ∥ InheritanceGraph │  3 passes in parallel
  │          ∥ ModifierAnalysis           │
  │ Level 2: CfgPass                      │  1 pass (if IR needed)
  └───────────────────────────────────────┘

Phase 2 - Detection (all detectors in parallel):
  ┌───────────────────────────────────────┐
  │ reentrancy ∥ cei_violation ∥ ...      │
  │ tx_origin ∥ shadowing ∥ ...           │  16 detectors in parallel
  │ dead_code ∥ visibility ∥ ...          │
  └───────────────────────────────────────┘
```

### Why This Is Optimal

1. **Minimal analysis work**: Only passes required by enabled detectors are scheduled
2. **Maximum analysis parallelism**: `DependencyGraph.compute_levels()` already identifies independent passes at each level
3. **Maximum detection parallelism**: All detectors read from immutable `&AnalysisContext`, fully safe to parallelize
4. **No over-engineering**: No complex sub-function parallelization, no context cloning, no scoped contexts
5. **Simple mental model**: Two phases, each with clear parallelization boundaries

---

## 9. Key Files Reference

### Pipeline (New)
- `pipeline/engine.rs` — PipelineEngine (main orchestrator)
- `pipeline/detector.rs` — BugDetectionPass trait
- `pipeline/registry.rs` — DetectorRegistry + register_all_detectors

### Analysis (Unchanged)
- `analysis/manager.rs` — PassManager (Phase 1 orchestrator)
- `analysis/dependency.rs` — DependencyGraph with `compute_levels()`
- `analysis/context.rs` — AnalysisContext (shared artifact storage)
- `analysis/ast/*.rs` — AST analysis passes (SymbolTable, CallGraph, etc.)
- `analysis/ir/*.rs` — IR analysis passes (CfgPass)

### Detectors (Dependencies Updated)
- `dfa/detectors/*.rs` — 5 DFA-based detectors
- `grep/detectors/*.rs` — 11 GREP-based detectors
