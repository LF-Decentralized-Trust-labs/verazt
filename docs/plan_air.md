# Plan: Anir (ANIR) — Implementation Plan

**Scope:** Define the `anir` crate (Anir data structures) and implement Pass 2a
(the lowering from ContractIR/SCIR into ANIR).

---

## 1. Context

The SCIR pipeline has three layers:

```
Source AST
    │  Pass 1 (per-language frontend)
    ▼
ContractIR (SCIR)   ← already implemented in crates/scir
    │  Pass 2a     ← THIS PLAN
    ▼
Anir (ANIR)   ← to be defined
    │  Pass 3a
    ▼
Backend (Datalog / SMT / graph tools)
```

ANIR is a graph-structured, SSA-form IR optimised for static dataflow analysis.
Its key invariant: **every dialect construct that reaches ANIR implements at least
one of** `StorageOp`, `CallOp`, `TaintSource`, or `TaintSink`.

Pass 2a is a five-step transformation (§4.2 of the design doc):

| Step | Name | Chain-specific? |
|------|------|----------------|
| 1 | evm Modifier Expansion | EVM only |
| 2 | CFG Construction | No |
| 3 | SSA Renaming | No |
| 4 | Dialect Lowering | Per-dialect |
| 5 | ICFG + Alias + Taint init | No |

---

## 2. New Crate: `crates/anir`

Create `crates/anir` as a library crate (no binary). It depends on `crates/scir`
for shared types (e.g. `Type`, `Attr`, `Span`, dialect types used in lowered ops).

### 2.1 Module Layout

```
crates/anir/
  Cargo.toml
  src/
    lib.rs           — pub re-exports
    module.rs        — AnirModule (top-level ANIR container)
    cfg.rs           — BasicBlock, CFG, ICFGNode, ICFG
    ops.rs           — Op (SSA value node), OpKind
    interfaces.rs    — StorageOp, CallOp, TaintSource, TaintSink traits
    alias.rs         — AliasMap, AliasGroup, StorageRef
    taint.rs         — TaintGraph, TaintLabel, TaintSeed
    summary.rs       — FunctionSummary
    call_graph.rs    — CallGraph (static + dynamic edges)
    pdg.rs           — PDG (program dependence graph)
    lower/
      mod.rs         — Pass2a entry point: `lower_module(Module) -> AnirModule`
      step1_modifier_expand.rs   — EVM modifier inline expansion
      step2_cfg.rs               — structured-stmt → basic-block CFG
      step3_ssa.rs               — SSA renaming (Cytron pruned SSA)
      step4_dialect_lower.rs     — per-dialect SCIR → ANIR op lowering
      step5_icfg.rs              — ICFG + alias set + taint graph init
```

---

## 3. ANIR Data Structures (`src/`)

### 3.1 `AnirModule` (`module.rs`)

```rust
pub struct AnirModule {
    pub source_module_id: String,
    pub icfg:        ICFG,
    pub call_graph:  CallGraph,
    pub alias_sets:  AliasMap,
    pub pdg:         PDG,
    pub taint_graph: TaintGraph,
    pub summaries:   Vec<FunctionSummary>,
}
```

### 3.2 CFG and ICFG (`cfg.rs`)

**Basic block:**
```rust
pub struct BasicBlock {
    pub id:    BlockId,
    pub ops:   Vec<Op>,
    pub term:  Terminator,
}

pub enum Terminator {
    Branch { cond: OpRef, then_bb: BlockId, else_bb: BlockId },
    Jump(BlockId),
    TxnExit { reverted: bool },
    Unreachable,
}
```

**ICFG node types** (one node per SSA op + special boundary nodes):
```rust
pub enum ICFGNode {
    TxnEntry    { func: FunctionId },
    TxnExit     { func: FunctionId, reverted: bool },
    CallSite    { op: OpId },          // implements CallOp
    ReturnSite  { op: OpId },
    ExternalCallNode { op: OpId },     // #scir.call_risk.reentrancy = true
    ReentryPoint     { func: FunctionId },
    StmtNode    { op: OpId },
}
```

**ICFG:**
```rust
pub struct ICFG {
    pub nodes: Vec<ICFGNode>,
    pub edges: Vec<(ICFGNodeId, ICFGNodeId, EdgeKind)>,
}
pub enum EdgeKind { CfgEdge, CallEdge, ReturnEdge, ReentryEdge }
```

### 3.3 SSA Ops (`ops.rs`)

Each SCIR statement/expression lowers to one or more `Op` values:
```rust
pub struct Op {
    pub id:      OpId,
    pub kind:    OpKind,
    pub result:  Option<(SsaName, Type)>,
    pub attrs:   Vec<Attr>,           // carries #scir.taint, #scir.overflow, etc.
    pub span:    Option<Span>,
}

pub enum OpKind {
    // Core
    Const(Lit),
    BinOp { op: BinOp, lhs: OpRef, rhs: OpRef, overflow: OverflowSemantics },
    UnOp  { op: UnOp, operand: OpRef },
    Phi(Vec<(BlockId, OpRef)>),       // block-argument style
    Assert { cond: OpRef },
    Return(Vec<OpRef>),
    // Dialect ops (lowered; implement one of the four interfaces)
    Storage(Box<dyn StorageOp>),
    Call(Box<dyn CallOp>),
    TaintSrc(Box<dyn TaintSource>),
    TaintSink(Box<dyn TaintSink>),
    // SSA pseudo-values (no runtime cost)
    PseudoValue { label: String, taint: TaintLabel },
}
```

**Note on `Box<dyn Trait>`:** Since ANIR ops must be cloneable and comparable for
tests, use a concrete enum `DialectOp` (with variants per dialect) rather than
`Box<dyn Trait>` for the initial implementation. The trait interface is used
for analysis passes to query ops without branching on dialect.

### 3.4 Interface Traits (`interfaces.rs`)

```rust
/// A storage read or write operation.
pub trait StorageOp {
    fn to_storage_ref(&self) -> StorageRef;
    fn is_write(&self) -> bool;
    fn alias_group_id(&self) -> AliasGroupId;
}

/// An internal or external call operation.
pub trait CallOp {
    fn callee(&self) -> CallTarget;
    fn call_risk(&self) -> CallRisk;
}

/// A value that seeds the taint graph.
pub trait TaintSource {
    fn taint_label(&self) -> TaintLabel;
}

/// A value that is a taint sink (analysed for tainted inputs).
pub trait TaintSink {
    fn sink_category(&self) -> SinkCategory;
}
```

### 3.5 Alias Analysis (`alias.rs`)

