# Plan: Fix BIR Naming Conventions

## 1. Rename `__flat_N__` temporaries to `__tmp_N`

The `flatten_expr` pass introduces temporaries with the `__flat_{counter}__`
pattern. Only one line needs changing:

| File | Line | Current | Target |
|------|------|---------|--------|
| `crates/scirs/src/cir/lower/flatten_expr.rs` | 388 | `"__flat_{counter}__"` | `"__tmp_{counter}"` |

No other source or test files reference the `__flat_` prefix.

---

## 2. SSA-style naming in CIR → BIR lowering

Adopt SSA-style naming conventions for functions, basic blocks, and variables
in the BIR output.

### 2.1 Functions: `@` prefix

**Current**: `FunctionId` displays as bare `Contract.function`.

**Target**: prefix with `@`, e.g. `@Contract.function`.

| File | Line | Current | Target |
|------|------|---------|--------|
| `crates/scirs/src/bir/cfg.rs` (Display for FunctionId) | 30 | `write!(f, "{}", self.0)` | `write!(f, "@{}", self.0)` |

### 2.2 Basic blocks: `%bbN`

**Current**: `BlockId` displays as `bb0`, `bb1`, …

**Target**: prefix with `%`, e.g. `%bb0`, `%bb1`, …

| File | Line | Current | Target |
|------|------|---------|--------|
| `crates/scirs/src/bir/cfg.rs` (Display for BlockId) | 24 | `write!(f, "bb{}", self.0)` | `write!(f, "%bb{}", self.0)` |

### 2.3 Variables: `%vN`

**Current**: `SsaName` stores a `base` (original var name) and `version`.
Display renders `{base}_{version}`, e.g. `amount_0`, `__flat_1___0`.

**Target**: use a single global counter, display as `%v1`, `%v2`, …
The original name becomes a comment/debug hint only.

Changes needed:

#### a. Data structure (`crates/scirs/src/bir/ops.rs`)

- Change `SsaName.Display` from `{base}_{version}` to `%v{version}`.
  - Keep `base` field for debug/comment purposes.
  - Alternatively, add a `global_id: usize` field and display `%v{global_id}`.

| File | Line | Current | Target |
|------|------|---------|--------|
| `crates/scirs/src/bir/ops.rs` (Display for SsaName) | 51 | `write!(f, "{}_{}", self.base, self.version)` | `write!(f, "%v{}", self.version)` |

#### b. SSA renaming pass (`crates/scirs/src/bir/lower/ssa.rs`)

- Use a **single global counter** across all variables (not per-variable
  version maps) so that each variable gets a unique `%vN`.
- Currently uses `HashMap<String, u32>` keyed by base name → change to a
  single `u32` counter.

| File | Lines | Change |
|------|-------|--------|
| `crates/scirs/src/bir/lower/ssa.rs` | 16–36 | Replace per-variable `version_map` with a single `next_id: u32` counter. Assign each op result a unique, monotonically increasing ID. |

#### c. CFG construction (`crates/scirs/src/bir/lower/cfg.rs`)

- All calls to `SsaName::new(&name, 0)` currently set version 0 as a
  placeholder; the SSA pass overrides it. **No change needed** here — the
  SSA pass will assign the final IDs.

---

## 3. BIR → FIR lowering (Functional IR)

Lower BIR basic-block CFGs into **FIR** — a functional IR where every basic
block becomes a tail-recursive function in parameter-passing form.

### 3.1 Overview

Each BIR `Function` (`@Foo`) is lowered into a **family of FIR functions**:

| BIR construct | FIR function | Description |
|---------------|-------------|-------------|
| Entry block (`%bb0`) | `@Foo` | Original function; body = ops of `%bb0` + tail call |
| Block `%bb1` | `@Foo$1` | Lifted from `%bb1` |
| Block `%bb2` | `@Foo$2` | Lifted from `%bb2` |
| … | `@Foo$N` | One function per basic block |

Naming: functions, variables, and blocks retain BIR conventions (`@`, `%v`,
`%bb`). Lifted block-functions are suffixed `$N` where N = block index.

### 3.2 Translation rules

#### Terminators → tail calls

| BIR Terminator | FIR translation |
|----------------|-----------------|
| `jump %bbK` | `tail call @Foo$K(%v_live_in…)` |
| `branch %cond, %bbT, %bbF` | `if %cond then tail call @Foo$T(…) else tail call @Foo$F(…)` |
| `txn_exit(ok)` | `return …` |
| `txn_exit(reverted)` | `revert` |
| `unreachable` | `unreachable` |

