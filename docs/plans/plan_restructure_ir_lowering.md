# Plan: Restructure IR Lowering — Move Semantic Passes to SIR → CIR

## Goal

Restructure the IR lowering pipeline so that **SIR stays close to the
source AST** and **CIR is the canonical, normalized form**.

Currently, 12 AST normalization passes run _before_ lowering to SIR,
meaning SIR already contains a heavily transformed program. This plan
moves the 5 semantic normalization passes out of the AST→SIR pipeline
and into the SIR→CIR lowering, where they belong.

## Current State

```
Solidity AST
  ├─ 12 AST normalization passes (run_passes)           ← all happen here
  ▼
SIR  (already normalized — not source-faithful)
  │
  ▼ cir::lower  (structural pass-through today)
CIR
  │
  ▼ bir::lower
BIR
```

The `run_passes` function in `frontend/src/solidity/lowering/lower.rs`
runs these passes in order:

| #  | Pass                  | Category        |
|----|-----------------------|-----------------|
| 1  | `unroll_tuples`       | Syntactic       |
| 2  | `rename_contracts`    | Multi-file      |
| 3  | `rename_vars`         | Multi-file      |
| 4  | `eliminate_using`     | **Semantic** ⬇️  |
| 5  | `rename_defs`         | Multi-file      |
| 6  | `eliminate_imports`   | Multi-file      |
| 7  | `merge_pragmas`       | Syntactic       |
| 8  | `resolve_inheritance` | **Semantic** ⬇️  |
| 9  | `rename_callees`      | Multi-file      |
| 10 | `eliminate_named_args` | **Semantic** ⬇️ |
| 11 | `eliminate_modifiers` | **Semantic** ⬇️  |
| 12 | `flatten_expr`        | **Semantic** ⬇️  |

## Proposed Pipeline

```
Solidity AST
  ├─ 7 structural/multi-file passes (run_passes)
  ▼
SIR  (source-faithful, preserves using/inheritance/modifiers/named-args)
  │
  ├─ 5 semantic normalization passes (cir::lower)
  ▼
CIR  (canonical, normalized)
  │
  ▼ bir::lower
BIR
```

---

## Classification

### Keep at AST Level (before AST → SIR)

These passes are required for **multi-file compilation** (cross-file
name resolution, import inlining) or are **trivial syntactic cleanups**:

| Pass               | Rationale                                                  |
|--------------------|------------------------------------------------------------|
| `unroll_tuples`    | Trivial syntactic cleanup — unwraps `(f)(x)` → `f(x)`     |
| `rename_contracts` | Disambiguates contract names across source units           |
| `rename_vars`      | Disambiguates variable names before import elimination     |
| `rename_defs`      | Disambiguates function/struct/enum/error/event definitions |
| `eliminate_imports` | Inlines imported symbols from other source units           |
| `merge_pragmas`    | Deduplicates pragmas after imports are merged              |
| `rename_callees`   | Resolves overloaded call sites to unique names             |

### Move to SIR → CIR Lowering

These passes perform **semantic normalization** that CIR explicitly
requires (documented in `scirs/src/cir/mod.rs`):

| Pass                  | CIR Invariant                                        |
|-----------------------|------------------------------------------------------|
| `eliminate_using`     | Using-for directives eliminated                      |
| `resolve_inheritance` | Inheritance resolved (no `parents` field)            |
| `eliminate_named_args`| Named arguments converted to positional              |
| `eliminate_modifiers` | Modifiers inlined into function bodies               |
| `flatten_expr`        | Expressions flattened (call args are atoms)           |

### Move to CIR → BIR (None)

No AST passes belong here. BIR lowering already handles CFG
construction, SSA, dialect lowering, and ICFG building.

---

## What Changes in SIR

To preserve source-level constructs that are currently eliminated before
reaching SIR, we need to extend SIR with:

### 1. Modifier Invocations on Functions

Currently, `eliminate_modifiers` inlines modifier bodies at the AST
level, so `sir::FunctionDecl` never sees modifiers. To preserve them:

```rust
// sir::defs.rs — FunctionDecl
pub struct FunctionDecl {
    // ... existing fields ...
    pub modifier_invocs: Vec<ModifierInvoc>,  // NEW
}

pub struct ModifierInvoc {
    pub name: String,
    pub args: Vec<Expr>,
    pub span: Option<Span>,
}
```

### 2. Named Arguments in Calls

Currently, `eliminate_named_args` converts named args to positional at
the AST level. To preserve them, add a named variant to `CallExpr`:

```rust
// sir::exprs.rs — CallExpr
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: CallArgs,       // CHANGED from Vec<Expr>
    pub ty: Type,
    pub span: Option<Span>,
}

pub enum CallArgs {
    Positional(Vec<Expr>),
    Named(Vec<NamedArg>),
}

pub struct NamedArg {
    pub name: String,
    pub value: Expr,
}
```

### 3. Using-For Directives

Currently, `eliminate_using` removes `using` directives at the AST
level. To preserve them in SIR, add a new member declaration variant:

```rust
// sir::defs.rs — MemberDecl
pub enum MemberDecl {
    // ... existing variants ...
    UsingFor(UsingForDecl),  // NEW
}

pub struct UsingForDecl {
    pub library: String,         // library or function name
    pub target_type: Option<Type>, // None = using for *
    pub span: Option<Span>,
}
```

### 4. Inheritance on Contracts

`sir::ContractDecl` already has a `parents: Vec<String>` field. It's
currently always empty because `resolve_inheritance` runs at the AST
level. After the move, this field will be populated.

---

## Implementation Steps

### Step 1: Extend SIR Types

Add the new fields/variants listed above to SIR types in
`scirs/src/sir/defs.rs` and `scirs/src/sir/exprs.rs`.

**Files to modify:**
- `scirs/src/sir/defs.rs` — add `ModifierInvoc`, `UsingForDecl`, `MemberDecl::UsingFor`
- `scirs/src/sir/exprs.rs` — add `CallArgs`, `NamedArg`, change `CallExpr.args`

### Step 2: Update AST → SIR Lowering

Update `Lowerer` in `frontend/src/solidity/lowering/lower.rs` to produce
the new SIR constructs instead of expecting them to be eliminated:

- **Lower modifier invocations** into `FunctionDecl.modifier_invocs`
  instead of failing with "must be eliminated".
- **Lower named arguments** into `CallArgs::Named` instead of failing.
- **Lower `using` directives** into `MemberDecl::UsingFor` instead of
  failing.
- **Populate `ContractDecl.parents`** from `base_contracts` instead of
  failing with "must be eliminated".

**Files to modify:**
- `frontend/src/solidity/lowering/lower.rs` — update `lower_contract_def`,
  `lower_func_def`, `lower_call_args_exprs`, `lower_contract_elem`,
  `lower_source_unit`

### Step 3: Remove 5 Passes from `run_passes`

Remove calls to `eliminate_using`, `resolve_inheritance`,
`eliminate_named_args`, `eliminate_modifiers`, and `flatten_expr` from
`run_passes` in `lower.rs`.

The updated `run_passes` becomes:

```rust
pub fn run_passes(source_units: &[ast::SourceUnit]) -> Vec<ast::SourceUnit> {
    let source_units = unroll_unary_tuple(source_units);

    let env = ast::NamingEnv::new();
    let (source_units, env) = rename_contracts(&source_units, Some(&env));
    let (source_units, env) = rename_vars(&source_units, Some(&env));
    let (source_units, env) = rename_defs(&source_units, Some(&env));
    let source_units = eliminate_import(&source_units);
    let source_units = merge_pragmas(&source_units);
    let (source_units, _) = rename_callees(&source_units, Some(&env));

    unroll_unary_tuple(&source_units)
}
```

**Files to modify:**
- `frontend/src/solidity/lowering/lower.rs` — update `run_passes`
- `frontend/src/solidity/lowering/mod.rs` — optionally remove unused `pub use` re-exports

### Step 4: Implement SIR-Level Semantic Passes in `cir::lower`

Reimplement the 5 semantic passes to operate on SIR types within
`cir::lower`. Each pass transforms `sir::Module` → `sir::Module`
(intermediate normalization), and the final structural conversion
produces `cir::CanonModule`.

**Recommended order** (simplest first):

1. **`elim_named_args`** — walk `CallExpr`, reorder `Named` args to
   `Positional` using function parameter names from `FunctionDecl`.
2. **`elim_using`** — walk contracts, collect `UsingFor` directives,
   rewrite method-style calls on matching types.
3. **`resolve_inheritance`** — linearize `parents` (C3), merge members
   from parent contracts, resolve `super` calls.
4. **`elim_modifiers`** — find modifier definitions in `Dialect` member
   decls, inline their bodies into functions that reference them.
5. **`flatten_expr`** — introduce `LocalVar` temporaries so all call
   arguments become atoms.

**Files to create:**
- `scirs/src/cir/lower/elim_named_args.rs`
- `scirs/src/cir/lower/elim_using.rs`
- `scirs/src/cir/lower/resolve_inheritance.rs`
- `scirs/src/cir/lower/elim_modifiers.rs`
- `scirs/src/cir/lower/flatten_expr.rs`