```rust
pub struct AliasMap {
    pub groups: HashMap<AliasGroupId, AliasGroup>,
}

pub struct AliasGroup {
    pub id:     AliasGroupId,
    pub reads:  Vec<(OpId, Option<OpRef>)>,   // (op, key operand)
    pub writes: Vec<(OpId, Option<OpRef>)>,
}

pub struct StorageRef {
    pub base:    String,           // e.g. "balances", "Balance<CoinType>"
    pub indices: Vec<StorageIndex>,
}

pub enum StorageIndex {
    Wildcard,
    Key(OpRef),
}
```

### 3.6 Taint Graph (`taint.rs`)

```rust
pub struct TaintGraph {
    pub seeds:       Vec<TaintSeed>,
    pub propagation: Vec<(OpId, OpId)>,     // (source_op, derived_op)
    pub sinks:       Vec<TaintSinkEntry>,
}

pub struct TaintSeed {
    pub op:    OpId,
    pub label: TaintLabel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaintLabel {
    UserControlled,
    BlockContext,
    SignerArg,
    AccountData,
    ExternalReturn,
    StorageLoaded,
    Clean,
}
```

### 3.7 Function Summary (`summary.rs`)

```rust
pub struct FunctionSummary {
    pub func_id:        FunctionId,
    pub taint_in:       HashMap<ParamIndex, TaintLabel>,
    pub taint_out:      HashMap<ReturnIndex, TaintLabel>,
    pub modifies:       Vec<StorageRef>,
    pub may_revert:     bool,
    pub reentrancy_safe: bool,
    pub value_transfer: bool,
}
```

### 3.8 Call Graph and PDG (`call_graph.rs`, `pdg.rs`)

```rust
pub struct CallGraph {
    pub static_edges:  Vec<(FunctionId, FunctionId)>,
    pub dynamic_edges: Vec<(OpId, FunctionId)>,  // for ExternalCallNode
}

pub struct PDG {
    pub data_edges:    Vec<(OpId, OpId)>,    // def → use
    pub control_edges: Vec<(OpId, BlockId)>, // condition → dominated block
}
```

---

## 4. Lowering Pass 2a (`src/lower/`)

### 4.1 Entry Point (`lower/mod.rs`)

```rust
pub fn lower_module(cir: &scir::Module) -> Result<AnirModule, LowerError>
```

Iterates over each `ContractDecl` → each `FunctionDecl` and runs the five steps.
Produces one `AnirModule` per SCIR `Module`.

### 4.2 Step 1 — EVM Modifier Expansion (`step1_modifier_expand.rs`)

**Input:** `FunctionDecl` with `#evm.modifiers` attribute
**Output:** `FunctionDecl` with modifier bodies inlined at `evm.modifier_placeholder()` positions

Algorithm:
1. Check `attrs` for `#evm.modifiers = [{ ref: "name", args: [...] }]`
2. Look up `evm.modifier_def` `DialectMemberDecl` in the same `ContractDecl`
3. For each modifier reference, clone the modifier body
4. Substitute `evm.modifier_placeholder()` with the function body
5. Return the merged `Vec<Stmt>`

Only runs when `#scir.loaded_dialects` contains `"evm"`.

### 4.3 Step 2 — CFG Construction (`step2_cfg.rs`)

**Input:** `Vec<Stmt>` (structured statements from SCIR)
**Output:** `Vec<BasicBlock>` (labeled basic blocks with edges)

Algorithm — `flatten(stmt, current_block, exit_block)`:

```
Assign / LocalVar / Assert / dialect_stmt / dialect_expr
  → append to current_block; return current_block

If(cond, then, else?)
  → cond_block = current
  → then_exit = flatten(then)
  → else_exit = flatten(else?) [or fall-through]
  → merge_block = new block
  → edges: cond→then(true), cond→else(false), then_exit→merge, else_exit→merge
  → return merge_block

While(cond, body, invariant?)
  → header_block = new; body_exit = flatten(body)
  → edges: current→header, header→body(true), header→after(false)
  → back-edge: body_exit→header
  → return after_block

Return(values)
  → append ReturnOp; edge: current→TxnExit(reverted=false)

Revert / Assert-fail
  → edge: current→TxnExit(reverted=true)
```

`dialect_stmt` and `dialect_expr` nodes are treated as opaque `StmtNode` entries
(they carry interface tags after Step 4).

**Invariant annotations** on `While` are preserved as metadata on the header
block for use by VIR (Pass 2b); ANIR drops them.

### 4.4 Step 3 — SSA Renaming (`step3_ssa.rs`)

**Input:** `Vec<BasicBlock>` with named variables
**Output:** `Vec<BasicBlock>` with versioned SSA names; phi-ops inserted as block arguments

Algorithm: standard Cytron et al. pruned SSA construction.

1. Compute dominance tree and dominance frontier (using reverse-post-order)
2. Place phi-functions at dominance frontiers for each variable defined in more
   than one block
3. Rename pass: maintain per-variable version counter; on each use substitute
   the current version; on each definition increment and record the new version
4. Storage variables get array-SSA treatment: a storage write produces a new
   versioned "array value" (`store(arr_n, key, val) → arr_{n+1}`)
5. Block arguments replace phi-functions at block boundaries

**SsaName:** `{original_name}_{version}` (e.g. `balances_0`, `sender_1`)

### 4.5 Step 4 — Dialect Lowering (`step4_dialect_lower.rs`)

Each dialect registers a `DialectLowering` implementation. This is the **only**
dialect-specific step.

```rust
pub trait DialectLowering {
    fn dialect_name(&self) -> &str;
    fn lower_expr(&self, expr: &DialectExpr, ctx: &LowerCtx) -> Vec<Op>;
    fn lower_stmt(&self, stmt: &DialectStmt, ctx: &LowerCtx) -> Vec<Op>;
}
```

**EVM lowering table** (from §4.2 Step 4 of the design doc):

| SCIR Form | ANIR Op | Interface Tag | Notes |
|----------|--------|---------------|-------|
| `evm.msg_sender()` | `EVM_CALLER` pseudo-value | `TaintSource: UserControlled` | Seeds taint graph |
| `evm.msg_value()` | `EVM_CALLVALUE` pseudo-value | `TaintSource: UserControlled` | Seeds taint graph |
| `balances[k]` (read) | `evm.sload(keccak(slot_index, k))` | `StorageOp` | AliasGroupId = "balances[*]" |
| `balances[k] = v` (write) | `evm.sstore(keccak(slot_index, k), v)` | `StorageOp` | Alias group write |
| `evm.emit_event(...)` | retained as-is | `TaintSink: EventLog` | No CFG edge |
| `evm.call(...)` | `ExternalCallNode` | `CallOp` | Reads `#scir.call_risk.reentrancy` |

**Move lowering table:**

