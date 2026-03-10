# Plan: Migrate Scanner Detectors to SIR/AIR with Proper Program Analyses

## Goal

Remove all direct Solidity AST (`frontend::solidity::ast`) dependencies from scanner
detectors. Every detector must operate on **SIR** (`mlir::sir`) or **AIR** (`mlir::air`).

More importantly: detectors must not approximate program analysis with sequential AST
walks. Instead, dedicated **analysis passes** compute precise artifacts (dominance trees,
def-use chains, write sets, taint results, value ranges) that detectors query. Detectors
become thin query layers on top of pre-computed facts.

---

## Current State

The scanner has four implementation tiers:

| Tier | Count | Representation | Status |
|------|-------|----------------|--------|
| AST-based | 5 | `frontend::solidity::ast` | **Migrate** |
| GREP-based | 11 | `frontend::solidity::ast` via PatternMatcher | **Migrate** |
| SIR structural | 6 | `mlir::sir` | Keep / upgrade |
| AIR dataflow | 3 | `mlir::air` | Keep / upgrade |

**Detectors to migrate (16 total):**

AST tier: `CeiViolation`, `DeadCode`, `Reentrancy`, `UncheckedCall`, `UninitializedStorage`

GREP tier: `CentralizationRisk`, `ConstantStateVar`, `Delegatecall`, `Deprecated`,
`FloatingPragma`, `LowLevelCall`, `MissingAccessControl`, `Shadowing`,
`TimestampDependence`, `TxOrigin`, `Visibility`

### Why sequential SIR walks are not enough

The AST-based detectors already do sequential walks — moving them to SIR without adding
real analysis just replicates the same imprecision on a cleaner tree. Two concrete
failure modes:

- **False negative (branching):** `Reentrancy` misses cases where an external call is
  inside a branch that is not always taken, but the storage write is unconditional.
- **False positive (early return):** `CeiViolation` flags storage writes after a call
  that is on an unreachable branch.

Both require CFG-based path reasoning, not sequential statement ordering.

---

## Analysis Pass Architecture

Detectors declare analysis pass dependencies. The pipeline engine runs required passes
first (already supported via `Pass::dependencies()`), then runs detectors in parallel.
Each analysis pass stores its result as a typed artifact in `AnalysisContext`.

```
SIR modules
    │
    ├─ Phase 1: WriteSetPass          → WriteSetArtifact
    │
    ▼
AIR modules (eagerly lowered from SIR, already in AnalysisContext)
    │
    ├─ Phase 2a: DominancePass        → DominanceArtifact
    ├─ Phase 2b: DefUsePass           → DefUseArtifact
    ├─ Phase 2c: ICFGPass             → ICFGArtifact  (extends existing call_graph)
    │
    ├─ Phase 3a: TaintPass            → TaintArtifact  (extend existing TaintPropagationPass)
    ├─ Phase 3b: IntervalPass         → IntervalArtifact
    │
    └─ Detectors (query artifacts, emit bugs)
```

---

## Phase 1 — SIR Dialect Extensions

These changes are prerequisites for lowering Solidity constructs that are currently
left as generic AST nodes.

### 1.1  New SIR query helpers (`crates/mlir/src/sir/utils/query.rs`)