#### Block parameters (live-in variables)

Each lifted function's parameters are the **live-in SSA variables** of the
corresponding basic block. Phi nodes are eliminated: each predecessor passes
the appropriate value as an argument in the tail call.

#### Phi elimination

A BIR phi `%v5 = phi [%bb0: %v1, %bb2: %v3]` at the top of `%bbK` becomes:
- `%v5` is added as a parameter of `@Foo$K`
- At the tail call from `%bb0`: `tail call @Foo$K(…, %v1, …)`
- At the tail call from `%bb2`: `tail call @Foo$K(…, %v3, …)`

### 3.3 Data structures (new module `crates/scirs/src/fir/`)

| File | Purpose |
|------|---------|
| `mod.rs` | Module definition, re-exports |
| `ops.rs` | FIR ops: reuse BIR `OpKind` + add `TailCall`, `Return`, `Revert` |
| `module.rs` | `Module` — collection of `Function`s (accessed as `fir::Module`) |
| `lower/mod.rs` | BIR → FIR lowering orchestration |
| `lower/lift_blocks.rs` | Core: lift each basic block into a `fir::Function` |

### 3.4 Key types

```rust
// accessed as fir::Module, fir::Function, fir::Terminator

struct Module {
    functions: Vec<Function>,
}

struct Function {
    id: FunctionId,       // e.g. @Foo, @Foo$1, @Foo$2
    params: Vec<(SsaName, Type)>,  // live-in variables
    body: Vec<Op>,        // ops from the basic block
    term: Terminator,     // tail call / return / revert
}

enum Terminator {
    TailCall { callee: FunctionId, args: Vec<OpRef> },
    Branch { cond: OpRef, then_call: TailCallData, else_call: TailCallData },
    Return(Vec<OpRef>),
    Revert,
    Unreachable,
}
```

### 3.5 Lowering algorithm

1. For each BIR `Function` (`@Foo`):
   a. Compute live-in sets for each block (from SSA def-use chains).
   b. For each `BasicBlock` `%bbN`:
      - Collect phi parameters → function params.
      - Collect remaining live-in vars → additional params.
      - Copy block ops (excluding phis) → function body.
      - Convert terminator → `fir::Terminator` (tail call with live-in args).
      - Name the function `@Foo$N` (or `@Foo` for `%bb0`).

---

## 4. Utils modules: `visit` and `fold` for each IR layer

Add `utils/` sub-modules to **CIR**, **BIR**, and **FIR** following the same
design pattern already used by:

- `frontend/src/solidity/ast/utils/{visit.rs, fold.rs}`
- `scirs/src/sir/utils/{visit.rs, fold.rs}` *(already exists)*

### 4.1 Design pattern

Same as Solidity AST and SIR: one trait method **per node variant**, with
default impls that recurse into children. A companion `default` module
provides free functions so implementors can selectively override.

```rust
// Visit — read-only traversal (example from SIR)
pub trait Visit<'a> {
    fn visit_module(&mut self, m: &'a Module) { default::visit_module(self, m) }
    fn visit_if_stmt(&mut self, s: &'a IfStmt) { default::visit_if_stmt(self, s) }
    fn visit_binop_expr(&mut self, e: &'a BinOpExpr) { default::visit_binop_expr(self, e) }
    // …one method per variant…
}

// Fold — accumulating traversal
pub trait Fold<'a, T: Default> {
    fn fold_module(&mut self, m: &'a Module) -> T { default::fold_module(self, m) }
    fn fold_if_stmt(&mut self, s: &'a IfStmt) -> T { default::fold_if_stmt(self, s) }
    fn fold_binop_expr(&mut self, e: &'a BinOpExpr) -> T { default::fold_binop_expr(self, e) }
    fn combine(&mut self, a: T, b: T) -> T { a }
    // …one method per variant…
}
```

### 4.2 IR-specific methods

#### SIR (`scirs/src/sir/utils/`) — already exists

No changes needed.

#### CIR (`scirs/src/cir/utils/`) — **[NEW]**

`visit.rs` and `fold.rs` — methods mirror the CIR node types:

**Module / Decl level:**
`visit_module`, `visit_decl`, `visit_contract_decl`, `visit_member_decl`,
`visit_storage_decl`, `visit_function_decl`