| SCIR Form | ANIR Op | Interface Tag |
|----------|--------|---------------|
| `move.signer_address(s)` | `MOVE_SIGNER_ADDR` pseudo-value | `TaintSource: SignerArg` |
| `move.borrow_global<T>(addr)` | `move.borrow_global` (retained) | `StorageOp` |
| `move.borrow_global_mut<T>(addr)` | `move.borrow_global_mut` (retained) | `StorageOp` |
| `*ref = v` (Move ref write) | `move.write_ref(ref, v)` | `StorageOp` |

**Anchor lowering table:**

| SCIR Form | ANIR Op | Interface Tag |
|----------|--------|---------------|
| `anchor.account_load(ctx.X)` | `anchor.account_load` (retained) | `StorageOp` |
| `anchor.signer_key(ctx.X)` | `ANCHOR_SIGNER_KEY` pseudo-value | `TaintSource: SignerArg` |
| `anchor.cpi(...)` | `ExternalCallNode` | `CallOp` |

**ANIR verifier** (run after Step 4): walk all remaining dialect ops and assert
each implements at least one of `StorageOp`, `CallOp`, `TaintSource`, `TaintSink`.
Any untagged dialect op is a `LowerError::UntaggedDialectOp`.

### 4.6 Step 5 — ICFG, Alias Sets, and Taint Initialisation (`step5_icfg.rs`)

**Input:** SSA-renamed, dialect-lowered functions
**Output:** populated `ICFG`, `AliasMap`, `TaintGraph`

Algorithm (chain-agnostic — runs identically for all dialects):

```
For each FunctionDecl F with #scir.visibility = "public":
  icfg.add(TxnEntry(F))
  icfg.add(TxnExit(F, reverted=false))
  icfg.add(TxnExit(F, reverted=true))

For each SSA op N in any function:
  if N implements StorageOp:
    alias_sets.register(N.to_storage_ref())
    // alias sets group all ops with the same AliasGroupId
  if N implements CallOp:
    if N.call_risk().reentrancy:
      icfg.add(ExternalCallNode(N))
      icfg.add(ReentryPoint(current_func))
      icfg.edge(ExternalCallNode → ReentryPoint)
    else:
      icfg.add(CallSite(N))
  if N implements TaintSource:
    taint_graph.seed(N.result, N.taint_label())
  if N implements TaintSink:
    taint_graph.register_sink(N)
```

**Alias analysis:**
- `AliasGroup` collects all `StorageOp` nodes with the same `AliasGroupId`
- `MayAlias(x, y)` = true if `x.alias_group == y.alias_group` and at least one
  key operand is not syntactically identical in SSA form
- `DefiniteAlias(x, y)` = true if same alias group and same SSA key operand

**Taint propagation initialisation:**
- Seeds placed from `TaintSource` ops
- Initial propagation edges: from each `StorageOp` read to its result
  (label `StorageLoaded`), and from each SSA use of a tainted op

**Function summaries** are computed bottom-up over the call graph:
- `taint_in/out`: max label reaching each param / return
- `modifies`: union of `StorageRef` from all reachable `StorageOp` writes
- `may_revert`: any path reaches `TxnExit(reverted=true)`
- `reentrancy_safe`: no `ExternalCallNode` reachable in ICFG

---

## 5. Rust Crate Structure Details

### 5.1 `Cargo.toml` for `crates/anir`

```toml
[package]
name = "anir"
version = "0.1.0"
edition = "2021"

[dependencies]
scir = { path = "../scir" }
extlib = { path = "../../extlib" }   # adjust if different
indexmap = "2"
thiserror = "1"
```

Add `anir` to the workspace `Cargo.toml` members list.

### 5.2 Error Type

```rust
#[derive(Debug, thiserror::Error)]
pub enum LowerError {
    #[error("Untagged dialect op after Step 4: {0}")]
    UntaggedDialectOp(String),
    #[error("Modifier not found: {0}")]
    ModifierNotFound(String),
    #[error("SSA renaming error: {0}")]
    SsaError(String),
}
```

---

## 6. Implementation Order

Work proceeds in dependency order. Each step can be tested independently.

### Phase A — ANIR Data Structures

1. Create `crates/anir/` with `Cargo.toml`, add to workspace
2. Implement `cfg.rs`: `BasicBlock`, `Terminator`, `ICFGNode`, `ICFG`, `EdgeKind`
3. Implement `interfaces.rs`: four traits + `StorageRef`, `AliasGroupId`, `CallTarget`,
   `CallRisk`, `SinkCategory`
4. Implement `ops.rs`: `Op`, `OpKind`, `SsaName`, `OpId`, `OpRef`, `FunctionId`
5. Implement `alias.rs`: `AliasMap`, `AliasGroup`
6. Implement `taint.rs`: `TaintGraph`, `TaintLabel`, `TaintSeed`
7. Implement `summary.rs`: `FunctionSummary`
8. Implement `call_graph.rs`, `pdg.rs`
9. Implement `module.rs`: `AnirModule`
10. Wire up `lib.rs` re-exports

### Phase B — Pass 2a Lowering

11. `lower/mod.rs`: entry point, orchestration, `LowerError`
12. `lower/step1_modifier_expand.rs`: EVM modifier inlining
13. `lower/step2_cfg.rs`: `flatten()` algorithm, basic block construction
14. `lower/step3_ssa.rs`: dominance tree, frontier, rename pass, array-SSA for storage
15. `lower/step4_dialect_lower.rs`: `DialectLowering` trait, EVM impl, Move impl, Anchor impl;
    ANIR verifier that rejects untagged dialect ops
16. `lower/step5_icfg.rs`: ICFG construction, alias registration, taint seeding,
    function summary computation

### Phase C — Integration and Testing

17. Add unit tests for each step (step2: if/while CFG shapes; step3: SSA names;
    step4: EVM lowering table; step5: alias groups)
18. Add integration test: lower the Vyper ERC-20 SCIR from `crates/vyper` through
    Pass 2a; assert `TxnEntry`, `ExternalCallNode` counts; assert `balances[*]`
    alias group contains expected read/write ops
19. Hook `anir::lower_module` into `crates/smartproof` CLI as a subcommand
    (`--emit=anir` flag that prints the `AnirModule` debug/pretty output)

---

## 7. Key Invariants to Enforce

| Invariant | Where Enforced |
|-----------|---------------|
| Every dialect op in ANIR implements ≥1 interface | Step 4 verifier |
| Every public function has `TxnEntry` + 2× `TxnExit` in ICFG | Step 5 construction |
| Every `StorageOp` write is in an `AliasGroup` | Step 5 registration |
| `ExternalCallNode` ↔ `#scir.call_risk.reentrancy = true` | Step 5 invariant check |
| Spec annotations (`@requires`, `@ensures`) are stripped from ANIR | Step 4 (Pass 2b consumes them) |
| `evm.modifier_def` nodes have no trace in ANIR | Step 1 (fully consumed) |