```rust
/// Walk every `Expr` in a statement tree, bottom-up.
pub fn walk_exprs(stmts: &[Stmt], visitor: &mut dyn FnMut(&Expr));

/// Walk every `Stmt` in a tree, pre-order.
pub fn walk_stmts(stmts: &[Stmt], visitor: &mut dyn FnMut(&Stmt));

/// Walk every `Expr::FunctionCall` node.
pub fn walk_function_calls(stmts: &[Stmt], visitor: &mut dyn FnMut(&FunctionCallExpr));

/// True if the FunctionCall represents an EVM external call:
/// `.call`, `.delegatecall`, `.staticcall`, `.transfer`, `.send`.
pub fn is_evm_external_call(call: &FunctionCallExpr) -> bool;

/// True if a StorageDecl carries a `#sir.constant` or `#evm.immutable` attr.
pub fn is_constant_storage(storage: &StorageDecl) -> bool;
```

### 1.2  Add `EvmExpr::TxOrigin` (`crates/mlir/src/sir/dialect/evm.rs`)

```rust
/// `evm.tx_origin()` — `tx.origin`
TxOrigin,
```

Update irgen to lower `tx.origin` member accesses to
`Expr::Dialect(DialectExpr::Evm(EvmExpr::TxOrigin))`.

### 1.3  Add explicit low-level call dialect nodes (`crates/mlir/src/sir/dialect/evm.rs`)

```rust
/// `evm.delegatecall(target, data)` — low-level delegatecall.
Delegatecall { target: Box<Expr>, data: Box<Expr>, span: Option<Span> },

/// `evm.low_level_call(target, data, value?, gas?)` — `.call{value:…}(…)`.
LowLevelCall {
    target: Box<Expr>,
    data: Box<Expr>,
    value: Option<Box<Expr>>,
    gas: Option<Box<Expr>>,
    span: Option<Span>,
},
```

Update irgen to emit these instead of generic `FunctionCall` for call-options
expressions.

### 1.4  Store pragma version in SIR module attrs

Record `pragma solidity <version>` in `Module.attrs`:

```
#sir.pragma_solidity = "^0.8.0"
```

Add constant `sir_attrs::PRAGMA_SOLIDITY`. Needed by `FloatingPragma` detector.

---

## Phase 2 — New Analysis Passes

Each pass lives in `crates/analysis/src/passes/` and implements the `Pass` trait.
Results are stored as typed `ArtifactKey` values in `AnalysisContext`.

### 2.1  Write-Set Analysis (`WriteSetPass`)

**Location:** `crates/analysis/src/passes/sir/write_set.rs`
**Representation:** `PassRepresentation::Ir` (runs on SIR before AIR is needed)

**What it computes:**
For each `FunctionDecl` in each contract, the set of storage variable names that the
function *may write*, considering all nested statements. Interprocedural extension: also
include storage written by any internally-called function (conservative union).

```rust
pub struct WriteSetArtifact;
impl ArtifactKey for WriteSetArtifact {
    // Map from (contract_name, function_name) → Set<storage_var_name>
    type Value = HashMap<(String, String), HashSet<String>>;
}
```

**Algorithm:**
1. For each function, collect all `Stmt::Assign` / `Stmt::AugAssign` LHS expressions
   that reference a storage variable (`expr_references_storage`).
2. For each internal `FunctionCall` to a known function within the same contract,
   union in that callee's write set (fixed-point iteration for mutual recursion).

**Used by:** `MissingAccessControl`, `ConstantStateVar`, `CentralizationRisk`,
`CeiViolation`.

---

### 2.2  Dominance Analysis (`DominancePass`)

**Location:** `crates/analysis/src/passes/air/dominance.rs`
**Representation:** `PassRepresentation::Air`

**What it computes:**
For each AIR function, the dominator tree: `dom(n)` = set of CFG nodes that dominate
node `n` (every path from entry to `n` passes through them).

```rust
pub struct DominanceArtifact;
impl ArtifactKey for DominanceArtifact {
    // Map from function_name → DominatorTree
    type Value = HashMap<String, DominatorTree>;
}

pub struct DominatorTree {
    /// For each block, its immediate dominator.
    pub idom: HashMap<BlockId, BlockId>,
    /// Dominator sets (pre-computed for query efficiency).
    pub dominators: HashMap<BlockId, HashSet<BlockId>>,
}