**Files to modify:**
- `scirs/src/cir/lower/mod.rs` — orchestrate the 5 passes before the
  structural conversion

The `lower_module` function will become:

```rust
pub fn lower_module(sir_module: &sir::Module) -> Result<CanonModule, CirLowerError> {
    // Phase 1: Semantic normalization (SIR → SIR)
    let module = elim_named_args::run(sir_module)?;
    let module = elim_using::run(&module)?;
    let module = resolve_inheritance::run(&module)?;
    let module = elim_modifiers::run(&module)?;
    let module = flatten_expr::run(&module)?;

    // Phase 2: Structural conversion (SIR → CIR)
    let mut lowerer = CirLowerer::new();
    lowerer.lower_module(&module)
}
```

### Step 5: Add SIR Map/Fold Utilities (if needed)

The 5 passes need to traverse and transform SIR trees. Check if the
existing `sir::utils::Map`/`sir::utils::Fold`/`sir::utils::Visit`
utilities are sufficient; extend them if needed.

**Files to potentially modify:**
- `scirs/src/sir/utils/` — add or extend `Map`/`Fold` trait methods

### Step 6: Remove Dead AST-Level Pass Code

After Step 4 is complete and verified, the 5 pass files in
`frontend/src/solidity/lowering/` become dead code. Remove them:

- `frontend/src/solidity/lowering/eliminate_using.rs`
- `frontend/src/solidity/lowering/resolve_inheritance.rs`
- `frontend/src/solidity/lowering/eliminate_named_args.rs`
- `frontend/src/solidity/lowering/eliminate_modifiers.rs`
- `frontend/src/solidity/lowering/flatten_expr.rs`

Also remove:
- `frontend/src/solidity/lowering/flatten_names.rs` — only used by
  test code for `resolve_inheritance`
- `frontend/src/solidity/lowering/substitution.rs` — only used by
  `eliminate_modifiers` and `eliminate_imports`; may still be needed
  by `eliminate_imports`, check first

**Files to modify:**
- `frontend/src/solidity/lowering/mod.rs` — remove unused module
  declarations and re-exports

---

## Execution Order

The steps have **dependencies**:

```
Step 1 (extend SIR types)
  └── Step 2 (update AST→SIR lowering to populate new fields)
        └── Step 3 (remove passes from run_passes)
              └── Step 4 (implement SIR-level passes in cir::lower)
                    ├── Step 5 (extend SIR utilities if needed)
                    └── Step 6 (remove dead AST-level code)
```

> [!IMPORTANT]
> Steps 1–3 can be done first as a batch. This will temporarily break the
> pipeline because CIR lowering won't yet handle the new SIR constructs.
> To avoid breakage, implement Step 4's passes **incrementally**: move
> one pass at a time (remove from `run_passes` + add to `cir::lower`),
> verify, then proceed to the next.

### Recommended Incremental Approach

For each pass (in order: named_args → using → inheritance → modifiers →
flatten_expr):

1. Extend SIR types if needed for this pass
2. Update `Lowerer` to produce the new SIR constructs
3. Remove the pass from `run_passes`
4. Implement the SIR-level pass in `cir::lower`
5. Verify compilation and tests pass

---

## Verification Plan

### Existing Tests

Each AST-level pass has unit tests in its own file:

```bash
# Run all lowering tests
cargo test -p frontend -- solidity::lowering

# Individual pass tests
cargo test -p frontend -- solidity::lowering::eliminate_modifiers
cargo test -p frontend -- solidity::lowering::eliminate_named_args
cargo test -p frontend -- solidity::lowering::eliminate_using
cargo test -p frontend -- solidity::lowering::resolve_inheritance
cargo test -p frontend -- solidity::lowering::flatten_expr
cargo test -p frontend -- solidity::lowering::eliminate_imports
cargo test -p frontend -- solidity::lowering::rename_callees
```

### End-to-End Compilation

```bash
# Build everything
cargo build

# Run the compile subcommand with --debug to print all IR layers
cargo run -p verazt -- compile --debug <test-contract.sol>

# Compare SIR and CIR output before and after changes
# SIR should now show source-faithful constructs (modifiers, named
# args, using, inheritance) that were previously eliminated
# CIR should be identical to the current output
```

### New Tests

After moving each pass to `cir::lower`, write corresponding tests in
`scirs/src/cir/lower/`:
- Each test constructs a `sir::Module` with the relevant construct
  (e.g., named args, modifier invocations, using-for directives)
- Runs `cir::lower::lower_module`
- Asserts the output `CanonModule` matches expected canonical form

### Regression

```bash
# Full test suite
cargo test

# Integration tests
cargo run -p verazt -- compile tests/solidity/*.sol
```