---

## 8. Out of Scope for This Plan

- Pass 3a (ANIR → Datalog / SMT backend queries)
- Pass 2b (SCIR → VerificationIR)
- The Semgrep-style rule engine operating over ANIR
- Anchor and Move frontends (Pass 1 for those languages)

---

## 9. smarthunt Integration

This section describes how `crates/smarthunt` consumes the `anir` crate to
perform vulnerability detection on `AnirModule`. The existing pass/detector
framework is extended — not replaced — so AST-level detectors continue to work
unchanged.

### 9.1 Updated Pipeline Shape

smarthunt runs three distinct analysis tiers in sequence. Each tier uses a
different IR and a different class of detectors:

```
Source File
    │  Pass 1 (solidity/vyper frontend)
    ▼
SourceUnit (AST)
    │   ├─ Tier 1 — AST detectors (grep patterns, DFA on raw AST)
    │   │   PassRepresentation::Ast
    │   │   Cross-chain: NO  (Solidity AST only today)
    │
    │  IrGeneration pass  → ctx.ir_units: Vec<scir::Module>
    ▼
scir::Module (SCIR)
    │   ├─ Tier 2 — SCIR structural detectors  ← NEW (this plan)
    │   │   PassRepresentation::Ir
    │   │   Operate on the SCIR tree before SSA / dialect lowering
    │   │   Cross-chain: YES (core grammar + dialect_attrs)
    │
    │  AnirGeneration pass  → ctx.anir_units: Vec<anir::AnirModule>
    ▼
anir::AnirModule (ANIR)
    │   └─ Tier 3 — ANIR dataflow detectors   ← NEW (this plan)
    │       PassRepresentation::Air
    │       Operate on SSA + ICFG + alias sets + taint graph
    │       Cross-chain: YES (interfaces, not dialect names)
    ▼
Findings (Bug[])  — deduplicated, merged, emitted
```

**Why both SCIR and ANIR?**

| Property | SCIR (Tier 2) | ANIR (Tier 3) |
|----------|-------------|-------------|
| When | Before SSA | After SSA + lowering |
| Scope | Single function / contract | Interprocedural (ICFG) |
| Finds | Structural code patterns | Dataflow / taint violations |
| Example | Missing `@modifies` clause | Tainted value reaches storage write |
| False positive rate | Low (syntactic) | Medium (alias over-approximation) |
| Cross-chain | Yes — uses core grammar | Yes — uses interface tags |

SCIR detectors catch issues visible in the source structure (missing annotations,
wrong overflow semantics, structural auth absence). ANIR detectors catch issues
that only emerge from tracking values across SSA nodes and call edges.

### 9.2 Changes to `AnalysisContext`

Add an `anir_units` field alongside the existing `ir_units`:

```rust
pub struct AnalysisContext {
    // existing
    pub source_units: Vec<SourceUnit>,
    pub ir_units:     Option<Vec<scir::Module>>,
    // new
    pub anir_units:    Option<Vec<anir::AnirModule>>,
    // ...
}

impl AnalysisContext {
    pub fn has_air(&self) -> bool { self.anir_units.is_some() }

    pub fn anir_units(&self) -> &Vec<anir::AnirModule> {
        self.anir_units.as_ref().expect("ANIR not generated")
    }

    pub fn set_air_units(&mut self, units: Vec<anir::AnirModule>) {
        self.anir_units = Some(units);
    }
}
```

Add `anir` as a dependency in `smarthunt/Cargo.toml`:
```toml
anir = { path = "../anir" }
```

### 9.3 New `PassRepresentation` Variant

Extend `PassRepresentation` with an `Air` variant so the scheduler knows which
passes require ANIR to be present:

```rust
pub enum PassRepresentation {
    Ast,
    Ir,      // requires scir::Module (SCIR)
    Air,     // requires anir::AnirModule  ← new
    Hybrid,
}

impl PassRepresentation {
    pub fn requires_air(&self) -> bool {
        matches!(self, PassRepresentation::Air)
    }
}
```

### 9.4 New `PassId` Variants

Add SCIR structural and ANIR-tier pass IDs to `PassId`:

```rust
pub enum PassId {
    // ... existing variants ...

    // ========================================
    // SCIR Structural Passes  (PassRepresentation::Ir)
    // ========================================
    /// Public function writes storage without any auth guard (SCIR tree pattern)
    ScirMissingAccessControl,
    /// Public storage-writing function has no @modifies annotation
    ScirMissingModifies,
    /// BinOp with OverflowSemantics::Wrapping on a non-constant operand
    ScirUncheckedArithmetic,
    /// EVM: tx.origin used for authentication instead of msg.sender
    ScirTxOriginAuth,
    /// Move: move.borrow_global<T> called on type not in #move.acquires
    ScirAcquiresMismatch,
    /// Anchor: account loaded without #anchor.constraint="seeds=[...]"
    ScirMissingPdaConstraint,

    // ========================================
    // ANIR Generation  (PassRepresentation::Ir — needs SCIR)
    // ========================================
    /// Run Pass 2a: SCIR → AnirModule
    AnirGeneration,

    // ========================================
    // ANIR Analysis Passes  (PassRepresentation::Air)
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
```

**Helper predicates to add / update on `PassId`:**

```rust
impl PassId {
    /// True when the pass operates on scir::Module (SCIR).
    pub fn requires_ir(&self) -> bool {
        matches!(self,
            // existing IR passes...
            | PassId::IrGeneration | PassId::IrCfg | ...
            // new: SCIR structural passes and ANIR generation
            | PassId::ScirMissingAccessControl | PassId::ScirMissingModifies
            | PassId::ScirUncheckedArithmetic  | PassId::ScirTxOriginAuth
            | PassId::ScirAcquiresMismatch     | PassId::ScirMissingPdaConstraint
            | PassId::AnirGeneration
        )
    }

    /// True when the pass operates on anir::AnirModule.
    pub fn requires_air(&self) -> bool {
        matches!(self,
            PassId::AnirTaintPropagation | PassId::AnirReentrancy
            | PassId::AnirAccessControl  | PassId::AnirArithmetic
            | PassId::AnirStorageAliasing | PassId::AnirResourceLinearity
            | PassId::AnirPdaValidation
        )
    }
}
```

### 9.5 ANIR Generation Pass

**File:** `smarthunt/src/analysis/ir/anir_generation.rs`