impl DominatorTree {
    /// True if `a` dominates `b`.
    pub fn dominates(&self, a: BlockId, b: BlockId) -> bool;
    /// All nodes that `n` dominates.
    pub fn dominated_by(&self, n: BlockId) -> &HashSet<BlockId>;
}
```

**Algorithm:** Standard Cooper-Harvey-Kennedy algorithm (iterative bit-vector).

**Used by:** `MissingAccessControl`, `CeiViolation`, `UninitializedStorage`.

---

### 2.3  Def-Use Analysis (`DefUsePass`)

**Location:** `crates/analysis/src/passes/air/def_use.rs`
**Representation:** `PassRepresentation::Air`

**What it computes:**
For each SSA `OpId`, the set of `OpId`s that use its result. In SSA form this is
structural (each value has one def, potentially many uses), but making it explicit
as an artifact allows detectors to query "does this op's result have any uses?"
without re-walking the IR.

```rust
pub struct DefUseArtifact;
impl ArtifactKey for DefUseArtifact {
    // Map from OpId → set of OpIds that reference this op's result
    type Value = HashMap<OpId, HashSet<OpId>>;
}
```

**Algorithm:** Single forward pass over each function's blocks. For each `Op`, for each
operand `OpRef`, add the current `OpId` to the use set of the referenced `OpId`.

**Used by:** `UncheckedCall` (return value of call has empty use set),
`DeadCode` (local var has empty use set), `UninitializedStorage`.

---

### 2.4  Interprocedural CFG (`ICFGPass`)

**Location:** `crates/analysis/src/passes/air/icfg.rs`
**Representation:** `PassRepresentation::Air`
**Dependencies:** existing `CallGraph`

**What it computes:**
Extends the intra-procedural CFG with call/return edges between functions. Adds
`ICFGNode::ExternalCallNode` edges for calls to addresses outside the contract
(used by reentrancy).

```rust
pub struct ICFGArtifact;
impl ArtifactKey for ICFGArtifact {
    type Value = Icfg;  // existing mlir::air::cfg::ICFG, extended
}
```

**Key additions over existing partial ICFG:**
- Call edges from call-site block to callee entry block.
- Return edges from callee exit blocks back to post-call-site block.
- External call nodes tagged as re-entry points.

**Used by:** `Reentrancy` (interprocedural path from external call back to storage
write), `CentralizationRisk`.

---

### 2.5  Taint Analysis (`TaintPass` — extend existing)

**Location:** `crates/analysis/src/passes/air/taint.rs` (already exists as
`TaintPropagationPass`)
**Representation:** `PassRepresentation::Air`
**Dependencies:** `ICFGPass`

**Extend the existing pass to cover:**

| Source | Taint label |
|--------|-------------|
| `EvmExpr::TxOrigin` | `TxOrigin` |
| `EvmExpr::Timestamp` | `Timestamp` |
| `EvmExpr::MsgValue` | `MsgValue` |
| External call return value | `ExternalCallReturn` |

| Sink | Query |
|------|-------|
| Branch condition (`If`/`While` cond) | Which taint labels reach it? |
| Storage write LHS | Which labels reach it? |
| Arithmetic operand | Which labels reach it? |

```rust
pub struct TaintArtifact;
impl ArtifactKey for TaintArtifact {
    // Map from OpId → set of TaintLabel that reach this op
    type Value = HashMap<OpId, HashSet<TaintLabel>>;
}
```

**Used by:** `TxOrigin` (TxOrigin label reaches branch cond), `TimestampDependence`
(Timestamp label reaches branch cond), `UncheckedCall` (ExternalCallReturn label never
reaches a branch condition — absence of taint reaching a check).

---

### 2.6  Interval Analysis (`IntervalPass`)

**Location:** `crates/analysis/src/passes/air/interval.rs`
**Representation:** `PassRepresentation::Air`

**What it computes:**
Abstract interpretation over the integer interval lattice `[lo, hi] ∪ {⊤, ⊥}`.
For each SSA value of integer type, the range of values it can hold at its definition
point. Flow-sensitive within a function (per-block abstract state).

```rust
pub struct IntervalArtifact;
impl ArtifactKey for IntervalArtifact {
    // Map from OpId → Interval
    type Value = HashMap<OpId, Interval>;
}