**Statements** (one per `CanonStmt` variant):
`visit_stmt`, `visit_local_var_stmt`, `visit_assign_stmt`,
`visit_aug_assign_stmt`, `visit_expr_stmt`, `visit_if_stmt`,
`visit_while_stmt`, `visit_for_stmt`, `visit_return_stmt`,
`visit_revert_stmt`, `visit_assert_stmt`

**Expressions** (one per `CanonExpr` variant):
`visit_expr`, `visit_var_expr`, `visit_binop_expr`, `visit_unop_expr`,
`visit_index_access_expr`, `visit_field_access_expr`, `visit_call_expr`,
`visit_type_cast_expr`, `visit_old_expr`, `visit_forall_expr`,
`visit_exists_expr`

#### BIR (`scirs/src/bir/utils/`) — **[NEW]**

`visit.rs` and `fold.rs` — methods mirror BIR node types:

**Module / Function level:**
`visit_module`, `visit_function`, `visit_basic_block`

**Ops** (one per `OpKind` variant):
`visit_op`, `visit_const_op`, `visit_binop_op`, `visit_unop_op`,
`visit_phi_op`, `visit_assert_op`, `visit_return_op`, `visit_param_op`,
`visit_expr_stmt_op`, `visit_storage_op`, `visit_call_op`,
`visit_taint_src_op`, `visit_taint_snk_op`, `visit_opaque_op`

**Terminators** (one per `Terminator` variant):
`visit_terminator`, `visit_branch_term`, `visit_jump_term`,
`visit_txn_exit_term`

#### FIR (`scirs/src/fir/utils/`) — **[NEW]**

`visit.rs` and `fold.rs` — methods mirror FIR node types:

**Module / Function level:**
`visit_module`, `visit_function`

**Ops:** same as BIR (reuses `OpKind`)

**Terminators** (one per `fir::Terminator` variant):
`visit_terminator`, `visit_tail_call_term`, `visit_branch_term`,
`visit_return_term`, `visit_revert_term`

---

## 5. IR verifier modules

Add a `verifier/` sub-module to each IR layer that checks structural
invariants using the `visit`/`fold` utilities. Each verifier exposes a
`fn verify(module: &Module) -> Result<(), Vec<VerifyError>>` entry point.

### 5.1 Common infrastructure

All verifiers share a common error type in `crates/scirs/src/common/`:

```rust
pub struct VerifyError {
    pub pass: &'static str,   // e.g. "sir::type_check"
    pub message: String,
    pub span: Option<Span>,
}
```

Each pass is a struct implementing either `Visit` or `Fold`:

- **`Visit`** — for passes that only need to **detect forbidden patterns**
  (e.g. "no ternary", "no phi"). The visitor walks the tree and pushes
  errors into a `Vec<VerifyError>` stored in `&mut self`.
- **`Fold`** — for passes that need to **collect and cross-reference**
  information across nodes (e.g. collecting all defs to check uniqueness,
  building a scope map, comparing argument counts against callee signatures).
  The accumulator `T` is typically a `HashSet` or `HashMap`.


### 5.2 SIR verifier (`scirs/src/sir/verifier/`)

| Pass | Impl | Checks |
|------|------|--------|
| `type_well_formed` | `Visit` | Every `Expr` has a non-`None` type; function return types match body |
| `scope_check` | `Fold` | Variables used in expressions are declared in an enclosing scope |
| `spec_check` | `Visit` | `requires`/`ensures` clauses reference only params, `old()`, and `result` |
| `no_orphan_dialect` | `Visit` | No `Dialect` variants appear outside of recognized dialect contexts |

### 5.3 CIR verifier (`scirs/src/cir/verifier/`)

Checks that SIR → CIR canonicalization invariants hold:

| Pass | Impl | Checks |
|------|------|--------|
| `no_named_args` | `Visit` | No `CallArgs::Named` remain (all eliminated by `elim_named_args`) |
| `no_using_for` | `Visit` | No `UsingFor` member decls remain (eliminated by `elim_using`) |
| `no_modifiers` | `Visit` | No modifier definitions remain (inlined by `elim_modifiers`) |
| `atomic_call_args` | `Visit` | All function call arguments are atoms (var or literal), not complex exprs (guaranteed by `flatten_expr`) |
| `no_ternary` | `Visit` | No `Ternary` expressions remain (lowered during CIR conversion) |
| `no_inheritance` | `Visit` | All contracts have empty `parents` (resolved by `resolve_inheritance`) |