```rust
pub struct AnirGenerationPass;

impl Pass for AnirGenerationPass {
    fn id(&self)             -> PassId          { PassId::AnirGeneration }
    fn name(&self)           -> &'static str    { "anir-generation" }
    fn description(&self)    -> &'static str    { "Lower SCIR to Anir (Pass 2a)" }
    fn level(&self)          -> PassLevel       { PassLevel::Program }
    fn representation(&self) -> PassRepresentation { PassRepresentation::Ir }
    fn dependencies(&self)   -> Vec<PassId>    { vec![PassId::IrGeneration] }
}

impl AnalysisPass for AnirGenerationPass {
    fn run(&self, ctx: &mut AnalysisContext) -> PassResult<()> {
        let scir_modules = ctx.ir_units().clone();
        let anir_modules = scir_modules
            .iter()
            .map(|m| anir::lower::lower_module(m))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PassError::ExecutionFailed(
                self.name().to_string(), e.to_string()
            ))?;
        ctx.set_air_units(anir_modules);
        Ok(())
    }

    fn is_completed(&self, ctx: &AnalysisContext) -> bool {
        ctx.has_air()
    }
}
```

Register in `PassManager` alongside `IrGenerationPass`.

---

### 9.5a SCIR-Level Structural Passes

These passes operate on `scir::Module` (SCIR) **before** SSA or dialect lowering.
They use `PassRepresentation::Ir`, depend only on `PassId::IrGeneration`, and
run in the existing analysis phase — no new pipeline phase is required.

Because they match only core grammar nodes and `dialect_attrs`, they are
**cross-chain by construction**: the same rule fires for Solidity, Vyper, Move,
and Anchor contracts without any per-dialect branch.

#### Shared trait for SCIR structural passes

**File:** `smarthunt/src/analysis/ir/scir_structural.rs`

```rust
/// A pass that pattern-matches over a scir::Module tree.
pub trait ScirStructuralPass: Pass {
    fn check_module(&self, module: &scir::Module, bugs: &mut Vec<Bug>);
}

impl<T: ScirStructuralPass> AnalysisPass for T {
    fn run(&self, ctx: &mut AnalysisContext) -> PassResult<()> {
        let mut bugs = Vec::new();
        for module in ctx.ir_units() {
            self.check_module(module, &mut bugs);
        }
        ctx.store_artifact(
            &format!("scir.{}", self.id().as_str()),
            bugs,
        );
        ctx.mark_pass_completed(self.id());
        Ok(())
    }
    fn is_completed(&self, ctx: &AnalysisContext) -> bool {
        ctx.is_pass_completed(self.id())
    }
}
```

#### SCIR structural pass catalogue

**1. `ScirMissingAccessControlPass`**
**File:** `smarthunt/src/dfa/detectors/scir_missing_access_control.rs`
**PassId:** `ScirMissingAccessControl`

Pattern (from §4.10.2 of the design doc):
```
function $FUNC(...) attributes { #scir.visibility="public" }
  { ... $STORAGE[$KEY] = $VALUE ... }
unless:
  @requires $AUTH_EXPR present on FuncSpec
  OR Assert(cond=$AUTH_EXPR, ...) before first write in body
```

Walk: for each `FunctionDecl` with `#scir.visibility = "public"`, check whether
the `FuncSpec.requires` list is non-empty **or** the body contains an `Assert`
before the first `Assign`/`AugAssign` that touches a `StorageDecl` name.
If neither: emit `Bug { kind: MissingAccessControl }`.

**Cross-chain:** fires on Solidity, Vyper, Move, Anchor — uses only core
`MemberDecl::Function`, `Stmt::Assert`, and `Stmt::Assign`.

---

**2. `ScirMissingModifiesPass`**
**File:** `smarthunt/src/dfa/detectors/scir_missing_modifies.rs`
**PassId:** `ScirMissingModifies`

Pattern:
```
function $FUNC(...) attributes { #scir.visibility="public" }
  // no @modifies annotation on FuncSpec
  { ... $STORAGE[$KEY] = $VALUE ... }
```

Walk: public functions with a storage write but `FuncSpec.modifies.is_empty()`.
Emit `Bug { kind: MissingModifies, risk: Low }` (informational — aids VIR
frame-condition inference).

---

**3. `ScirUncheckedArithmeticPass`**
**File:** `smarthunt/src/dfa/detectors/scir_unchecked_arithmetic.rs`
**PassId:** `ScirUncheckedArithmetic`

Pattern:
```
BinOp(op=Add|Sub|Mul, lhs, rhs, overflow=Wrapping)
  where lhs or rhs is NOT a Lit node
```

Walk all `Expr::BinOp` nodes recursively. Flag `overflow = Wrapping` with
non-literal operands. Severity depends on dialect: EVM pre-0.8 → `High`;
Move wrapping → `Medium` (Move VM semantics are wrapping by design, lower risk).

**Cross-chain:** `OverflowSemantics::Wrapping` is a core-grammar field on
`BinOpExpr`; no dialect branching needed.

---

**4. `ScirTxOriginAuthPass`** *(EVM-dialect-specific)*
**File:** `smarthunt/src/dfa/detectors/scir_tx_origin_auth.rs`
**PassId:** `ScirTxOriginAuth`

Pattern (§4.10.5):
```
rule TxOriginAuthentication:
  dialect: evm
  $AUTH = evm.tx_origin()    // EvmExpr::TxOrigin, not MsgSender
  assert($AUTH == $EXPECTED, ...)
```

Walk `DialectExpr::Evm(EvmExpr::TxOrigin)` appearing as an operand to `Assert`
or direct comparison. Emit `Bug { kind: TxOriginAuth, risk: High }`.

This is the only SCIR structural pass that is dialect-specific. The check
`dialect: evm` is encoded by matching on `EvmExpr::TxOrigin` — Move and Anchor
contracts will never contain this node.

---

**5. `ScirAcquiresMismatchPass`** *(Move-dialect-specific)*
**File:** `smarthunt/src/dfa/detectors/scir_acquires_mismatch.rs`
**PassId:** `ScirAcquiresMismatch`

Pattern:
```
function $FUNC(...) attributes { #move.acquires = [$DECLARED] }
  { ... move.borrow_global<$T>(...) ... }
  where $T not in $DECLARED
```

Walk `FunctionDecl` with `#move.acquires` attribute; collect all
`DialectExpr::Move(MoveExpr::BorrowGlobal { ty })` in the body; emit if `ty`
is not listed in the attribute.

---

**6. `ScirMissingPdaConstraintPass`** *(Anchor-dialect-specific)*
**File:** `smarthunt/src/dfa/detectors/scir_missing_pda_constraint.rs`
**PassId:** `ScirMissingPdaConstraint`

Pattern (§4.10.5):
```
anchor.accounts_context $CTX {
  $ACC : !anchor.account<$T>
    // missing #anchor.constraint="seeds=[...]"
}
function $FUNC(ctx: !anchor.context<$CTX>, ...) {
  ... anchor.account_load(ctx.accounts.$ACC) ...
}
```