pub enum Interval {
    Bottom,                  // unreachable
    Range { lo: i256, hi: i256 },
    Top,                     // unknown / unbounded
}
```

**Algorithm:** Worklist-based abstract interpretation over the CFG. Widening on back
edges (loop headers) to ensure termination.

**Used by:** `UncheckedArithmetic` — an arithmetic op only overflows if the interval of
at least one operand can reach the type boundary. Eliminates false positives from
`SirUncheckedArithmetic` which currently flags all wrapping ops regardless of value
ranges.

---

## Phase 3 — Migrate Detectors

Detectors declare their analysis dependencies via `Pass::dependencies()`. The pipeline
runs all required passes first. Each detector queries `AnalysisContext` for the
pre-computed artifacts.

### 3.1  `Reentrancy`

**Target representation:** `PassRepresentation::Air`
**Replaces:** `ReentrancyAstDetector`
**Dependencies:** `ICFGPass`, `TaintPass`, write-set info from `WriteSetArtifact`

**Query on AIR:**
1. For each function not protected by `has_reentrancy_guard`:
   a. Find all `ExternalCallNode` ops in the function's ICFG subgraph.
   b. For each external call node `C`, find all storage-write ops `W` such that `C`
      can reach `W` along some ICFG path (forward reachability from `C`).
   c. Cross-reference with alias sets: the storage slot written at `W` must be in the
      same alias group as a storage slot read before `C` (read → call → write on same
      storage = reentrancy).
2. Report `(C, W)` pairs.

**Precision gain over current:** Handles branching correctly — only paths where the
external call actually precedes the write are flagged.

**Files:** `crates/scanner/src/detectors/reentrancy.rs`

---

### 3.2  `CeiViolation`

**Target representation:** `PassRepresentation::Air`
**Dependencies:** `DominancePass`, `WriteSetPass`

**Query on AIR:**
1. For each function, collect:
   - `guard_blocks`: blocks containing an `Assert` or `Revert` op (checks).
   - `effect_blocks`: blocks containing storage-write ops (effects).
   - `interaction_blocks`: blocks containing external call ops (interactions).
2. A violation exists if any `interaction_block I` **dominates** an `effect_block E`.
   This means: on every path from the function entry to `E`, the external call at `I`
   is always executed first — a true CEI violation regardless of branching.
3. Skip functions where `has_reentrancy_guard` is true.

**Precision gain over current:** The current AST detector does a linear scan and flags
any write after a call in text order. The dominance-based check is path-sensitive.

**Files:** `crates/scanner/src/detectors/cei_violation.rs`

---

### 3.3  `DeadCode`

**Target representation:** `PassRepresentation::Air`
**Dependencies:** `ICFGPass`, `DefUsePass`

**Two sub-analyses:**

**a) Unreachable statements:**
1. For each function's CFG, find blocks not reachable from the entry block (standard
   CFG reachability, not requiring new infrastructure — the CFG already has this).
2. Map unreachable `BlockId`s back to their `Span` for reporting.

**b) Unused private/internal functions:**
1. From the call graph (ICFG artifact), find functions with no incoming call edges.
2. Exclude public/external functions.
3. Report as dead functions.

**c) Unused local variables (bonus):**
1. Query `DefUseArtifact` for `LocalVar` ops with empty use sets.

**Files:** `crates/scanner/src/detectors/dead_code.rs`

---

### 3.4  `UncheckedCall`

**Target representation:** `PassRepresentation::Air`
**Dependencies:** `DefUsePass`, `TaintPass`

**Query on AIR:**
1. Find all `Call` ops in the AIR module that represent EVM external calls (tagged via
   `CallDialectOp` with an external call marker).
2. Query `DefUseArtifact`: if the `OpId` of the call result has an empty use set →
   return value is discarded → unchecked call.
3. Alternatively: query `TaintArtifact` — if the `ExternalCallReturn` taint originating
   from this call never reaches any branch condition op → the result is never checked.

Method 3 catches subtler cases: the return value is assigned to a variable but that
variable is never used in a conditional.

**Files:** `crates/scanner/src/detectors/unchecked_call.rs`

---

### 3.5  `UninitializedStorage`

**Target representation:** `PassRepresentation::Air`
**Dependencies:** `DominancePass`, `DefUsePass`

**Query on AIR:**
1. For each storage slot (from `WriteSetArtifact`), collect all read ops and write ops
   that access it.
2. For each read op `R`, check if some write op `W` dominates `R`. If no write op
   dominates `R` → the storage slot may be read before being initialized on some path.
3. Exclude slots with a compile-time initializer (`StorageDecl.init.is_some()`).
4. Exclude value types that Solidity zero-initializes by default (`Type::I256`,
   `Type::Bool`, etc.). Flag only `Type::Map`, `Type::Array`, `TypeRef` (structs).

**Precision gain:** The current detector flags types by name without checking whether
the contract actually writes to them before any read.

**Files:** `crates/scanner/src/detectors/uninitialized.rs`

---

### 3.6  `MissingAccessControl` (upgrade `SirMissingAccessControl`)

**Target representation:** `PassRepresentation::Air`
**Replaces:** both GREP `MissingAccessControl` and `SirMissingAccessControl`
**Dependencies:** `DominancePass`, `WriteSetPass`

**Query on AIR:**
1. For each public function whose write set is non-empty:
   a. Collect all storage-write blocks `W`.
   b. Collect all guard blocks `G` (blocks containing `Assert` or an access-control
      call pattern — e.g., `require(msg.sender == owner)`).
   c. A function is safe if every `W` has at least one `G` that dominates it.
   d. If any `W` is not dominated by any `G` → missing access control on that path.

**Precision gain over current `SirMissingAccessControl`:** The current version checks
for an `Assert` anywhere before the first storage write in a linear scan, which misses
guards inside branches and is confused by complex control flow.

**Files:** `crates/scanner/src/detectors/sir_missing_access_control.rs` (rename/rewrite),
`crates/scanner/src/detectors/missing_access_control.rs` (delete)

---

### 3.7  `TxOrigin`

**Target representation:** `PassRepresentation::Air`
**Dependencies:** `TaintPass`

**Query on AIR:**
1. Query `TaintArtifact` for all branch-condition ops (`If`/`While` cond operands)
   that are tainted with the `TxOrigin` label.
2. Report each such op as an authorization via `tx.origin`.

This detects indirect flows: `address a = tx.origin; require(a == owner)` — the current
GREP detector misses this because it only pattern-matches the literal `tx.origin`
expression.

**Files:** `crates/scanner/src/detectors/tx_origin.rs`

---

### 3.8  `TimestampDependence`

**Target representation:** `PassRepresentation::Air`
**Dependencies:** `TaintPass`

**Query on AIR:**
1. Query `TaintArtifact` for all branch-condition ops tainted with the `Timestamp`
   label.
2. Also flag storage writes tainted with `Timestamp` (timestamp-dependent state).

Detects indirect flows: `uint t = block.timestamp; if (t > deadline)`.

**Files:** `crates/scanner/src/detectors/timestamp_dependence.rs`

---

### 3.9  `UncheckedArithmetic` (upgrade `SirUncheckedArithmetic`)

**Target representation:** `PassRepresentation::Air`
**Replaces:** `SirUncheckedArithmetic`
**Dependencies:** `IntervalPass`

**Query on AIR:**
1. Find all `BinOp` ops with `OverflowSemantics::Wrapping` and arithmetic operators
   (`Add`, `Sub`, `Mul`, `Pow`).
2. Query `IntervalArtifact` for the intervals of each operand.
3. Check if the interval of the result can overflow the type's bit-width:
   - For `Add`: `hi(lhs) + hi(rhs) > type_max` → possible overflow.
   - For `Sub`: `lo(lhs) - hi(rhs) < type_min` → possible underflow.
   - For `Mul`: `hi(lhs) * hi(rhs) > type_max` → possible overflow.
4. If the interval proves the operation is safe (stays within bounds), skip it.

**Precision gain:** `SirUncheckedArithmetic` currently flags `a + b` even when both
`a` and `b` are in `[0, 100]` on a `uint256`. The interval check eliminates these.

**Files:** `crates/scanner/src/detectors/unchecked_arithmetic.rs`

---

### 3.10  `CentralizationRisk`

**Target representation:** `PassRepresentation::Air`
**Dependencies:** `ICFGPass`, `WriteSetPass`

**Query on AIR:**
1. Identify the set of "privileged" functions: those with a write set that includes
   security-sensitive storage (e.g., owner address, role mappings).
2. For each privileged function, walk all call paths in the ICFG that lead to it from
   a public entry point.
3. If all paths to privileged functions go through a single caller or pass through a
   single access-control check (same `msg.sender == X` guard), flag centralization risk.

This replaces the attribute-name heuristic (matching "onlyOwner" by string) with a
structural check based on actual call-graph shape.

**Files:** `crates/scanner/src/detectors/centralization_risk.rs`

---

### 3.11  `ConstantStateVar`

**Target representation:** `PassRepresentation::Ir` (SIR write-set query)
**Dependencies:** `WriteSetPass`

**Query:**
1. For each `StorageDecl` with `init.is_some()` and `!is_constant_storage`:
2. Query `WriteSetArtifact`: union write sets across all functions in the contract.
3. If the storage variable name is not in any function's write set → effectively
   constant, flag it.

**Files:** `crates/scanner/src/detectors/constant_state_var.rs`

---

### 3.12  `Delegatecall`

**Target representation:** `PassRepresentation::Ir` (SIR structural, no analysis needed)

**Query on SIR:**
Walk expressions with `walk_exprs`. Match:
- `EvmExpr::Delegatecall { .. }` (Phase 1.3)
- Fallback: `FunctionCall` with `FieldAccess` callee field `"delegatecall"`

No analysis dependency — presence of the node is the bug.

**Files:** `crates/scanner/src/detectors/delegatecall.rs`

---

### 3.13  `Deprecated`

**Target representation:** `PassRepresentation::Ir` (SIR structural)

**Query on SIR:**
Walk expressions. Flag `FunctionCall` where callee is a `Var` or `FieldAccess` with
name in: `{ suicide, sha3, callcode, throw }`.

No analysis dependency.

**Files:** `crates/scanner/src/detectors/deprecated.rs`

---

### 3.14  `FloatingPragma`

**Target representation:** `PassRepresentation::Ir` (SIR module attrs)

**Query on SIR:**
Inspect `module.attrs` for `#sir.pragma_solidity`. If the version string uses `^` or
open-ended `>=` without an upper bound, flag it. Requires Phase 1.4.