### 5.4 BIR verifier (`scirs/src/bir/verifier/`)

Checks SSA and CFG structural invariants:

| Pass | Impl | Checks |
|------|------|--------|
| `ssa_single_def` | `Fold` | Every `SsaName` is defined (assigned) exactly once across all blocks |
| `ssa_use_def` | `Fold` | Every `OpRef` references a previously defined `OpId` (def dominates use or via phi) |
| `cfg_well_formed` | `Visit` | Every block has a terminator; terminator targets reference valid `BlockId`s |
| `cfg_entry_exit` | `Visit` | Exactly one entry block (`%bb0`); every `TxnExit` block has no successors |
| `phi_consistency` | `Visit` | Every `Phi` node has one entry per predecessor block; no phi in entry block without predecessors |
| `op_id_unique` | `Fold` | All `OpId` values are unique within a function |

### 5.5 FIR verifier (`scirs/src/fir/verifier/`)

Checks functional tail-recursive form invariants:

| Pass | Impl | Checks |
|------|------|--------|
| `no_phi` | `Visit` | No `Phi` ops remain (eliminated into function params) |
| `tail_call_targets` | `Visit` | Every `TailCall` callee references a function in the same family (`@Foo$N`) |
| `param_arity` | `Fold` | Tail call argument count matches the callee function's parameter count |
| `no_orphan_blocks` | `Visit` | Every function in the family is reachable from `@Foo` via tail calls |
| `terminal_completeness` | `Visit` | Every function body ends with exactly one `fir::Terminator` |

### 5.6 Module structure

Each `verifier/` directory:

```
verifier/
├── mod.rs          // pub fn verify(...) orchestrates all passes
├── <pass_name>.rs  // one file per pass (struct + Visit/Fold impl)
```

---

## 6. Relocate lowering modules into source IR

Each lowering pass should live in the **source** IR module, not the target.
This follows the convention that the producer knows how to emit the next IR.

### 6.1 Current vs target layout

| Lowering | Current location | Target location |
|----------|-----------------|-----------------|
| SIR → CIR | `cir/lower/` | `sir/lower/` |
| CIR → BIR | `bir/lower/` | `cir/lower/` |
| BIR → FIR | `fir/lower/` (planned) | `bir/lower/` |

### 6.2 SIR → CIR: move `cir/lower/` → `sir/lower/`

Move all files currently in `crates/scirs/src/cir/lower/`:

| File | Description |
|------|-------------|
| `mod.rs` | Orchestration + structural SIR→CIR conversion |
| `elim_named_args.rs` | Named → positional call args |
| `elim_using.rs` | Strip UsingFor declarations |
| `elim_modifiers.rs` | Inline modifier bodies |
| `resolve_inheritance.rs` | Flatten inheritance hierarchy |
| `flatten_expr.rs` | Introduce temporaries for call args |

After move:
- `sir/mod.rs` gets `pub mod lower;`
- `cir/mod.rs` removes `pub mod lower;`
- All `use crate::cir::lower::…` imports elsewhere update to `use crate::sir::lower::…`

### 6.3 CIR → BIR: move `bir/lower/` → `cir/lower/`

Move all files currently in `crates/scirs/src/bir/lower/`:

| File | Description |
|------|-------------|
| `mod.rs` | Four-step CIR→BIR orchestration |
| `cfg.rs` | CFG construction |
| `ssa.rs` | SSA renaming |
| `dialect_lower.rs` | Dialect op lowering |
| `icfg.rs` | ICFG + alias + taint init |

After move:
- `cir/mod.rs` gets `pub mod lower;` (re-purposed for CIR→BIR)
- `bir/mod.rs` removes `pub mod lower;`
- All `use crate::bir::lower::…` imports update to `use crate::cir::lower::…`

### 6.4 BIR → FIR: create in `bir/lower/` directly

The planned FIR lowering (section 3) should be created as `bir/lower/`
instead of `fir/lower/`:

| File | Description |
|------|-------------|
| `mod.rs` | BIR→FIR orchestration |
| `lift_blocks.rs` | Block-to-function lifting |

After creation:
- `bir/mod.rs` gets `pub mod lower;` (now for BIR→FIR)

### 6.5 Dependency direction

```
sir  ──lower──▸  cir  ──lower──▸  bir  ──lower──▸  fir
 │                │                │                │
 ▼                ▼                ▼                ▼
sir::lower     cir::lower     bir::lower        (pure data)
 produces        produces        produces
  cir::*          bir::*          fir::*
```