Walk `DialectMemberDecl::Anchor(AnchorMemberDecl::AccountsContext)` entries;
for each account without a seeds constraint, check if the body calls
`anchor.account_load` on it; emit `Bug { kind: MissingPdaValidation, risk: High }`.

---

### 9.6 Standard ANIR Analysis Passes

All four passes follow the same structure: `representation()` returns
`PassRepresentation::Air`, `dependencies()` includes `PassId::AnirGeneration`.
Results are stored as named artifacts in the context for downstream detectors.

---

#### 9.6.1 Taint Propagation Pass

**File:** `smarthunt/src/analysis/ir/anir_taint.rs`
**PassId:** `AnirTaintPropagation`
**Dependencies:** `[AnirGeneration]`

Algorithm — iterative forward dataflow over SSA def-use edges:

```
taint_map: HashMap<OpId, TaintLabel>  // final result artifact

For each AnirModule M:
  // Initialise from seeds
  for seed in M.taint_graph.seeds:
    taint_map[seed.op] = seed.label

  // Propagate: repeat until fixed point
  changed = true
  while changed:
    changed = false
    for each BasicBlock B in topological order:
      for each Op N in B:
        // StorageOp read → result is StorageLoaded
        if N is StorageOp && N.is_read():
          if taint_map.set(N.id, TaintLabel::StorageLoaded):
            changed = true

        // Propagate through BinOp, UnOp, Phi, etc.
        // Result label = max(labels of all operands)
        for each operand R of N:
          if let Some(label) = taint_map.get(R.op_id):
            if taint_map.set(N.id, max(existing, label)):
              changed = true

        // CallOp with unresolved callee → result is ExternalReturn
        if N is CallOp && N.callee() == CallTarget::Dynamic:
          taint_map.set(N.id, TaintLabel::ExternalReturn)

// Store artifact
ctx.store_artifact("anir.taint_map", taint_map);
```

`TaintLabel` ordering (for `max`):
`Clean < StorageLoaded < SignerArg < AccountData < ExternalReturn < BlockContext < UserControlled`

---

#### 9.6.2 Reentrancy Detection Pass

**File:** `smarthunt/src/dfa/detectors/anir_reentrancy.rs`
**PassId:** `AnirReentrancy`
**Dependencies:** `[AnirTaintPropagation]`

This replaces the AST-pattern `ReentrancyDfaDetector` with an ICFG-precise
version. The pattern from §4.8 of the design doc:

```
node A: StorageOp read
node B: ExternalCallNode (CallOp with #scir.call_risk.reentrancy = true)
node C: StorageOp write with alias_group_id == A.alias_group_id
path:   A dominates B, B dominates C (in ICFG)
```

Algorithm:
```
bugs = []
taint_map = ctx.get_artifact::<TaintMap>("anir.taint_map")

for each AnirModule M:
  dom_tree = compute_dominator_tree(M.icfg)

  for each ExternalCallNode B in M.icfg.nodes:
    // Find all StorageOp reads dominated by nothing before B
    reads_before_B = M.alias_sets.all_reads()
        .filter(|r| dom_tree.dominates(r.op_id, B))

    // Find all StorageOp writes that B dominates (in same alias group)
    for each read A in reads_before_B:
      writes_after_B = M.alias_sets.group(A.alias_group_id).writes
          .filter(|w| dom_tree.dominates(B, w.op_id))

      for each write C in writes_after_B:
        // Classic CEI violation: read before call, write after
        bugs.push(Bug {
          kind: BugKind::Reentrancy,
          location: B.op.span,
          message: format!(
            "State variable '{}' read before external call, written after: \
             reentrancy window",
            A.alias_group_id
          ),
          risk: RiskLevel::Critical,
        })

ctx.store_artifact("anir.reentrancy_findings", bugs);
```

**Soundness declaration:** `sound = true` (no false negatives by construction),
`complete = false` (may produce false positives when `MayAlias` is conservative).
Assumes `NoRecursion`.

---

#### 9.6.3 Access Control Inference Pass

**File:** `smarthunt/src/analysis/ir/anir_access_control.rs`
**PassId:** `AnirAccessControl`
**Dependencies:** `[AnirTaintPropagation]`

Checks: every public function that performs a `StorageOp` write must have an
access guard that uses a `SignerArg`-labelled value before the first write.

```
for each AnirModule M:
  taint_map = ctx.get_artifact("anir.taint_map")

  for each FunctionDecl F with #scir.visibility = "public":
    first_write = M.alias_sets.first_write_in_function(F)
    if first_write is None: continue

    // Check: is there an Assert op dominating first_write
    // whose condition operand has label SignerArg or UserControlled
    // (i.e. it checks sender/signer)?
    guard_found = false
    for each Assert op A dominating first_write:
      if taint_map[A.cond] is SignerArg:
        guard_found = true

    // Also accept: function has #scir.access_control attr set by Pass 2a
    if F.attrs.has("scir.access_control"):
      guard_found = true

    if !guard_found:
      emit Bug { kind: MissingAccessControl, ... }
```

---

#### 9.6.4 Arithmetic Analysis Pass

**File:** `smarthunt/src/analysis/ir/anir_arithmetic.rs`
**PassId:** `AnirArithmetic`
**Dependencies:** `[AnirTaintPropagation]`

Detects integer overflow/underflow on `BinOp` nodes where:
- `overflow = OverflowSemantics::Wrapping` (pre-0.8 Solidity, Move VM wrapping)
- At least one operand has `TaintLabel >= StorageLoaded` (i.e. not a literal constant)

```
for each BinOp N with overflow == Wrapping:
  if taint_map[N.lhs] >= StorageLoaded
     OR taint_map[N.rhs] >= StorageLoaded:
    emit Bug { kind: IntegerOverflow, location: N.span, ... }
```

For `overflow = Checked` (Solidity ≥0.8), the runtime reverts — no bug. For
`overflow = Saturating` (Cairo felt252), emit an informational note only.

---

### 9.7 Wiring Detectors into the Pipeline Engine

**File:** `smarthunt/src/pipeline/engine.rs` (update existing)

Extend the pipeline from two phases to four — one per IR tier plus detection:

```rust
pub fn run(&self, ctx: &mut AnalysisContext) -> PipelineResult {
    // Phase 1 — AST analysis (existing, unchanged)
    // Runs: SymbolTable, TypeIndex, InheritanceGraph, CallGraph, ModifierAnalysis, ...
    self.run_analysis_phase(ctx);

    // Phase 2 — SCIR structural analysis  ← NEW
    // Requires: IrGeneration (populates ctx.ir_units)
    // Runs: ScirMissingAccessControl, ScirMissingModifies,
    //       ScirUncheckedArithmetic, ScirTxOriginAuth,
    //       ScirAcquiresMismatch, ScirMissingPdaConstraint
    if ctx.has_ir() {
        self.run_scir_phase(ctx);
    }

    // Phase 3 — ANIR dataflow analysis    ← NEW
    // Requires: IrGeneration (to run AnirGeneration first)
    // Runs: AnirGeneration, AnirTaintPropagation, AnirReentrancy,
    //       AnirAccessControl, AnirArithmetic, AnirStorageAliasing,
    //       AnirResourceLinearity, AnirPdaValidation
    if ctx.has_ir() && ctx.config.enable_air {
        self.run_air_phase(ctx);
    }

    // Phase 4 — detection: harvest artifacts from all three tiers
    self.run_detection_phase(ctx);
}

fn run_scir_phase(&self, ctx: &mut AnalysisContext) {
    // All ScirStructuralPass impls use PassRepresentation::Ir.
    // The PassManager schedules them after IrGeneration completes.
    self.pass_manager.run_scheduled(ctx, PassRepresentation::Ir);
}

fn run_air_phase(&self, ctx: &mut AnalysisContext) {
    // Dependency order:
    //   AnirGeneration
    //     → AnirTaintPropagation
    //         → AnirReentrancy
    //         → AnirAccessControl
    //         → AnirArithmetic
    //         → AnirStorageAliasing
    //     → AnirResourceLinearity (gated: move/tezos dialect present)
    //     → AnirPdaValidation     (gated: anchor dialect present)
    self.pass_manager.run_scheduled(ctx, PassRepresentation::Air);
}
```

**Config additions:**
```rust
pub struct AnalysisConfig {
    // existing...
    pub enable_ir:  bool,   // existing — enables SCIR generation
    pub enable_air: bool,   // new — enables ANIR generation + ANIR passes
                            // default: true when enable_ir = true
}
```

**Dialect-gating** for chain-specific SCIR and ANIR passes: before running
`ScirTxOriginAuthPass`, `ScirAcquiresMismatchPass`, or `ScirMissingPdaConstraintPass`,
the scheduler checks the loaded dialect in `ctx.ir_units[i].attrs`
(`#scir.loaded_dialects`). Only passes whose required dialect is present are
scheduled. This avoids false "not found" errors on cross-chain runs.

### 9.8 ANIR-Based `BugDetectionPass` Implementations

Each ANIR analysis pass stores its findings as an artifact. Bug detection passes
simply harvest those artifacts:

```rust
pub struct AnirReentrancyDetector;

impl BugDetectionPass for AnirReentrancyDetector {
    fn detect(&self, ctx: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        // Findings are pre-computed by AnirReentrancy analysis pass
        let findings = ctx
            .get_artifact::<Vec<Bug>>("anir.reentrancy_findings")
            .ok_or(DetectorError::MissingAnalysis("AnirReentrancy".into()))?;
        Ok(findings.clone())
    }
}

impl Pass for AnirReentrancyDetector {
    fn id(&self)           -> PassId { PassId::Reentrancy }
    fn representation(&self) -> PassRepresentation { PassRepresentation::Air }
    fn dependencies(&self) -> Vec<PassId> { vec![PassId::AnirReentrancy] }
    // ...
}
```

The same pattern applies to `AnirAccessControlDetector`, `AnirArithmeticDetector`,
`AnirStorageAliasingDetector`.

**Coexistence with existing detectors:** The existing `ReentrancyDfaDetector`
(AST-based) continues to run as a fallback when ANIR is not available (e.g. for
languages without a SCIR frontend yet). When ANIR is present, the `PipelineEngine`
skips the AST-based reentrancy detector to avoid duplicate findings.

### 9.9 Detector Registry Updates

**File:** `smarthunt/src/pipeline/registry.rs`

Register SCIR structural and ANIR dataflow detectors alongside existing AST ones:

```rust
pub fn register_all_detectors(registry: &mut DetectorRegistry) {
    // ── Tier 1: existing AST detectors (unchanged) ──────────────────
    registry.register(Box::new(TxOriginDetector));         // AST fallback
    registry.register(Box::new(ReentrancyDfaDetector));   // AST fallback
    registry.register(Box::new(FloatingPragmaDetector));
    // ... (all existing)

    // ── Tier 2: SCIR structural detectors (new) ─────────────────────
    registry.register(Box::new(ScirMissingAccessControlDetector));
    registry.register(Box::new(ScirMissingModifiesDetector));
    registry.register(Box::new(ScirUncheckedArithmeticDetector));
    // dialect-specific SCIR detectors (gated by loaded dialect):
    registry.register(Box::new(ScirTxOriginAuthDetector));       // evm
    registry.register(Box::new(ScirAcquiresMismatchDetector));   // move
    registry.register(Box::new(ScirMissingPdaConstraintDetector)); // anchor

    // ── Tier 3: ANIR dataflow detectors (new) ───────────────────────
    registry.register(Box::new(AnirReentrancyDetector));
    registry.register(Box::new(AnirAccessControlDetector));
    registry.register(Box::new(AnirArithmeticDetector));
    registry.register(Box::new(AnirStorageAliasingDetector));
}
```

**Deduplication rule:** when both a SCIR/ANIR detector and an AST fallback detector
produce findings for the same location (same file + line range), the pipeline
engine keeps only the higher-tier finding. This is implemented as a post-detection
merge step keyed on `(file, line, BugKind)`.

| AST fallback | Suppressed when |
|---|---|
| `TxOriginDetector` (AST) | `ScirTxOriginAuthDetector` fires |
| `ReentrancyDfaDetector` (AST) | `AnirReentrancyDetector` fires |
| `MissingAccessControl` (AST grep) | `ScirMissingAccessControlDetector` fires |
| `IntegerOverflow` (IR stub) | `ScirUncheckedArithmeticDetector` or `AnirArithmeticDetector` fires |

### 9.10 smarthunt Implementation Order

This builds on Phases A–C from §6 (which produce the `anir` crate). After that:

**Phase D — smarthunt Wiring**

20. Add `anir` dependency to `smarthunt/Cargo.toml`
21. Extend `AnalysisContext` with `anir_units` + `has_air()` / `set_air_units()`
22. Add `PassRepresentation::Air` variant; update `PassManager` scheduler to
    gate `Air` passes on `ctx.has_air()` and `Ir` passes on `ctx.has_ir()`
23. Add SCIR and ANIR `PassId` variants; implement `requires_ir()` / `requires_air()`
24. Update `PipelineEngine` with `run_scir_phase` + `run_air_phase`; add
    `enable_air` to `AnalysisConfig`; add dialect-gating to scheduler

**Phase E — SCIR Structural Passes**