**Files:** `crates/scanner/src/detectors/floating_pragma.rs`

---

### 3.15  `LowLevelCall`

**Target representation:** `PassRepresentation::Ir` (SIR structural)

**Query on SIR:**
Walk expressions. Flag `EvmExpr::LowLevelCall`, `EvmExpr::RawCall`, `EvmExpr::Send`,
and `FunctionCall` with callee field `"call"` or `"staticcall"`. Requires Phase 1.3.

**Files:** `crates/scanner/src/detectors/low_level_call.rs`

---

### 3.16  `Shadowing`

**Target representation:** `PassRepresentation::Ir` (SIR structural)

**Query on SIR:**
1. Collect storage var names with `storage_names(contract)`.
2. Walk each function body tracking declared local names per scope.
3. Flag `Stmt::LocalVar` whose name collides with a storage var name or a parameter name.

No analysis dependency.

**Files:** `crates/scanner/src/detectors/shadowing.rs`

---

### 3.17  `Visibility`

**Target representation:** `PassRepresentation::Ir` (SIR structural)

**Query on SIR:**
For each `FunctionDecl`, check for absence of `#sir.visibility` attr. Flag if absent,
excluding constructors and fallback/receive functions.

**Files:** `crates/scanner/src/detectors/visibility.rs`