Each `lower` module depends on both its own IR types and the target IR types.
The target IR module remains a pure data + utils + verifier module with no
`lower` sub-module of its own (except when it too is a source for the next stage).

---

## Files summary

| File | Changes |
|------|---------|
| `crates/scirs/src/cir/lower/flatten_expr.rs` | `__flat_{N}__` → `__tmp_{N}` |
| `crates/scirs/src/bir/cfg.rs` | `FunctionId` display → `@…`, `BlockId` display → `%bb…` |
| `crates/scirs/src/bir/ops.rs` | `SsaName` display → `%v{N}` |
| `crates/scirs/src/bir/lower/ssa.rs` | Use global counter for unique `%vN` IDs |
| `crates/scirs/src/fir/` | **[NEW]** FIR module: data structures (no lower/) |
| `crates/scirs/src/cir/utils/` | **[NEW]** `Visit` + `Fold` traits for CIR |
| `crates/scirs/src/bir/utils/` | **[NEW]** `Visit` + `Fold` traits for BIR |
| `crates/scirs/src/fir/utils/` | **[NEW]** `Visit` + `Fold` traits for FIR |
| `crates/scirs/src/sir/verifier/` | **[NEW]** SIR verification passes |
| `crates/scirs/src/cir/verifier/` | **[NEW]** CIR verification passes |
| `crates/scirs/src/bir/verifier/` | **[NEW]** BIR verification passes |
| `crates/scirs/src/fir/verifier/` | **[NEW]** FIR verification passes |
| `crates/scirs/src/sir/lower/` | **[MOVE]** SIR→CIR lowering (from `cir/lower/`) |
| `crates/scirs/src/cir/lower/` | **[MOVE]** CIR→BIR lowering (from `bir/lower/`) |
| `crates/scirs/src/bir/lower/` | **[NEW]** BIR→FIR lowering (was planned in `fir/lower/`) |
| `crates/scirs/src/lib.rs` | Add `pub mod fir;` |

## Verification

1. `cargo build` — must compile cleanly.
2. `cargo test` — no regressions.
3. Grep for old patterns:
   ```
   rg '__flat_' --type rust
   ```
4. Run the scanner on a sample Solidity contract and inspect BIR output to
   confirm: functions show `@`, blocks show `%bb`, variables show `%v`.
5. Inspect FIR output to confirm: each basic block lifted into `@Foo$N`,
   terminators converted to tail calls, phis eliminated into parameters.
6. Verify each `utils/` module compiles and trait methods are callable
   (e.g. a simple test that visits all nodes without panicking).
7. Run each IR verifier on a sample module and confirm all passes return `Ok`.
8. Add unit tests for each verifier pass with intentionally malformed IR
   inputs to confirm errors are detected.
9. Grep for stale `crate::cir::lower` and `crate::bir::lower` imports to
   confirm all references point to the new locations.

---

## Task checklist

### 1. Rename `__flat_N__` → `__tmp_N`

- [x] Change format string in `flatten_expr.rs:388`
- [x] Grep to confirm no remaining `__flat_` references
- [x] `cargo build` + `cargo test`

### 2. SSA-style naming in BIR

- [x] 2.1 `FunctionId` Display: add `@` prefix (`cfg.rs`)
- [x] 2.2 `BlockId` Display: change `bb{}` → `%bb{}` (`cfg.rs`)
- [x] 2.3a `SsaName` Display: change `{base}_{version}` → `%v{version}` (`ops.rs`)
- [x] 2.3b SSA pass: replace per-variable `version_map` with single global counter (`ssa.rs`)
- [x] `cargo build` + `cargo test`
- [x] Run scanner on sample contract, inspect BIR output

### 3. BIR → FIR lowering

- [x] 3.1 Create `crates/scirs/src/fir/mod.rs` with re-exports
- [x] 3.2 Create `fir/ops.rs` — FIR ops (reuse BIR `OpKind` + `TailCall`)
- [x] 3.3 Create `fir/module.rs` — `Module`, `Function`, `Terminator`
- [x] 3.4 Add `pub mod fir;` to `scirs/src/lib.rs`
- [x] 3.5 Create `fir/lower/mod.rs` — BIR → FIR lowering orchestration
- [x] 3.6 Create `fir/lower/lift_blocks.rs` — core block-to-function lifting
  - [x] Compute live-in sets per block
  - [x] Eliminate phi nodes into function params
  - [x] Convert terminators to tail calls
  - [x] Name lifted functions `@Foo$N`