25. Implement `ScirStructuralPass` shared trait in `analysis/ir/scir_structural.rs`
26. Implement `ScirMissingAccessControlDetector` in
    `dfa/detectors/scir_missing_access_control.rs`
27. Implement `ScirMissingModifiesDetector` in
    `dfa/detectors/scir_missing_modifies.rs`
28. Implement `ScirUncheckedArithmeticDetector` in
    `dfa/detectors/scir_unchecked_arithmetic.rs`
29. Implement `ScirTxOriginAuthDetector` (evm) in
    `dfa/detectors/scir_tx_origin_auth.rs`
30. Implement `ScirAcquiresMismatchDetector` (move) in
    `dfa/detectors/scir_acquires_mismatch.rs`
31. Implement `ScirMissingPdaConstraintDetector` (anchor) in
    `dfa/detectors/scir_missing_pda_constraint.rs`

**Phase F — ANIR Analysis Passes**

32. Implement `AnirGenerationPass` in `analysis/ir/anir_generation.rs`
33. Implement `AnirTaintPropagationPass` in `analysis/ir/anir_taint.rs`
34. Implement `AnirAccessControlPass` in `analysis/ir/anir_access_control.rs`
35. Implement `AnirArithmeticPass` in `analysis/ir/anir_arithmetic.rs`

**Phase G — ANIR Detectors**

36. Implement `AnirReentrancyPass` + `AnirReentrancyDetector` in
    `dfa/detectors/anir_reentrancy.rs`
37. Implement `AnirAccessControlDetector`, `AnirArithmeticDetector`
38. Update `DetectorRegistry` to register all SCIR and ANIR detectors
39. Add deduplication merge step in `PipelineEngine::run_detection_phase`

**Phase H — Testing**

40. Unit test — SCIR structural: load ERC-20 SCIR, assert
    `ScirMissingAccessControlDetector` fires on a no-auth public function and
    does not fire on one protected by `@requires`
41. Unit test — SCIR arithmetic: construct a SCIR `BinOp(Add, Wrapping)` with
    a `Var` operand; assert `ScirUncheckedArithmeticDetector` fires
42. Unit test — ANIR generation: load Vyper ERC-20 SCIR, run `AnirGenerationPass`,
    assert `anir_units` is populated and taint seeds include `EVM_CALLER`
43. Unit test — taint propagation: assert `evm.msg_sender()` result is labelled
    `UserControlled` after `AnirTaintPropagationPass`
44. Integration test — reentrancy: run full pipeline on DAO-style contract;
    assert `AnirReentrancyDetector` fires, AST fallback detector does not duplicate
45. Integration test — access control: run on a no-auth storage-writing public
    function; assert both `ScirMissingAccessControlDetector` (structural) and
    `AnirAccessControlDetector` (taint-flow) fire; deduplicate to one finding
46. Integration test — cross-chain: run on Move BasicCoin SCIR; assert
    `ScirAcquiresMismatchDetector` fires when `borrow_global<T>` type is absent
    from `#move.acquires`; assert EVM-specific detectors do not fire

---

## 10. Summary of New Files

**New crate:**

| Path | Purpose |
|------|---------|
| `crates/anir/` | ANIR data structures + Pass 2a lowering (`lower_module`) |

**SCIR-tier additions (Tier 2 — structural, `PassRepresentation::Ir`):**

| File | Purpose |
|------|---------|
| `smarthunt/src/analysis/ir/scir_structural.rs` | Shared `ScirStructuralPass` trait |
| `smarthunt/src/dfa/detectors/scir_missing_access_control.rs` | Structural auth-guard check |
| `smarthunt/src/dfa/detectors/scir_missing_modifies.rs` | Missing `@modifies` annotation |
| `smarthunt/src/dfa/detectors/scir_unchecked_arithmetic.rs` | `BinOp(Wrapping)` on non-literals |
| `smarthunt/src/dfa/detectors/scir_tx_origin_auth.rs` | EVM `tx.origin` auth misuse |
| `smarthunt/src/dfa/detectors/scir_acquires_mismatch.rs` | Move `borrow_global` vs `#move.acquires` |
| `smarthunt/src/dfa/detectors/scir_missing_pda_constraint.rs` | Anchor missing PDA seeds constraint |

**ANIR-tier additions (Tier 3 — dataflow, `PassRepresentation::Air`):**

| File | Purpose |
|------|---------|
| `smarthunt/src/analysis/ir/anir_generation.rs` | `AnirGenerationPass` (runs Pass 2a) |
| `smarthunt/src/analysis/ir/anir_taint.rs` | `AnirTaintPropagationPass` |
| `smarthunt/src/analysis/ir/anir_access_control.rs` | `AnirAccessControlPass` |
| `smarthunt/src/analysis/ir/anir_arithmetic.rs` | `AnirArithmeticPass` |
| `smarthunt/src/dfa/detectors/anir_reentrancy.rs` | ICFG-based reentrancy detector |
| `smarthunt/src/dfa/detectors/anir_access_control.rs` | Taint-flow access control detector |
| `smarthunt/src/dfa/detectors/anir_arithmetic.rs` | Tainted-operand overflow detector |

**Modified files:**

| File | Change |
|------|--------|
| `smarthunt/src/analysis/context.rs` | Add `anir_units` field + `has_air()`, `set_air_units()` |
| `smarthunt/src/analysis/pass_representation.rs` | Add `Air` variant + `requires_air()` |
| `smarthunt/src/analysis/pass_id.rs` | Add `Cir*` and `Air*` pass IDs; update `requires_ir()` / `requires_air()` |
| `smarthunt/src/pipeline/engine.rs` | Add `run_scir_phase` + `run_air_phase`; deduplication merge |
| `smarthunt/src/pipeline/registry.rs` | Register SCIR structural + ANIR dataflow detectors |
| `smarthunt/Cargo.toml` | Add `anir = { path = "../anir" }` |
| Workspace `Cargo.toml` | Add `anir` to members list |

**Detector coverage by tier and chain:**

| Detector | Tier | Cross-chain? | Dialect gate |
|----------|------|-------------|--------------|
| `ScirMissingAccessControl` | SCIR | Yes | — |
| `ScirMissingModifies` | SCIR | Yes | — |
| `ScirUncheckedArithmetic` | SCIR | Yes | — |
| `ScirTxOriginAuth` | SCIR | No | evm |
| `ScirAcquiresMismatch` | SCIR | No | move |
| `ScirMissingPdaConstraint` | SCIR | No | anchor |
| `AnirReentrancy` | ANIR | Yes | — |
| `AnirAccessControl` | ANIR | Yes | — |
| `AnirArithmetic` | ANIR | Yes | — |
| `AnirStorageAliasing` | ANIR | Mostly | delegatecall rule: evm |