---

## Phase 4 — Cleanup

### 4.1  Remove `frontend` dependency from scanner

After all detectors are migrated, remove `frontend` from `crates/scanner/Cargo.toml`.

### 4.2  Delete the GREP pattern engine

`engines/pattern/` exists solely for the GREP tier. Once all detectors are migrated,
delete the module. Update `engines/mod.rs`.

### 4.3  Remove `SourceUnitsArtifact` from scanner

`crates/scanner/src/artifacts.rs` can be deleted. Verify other crates (verifier) that
use `SourceUnitsArtifact` still have access through their own artifact definitions.

### 4.4  Update `DetectorId` tier comments

Reflect the new two-tier world — SIR structural and AIR analysis-based:

```rust
// ── SIR structural (local pattern matching, no analysis required) ─
Delegatecall, Deprecated, FloatingPragma, LowLevelCall,
Shadowing, Visibility, ConstantStateVar,

// ── AIR analysis-based (require pre-computed analysis artifacts) ──
Reentrancy, CeiViolation, DeadCode, UncheckedCall,
UninitializedStorage, TxOrigin, TimestampDependence,
CentralizationRisk, MissingAccessControl, UncheckedArithmetic,
```

Remove `DetectorId::MissingAccessControl` (merged into upgraded
`SirMissingAccessControl`).