- [x] `cargo build` + `cargo test`
- [x] Run scanner, inspect FIR output

### 4. Utils modules (visit + fold)

- [x] 4.1 CIR `utils/mod.rs`
- [x] 4.2 CIR `utils/visit.rs` — per-variant visit methods
- [x] 4.3 CIR `utils/fold.rs` — per-variant fold methods
- [x] 4.4 Wire CIR utils into `cir/mod.rs`
- [x] 4.5 BIR `utils/mod.rs`
- [x] 4.6 BIR `utils/visit.rs` — per-variant visit methods
- [x] 4.7 BIR `utils/fold.rs` — per-variant fold methods
- [x] 4.8 Wire BIR utils into `bir/mod.rs`
- [x] 4.9 FIR `utils/mod.rs`
- [x] 4.10 FIR `utils/visit.rs` — per-variant visit methods
- [x] 4.11 FIR `utils/fold.rs` — per-variant fold methods
- [x] 4.12 Wire FIR utils into `fir/mod.rs`
- [x] `cargo build` + `cargo test`

### 5. IR verifier modules

- [x] 5.0 Add common `VerifyError` type
- [x] **SIR verifier** (`sir/verifier/`)
  - [x] 5.1 `type_well_formed` (Visit)
  - [x] 5.2 `scope_check` (Fold)
  - [x] 5.3 `spec_check` (Visit)
  - [x] 5.4 `no_orphan_dialect` (Visit)
  - [x] 5.5 `mod.rs` — orchestrate all SIR passes
- [x] **CIR verifier** (`cir/verifier/`)
  - [x] 5.6 `no_named_args` (Visit)
  - [x] 5.7 `no_using_for` (Visit)
  - [x] 5.8 `no_modifiers` (Visit)
  - [x] 5.9 `atomic_call_args` (Visit)
  - [x] 5.10 `no_ternary` (Visit)
  - [x] 5.11 `no_inheritance` (Visit)
  - [x] 5.12 `mod.rs` — orchestrate all CIR passes
- [x] **BIR verifier** (`bir/verifier/`)
  - [x] 5.13 `ssa_single_def` (Fold)
  - [x] 5.14 `ssa_use_def` (Fold)
  - [x] 5.15 `cfg_well_formed` (Visit)
  - [x] 5.16 `cfg_entry_exit` (Visit)
  - [x] 5.17 `phi_consistency` (Visit)
  - [x] 5.18 `op_id_unique` (Fold)
  - [x] 5.19 `mod.rs` — orchestrate all BIR passes
- [x] **FIR verifier** (`fir/verifier/`)
  - [x] 5.20 `no_phi` (Visit)
  - [x] 5.21 `tail_call_targets` (Visit)
  - [x] 5.22 `param_arity` (Fold)
  - [x] 5.23 `no_orphan_blocks` (Visit)
  - [x] 5.24 `terminal_completeness` (Visit)
  - [x] 5.25 `mod.rs` — orchestrate all FIR passes
- [x] `cargo build` + `cargo test`
- [ ] Unit tests with malformed IR for each pass

### 6. Relocate lowering modules

- [ ] 6.1 Move `cir/lower/` → `sir/lower/` (SIR→CIR lowering)
  - [ ] Move all 6 files
  - [ ] Add `pub mod lower;` to `sir/mod.rs`
  - [ ] Remove `pub mod lower;` from `cir/mod.rs`
  - [ ] Update all import paths (`crate::cir::lower` → `crate::sir::lower`)
- [ ] 6.2 Move `bir/lower/` → `cir/lower/` (CIR→BIR lowering)
  - [ ] Move all 5 files
  - [ ] Add `pub mod lower;` to `cir/mod.rs` (for CIR→BIR)
  - [ ] Remove `pub mod lower;` from `bir/mod.rs`
  - [ ] Update all import paths (`crate::bir::lower` → `crate::cir::lower`)
- [ ] 6.3 Move `fir/lower/` → `bir/lower/` (BIR→FIR lowering)
  - [ ] Move the lowering files
  - [ ] Add `pub mod lower;` to `bir/mod.rs` (for BIR→FIR)
  - [ ] Remove `pub mod lower;` from `fir/mod.rs`
  - [ ] Update all import paths
- [ ] `cargo build` + `cargo test`
- [ ] Grep for stale import paths