### 4.5  Update `pipeline/registry.rs`

Group detectors by their dependency tier so the engine can schedule them correctly.
Analysis passes must be registered and run before their dependent detectors.

---

## Summary

### New analysis passes to build

| Pass | Crate | Depends on | Artifact |
|------|-------|------------|---------|
| `WriteSetPass` | `analysis` | SIR | `WriteSetArtifact` |
| `DominancePass` | `analysis` | AIR CFG | `DominanceArtifact` |
| `DefUsePass` | `analysis` | AIR SSA | `DefUseArtifact` |
| `ICFGPass` | `analysis` | AIR + CallGraph | `ICFGArtifact` |
| `TaintPass` (extend) | `analysis` | AIR + ICFG | `TaintArtifact` |
| `IntervalPass` | `analysis` | AIR CFG | `IntervalArtifact` |

### Detector representation after migration

| Detector | New representation | Key analyses used |
|----------|--------------------|-------------------|
| `Reentrancy` | AIR | ICFG, TaintPass, alias sets |
| `CeiViolation` | AIR | DominancePass, WriteSetPass |
| `DeadCode` | AIR | ICFG (call graph), DefUsePass |
| `UncheckedCall` | AIR | DefUsePass, TaintPass |
| `UninitializedStorage` | AIR | DominancePass, DefUsePass |
| `MissingAccessControl` | AIR | DominancePass, WriteSetPass |
| `TxOrigin` | AIR | TaintPass |
| `TimestampDependence` | AIR | TaintPass |
| `UncheckedArithmetic` | AIR | IntervalPass |
| `CentralizationRisk` | AIR | ICFG, WriteSetPass |
| `ConstantStateVar` | SIR | WriteSetPass |
| `Delegatecall` | SIR | — |
| `Deprecated` | SIR | — |
| `FloatingPragma` | SIR | — |
| `LowLevelCall` | SIR | — |
| `Shadowing` | SIR | — |
| `Visibility` | SIR | — |

### File changes

| File | Action |
|------|--------|
| `crates/mlir/src/sir/utils/query.rs` | Add walker helpers |
| `crates/mlir/src/sir/dialect/evm.rs` | Add `TxOrigin`, `Delegatecall`, `LowLevelCall` |
| `crates/mlir/src/sir/attrs.rs` | Add `PRAGMA_SOLIDITY` |
| `crates/frontend/src/solidity/irgen/` | Lower new SIR nodes |
| `crates/analysis/src/passes/sir/write_set.rs` | **New** `WriteSetPass` |
| `crates/analysis/src/passes/air/dominance.rs` | **New** `DominancePass` |
| `crates/analysis/src/passes/air/def_use.rs` | **New** `DefUsePass` |
| `crates/analysis/src/passes/air/icfg.rs` | **New** `ICFGPass` |
| `crates/analysis/src/passes/air/taint.rs` | Extend `TaintPropagationPass` |
| `crates/analysis/src/passes/air/interval.rs` | **New** `IntervalPass` |
| `crates/scanner/src/detectors/reentrancy.rs` | Replace AST detector; upgrade AIR detector |
| `crates/scanner/src/detectors/cei_violation.rs` | Rewrite with dominance |
| `crates/scanner/src/detectors/dead_code.rs` | Rewrite with ICFG + def-use |
| `crates/scanner/src/detectors/unchecked_call.rs` | Rewrite with def-use + taint |
| `crates/scanner/src/detectors/uninitialized.rs` | Rewrite with dominance + def-use |
| `crates/scanner/src/detectors/sir_missing_access_control.rs` | Upgrade with dominance |
| `crates/scanner/src/detectors/missing_access_control.rs` | **Delete** |
| `crates/scanner/src/detectors/tx_origin.rs` | Rewrite with taint |
| `crates/scanner/src/detectors/timestamp_dependence.rs` | Rewrite with taint |
| `crates/scanner/src/detectors/unchecked_arithmetic.rs` | Upgrade with interval |
| `crates/scanner/src/detectors/centralization_risk.rs` | Rewrite with ICFG + write-set |
| `crates/scanner/src/detectors/constant_state_var.rs` | Rewrite with write-set |
| `crates/scanner/src/detectors/delegatecall.rs` | Rewrite as SIR walk |
| `crates/scanner/src/detectors/deprecated.rs` | Rewrite as SIR walk |
| `crates/scanner/src/detectors/floating_pragma.rs` | Rewrite as module attr check |
| `crates/scanner/src/detectors/low_level_call.rs` | Rewrite as SIR walk |
| `crates/scanner/src/detectors/shadowing.rs` | Rewrite as SIR walk |
| `crates/scanner/src/detectors/visibility.rs` | Rewrite as SIR walk |
| `crates/scanner/src/detector/id.rs` | Update tier comments, remove `MissingAccessControl` |
| `crates/scanner/src/pipeline/registry.rs` | Update registrations |
| `crates/scanner/src/artifacts.rs` | Delete |
| `crates/scanner/src/engines/pattern/` | **Delete** |
| `crates/scanner/Cargo.toml` | Remove `frontend` dependency |

---

## Constraints

- **No new `frontend::solidity::ast` imports** in any scanner file.
- **Analysis passes are stateless:** they read from `AnalysisContext`, compute a result,
  and store it as an artifact. They must not cache mutable state between runs.
- **Detectors are query-only:** a detector must not re-implement analysis logic. If
  the required analysis is missing, add a new pass — not inline logic in the detector.
- **Cross-chain correctness:** SIR core detectors (`Shadowing`, `DeadCode`,
  `UninitializedStorage`) must not match on EVM dialect attrs. Only EVM-specific
  detectors (`TxOrigin`, `Delegatecall`, `LowLevelCall`) use `namespace == "evm"`.
- **Span reporting:** prefer `FunctionDecl.span` or `ContractDecl.span` when an
  individual op or statement has no span.
- **Test coverage:** each analysis pass needs unit tests with a minimal AIR/SIR module
  constructed in Rust. Detectors need at least one positive and one negative test.
