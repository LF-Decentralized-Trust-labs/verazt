# Plan: Canonical IR (CIR)

## Goal

Introduce a **Canonical IR (CIR)** layer between SIR and AIR that:

1. Gives semantic normalizations a proper IR-level home.
2. Makes the AST source-faithful (no semantic transforms on the AST).
3. Removes the duplicate modifier expansion that currently runs in both
   `frontend/ast/normalize` and `mlir/air/lower/modifier_expand.rs`.
4. Gives the AIR lowering a compile-time guarantee that its input is
   already normalized (it receives `cir::Module`, not `sir::Module`).

## Current Problems

| Problem | Location |
|---------|----------|
| `resolve_inheritance` — semantic transform on AST | `frontend/ast/normalize/resolve_inheritance.rs` |
| `eliminate_modifier_invocs` — inlines modifiers on AST | `frontend/ast/normalize/elim_func_modifier.rs` |
| `modifier_expand` — inlines modifiers again on SIR | `mlir/air/lower/modifier_expand.rs` ← **duplicate** |
| `flatten_expr` — introduces temporaries on AST | `frontend/ast/normalize/flatten_expr.rs` |
| `eliminate_named_args` — semantic simplification on AST | `frontend/ast/normalize/elim_named_args.rs` |
| `eliminate_using_directives` — semantic simplification on AST | `frontend/ast/normalize/elim_using_directives.rs` |
| `ast/normalize` is a separate caller-visible step | `frontend/src/*/ast/normalize/` — callers must invoke it manually |
| AST is not source-faithful | All of the above |

## Proposed Pipeline

```
Solidity/Vyper source
        │
        ▼ parse only — no transforms on the AST
      AST  (source-faithful)
        │
        ▼ sir::lower (frontend/src/*/lower/)
        │   internally runs: all former ast/normalize passes → lower to SIR
      SIR  (language-neutral, dialect-extensible)
        │
        ▼ cir::lower (mlir/src/cir/lower/)
      CIR  (canonical, normalized, still structured)
        │
        ▼ air::lower (mlir/src/air/lower/)
      AIR  (graph/SSA, analysis engine input)
```

---

## CIR Design

### Where CIR Lives

**No new crate needed.** CIR definitions go into the `mlir` crate as a new
module `mlir::cir`, following the same pattern as `mlir::sir` and `mlir::air`.

The SIR → CIR lowering passes go into `mlir/src/cir/lower/`, co-located with
the target IR (same pattern as `mlir/src/air/lower/`).

```
mlir/src/
├── sir/       (existing)
├── air/       (existing)
└── cir/       (new)
    ├── mod.rs
    ├── defs.rs
    ├── exprs.rs
    ├── stmts.rs
    ├── module.rs
    └── lower/
        ├── mod.rs              # orchestrates the steps below
        ├── resolve_inheritance.rs
        ├── elim_modifiers.rs
        ├── flatten_expr.rs
        ├── elim_named_args.rs
        └── elim_using.rs
```

### `sir::lower` Structure

`sir::lower` (`frontend/src/*/lower/`) becomes a self-contained pipeline that
takes a raw, source-faithful AST and produces SIR. Callers no longer call a
separate normalize step. The `ast/normalize/` module is deleted entirely.

```
frontend/src/solidity/lower/
├── mod.rs              # public API: lower_source_unit(ast) -> sir::Module
├── lower.rs            # core AST→SIR lowering (former ir_gen.rs)
└── normalize/          # all former ast/normalize passes, now internal
    ├── mod.rs          # run_passes — called by mod.rs before lower.rs
    ├── elim_func_modifier.rs
    ├── elim_import_directives.rs
    ├── elim_named_args.rs
    ├── elim_using_directives.rs
    ├── flatten_expr.rs
    ├── flatten_name_index.rs
    ├── merge_pragmas.rs
    ├── rename_callees.rs
    ├── rename_contracts.rs
    ├── rename_defs.rs
    ├── rename_vars.rs
    ├── resolve_inheritance.rs
    ├── substitution.rs
    ├── unroll_unary_tuple.rs
    └── utils.rs

frontend/src/vyper/lower/
├── mod.rs
├── lower.rs
└── normalize/          # all former vyper/ast/normalize passes
    ├── mod.rs
    ├── flatten_expr.rs
    ├── rename_defs.rs
    └── rename_vars.rs
```

The public signature of `lower_source_unit` does not change — only its
internals now include the normalize step that callers previously had to invoke
manually.

### CIR Type Definitions

CIR reuses SIR types where there is no structural difference (`Type`, `Lit`,
`Attr`, `Span`, `FuncSpec`, dialect extension points). It introduces distinct
types only where normalization changes the structure.

#### `cir::ContractDecl` vs `sir::ContractDecl`

| Field | SIR | CIR |
|-------|-----|-----|
| `name` | `String` | `String` |
| `parents` | `Vec<String>` | **removed** — inheritance resolved |
| `attrs` | `Vec<Attr>` | `Vec<Attr>` |
| `members` | `Vec<MemberDecl>` | `Vec<CanonMemberDecl>` |
| `span` | `Option<Span>` | `Option<Span>` |

#### `cir::CanonMemberDecl` vs `sir::MemberDecl`

| Variant | SIR | CIR |
|---------|-----|-----|
| `Storage` | ✓ | ✓ |
| `Function` | ✓ | ✓ (body is `Vec<CanonStmt>`, never `None` after normalization) |
| `Modifier` | ✓ (via `DialectMemberDecl::evm::Modifier`) | **removed** — inlined into functions |
| `TypeAlias` | ✓ | ✓ |
| `GlobalInvariant` | ✓ | ✓ |
| `Dialect` | ✓ | ✓ (modifier variant excluded) |

#### `cir::CanonExpr` vs `sir::Expr`

| Variant | SIR | CIR |
|---------|-----|-----|
| `Var`, `Lit`, `BinOp`, `UnOp` | ✓ | ✓ |
| `IndexAccess`, `FieldAccess` | ✓ | ✓ |
| `FunctionCall` | nested args allowed | args must be atoms (Var or Lit) — no nested calls |
| `TypeCast` | ✓ | ✓ |
| `Ternary` | ✓ | **removed** — lowered to `if` statement |
| `Tuple` | ✓ | **removed** — unrolled |
| `Old`, `Result`, `Forall`, `Exists` | ✓ (spec only) | ✓ (spec only) |
| `Dialect` | ✓ | ✓ |

The key invariant of `CanonExpr`: **every call argument is an atom**, enforced
by flattening compound expressions into preceding `LocalVar` statements.

---

## Normalization Passes (SIR → CIR)

Orchestrated by `cir::lower::lower_module(sir: &sir::Module) -> Result<cir::Module>`.

| Step | Pass | Source (move from) |
|------|------|--------------------|
| 1 | `elim_using` — remove using-for directives | `frontend/ast/normalize/elim_using_directives.rs` |
| 2 | `elim_named_args` — convert named args to positional | `frontend/ast/normalize/elim_named_args.rs` |
| 3 | `resolve_inheritance` — inline parent members, clear `parents` | `frontend/ast/normalize/resolve_inheritance.rs` |
| 4 | `elim_modifiers` — inline modifier bodies into functions | `frontend/ast/normalize/elim_func_modifier.rs` + `mlir/air/lower/modifier_expand.rs` (merge, delete both) |
| 5 | `flatten_expr` — introduce temporaries, atomise call args | `frontend/ast/normalize/flatten_expr.rs` |

Each pass takes a `sir::Module` or intermediate CIR and returns a `cir::Module`
(or intermediate form). The final output of step 5 is a fully-formed `cir::Module`.

---

## Changes to AIR Lowering

`mlir/src/air/lower/mod.rs` currently runs **5 steps**:

```
Step 1: EVM Modifier Expansion   ← DELETE (moved to CIR)
Step 2: CFG Construction         ← keep, renumber to Step 1
Step 3: SSA Renaming             ← keep, renumber to Step 2
Step 4: Dialect Lowering         ← keep, renumber to Step 3
Step 5: ICFG + Alias + Taint     ← keep, renumber to Step 4
```

`lower_module` signature changes from:
```rust
pub fn lower_module(cir: &sir::Module) -> Result<air::Module>
```
to:
```rust
pub fn lower_module(cir: &cir::Module) -> Result<air::Module>
```

`mlir/src/air/lower/modifier_expand.rs` — **delete**.

---

## AST Normalize Cleanup

`frontend/src/solidity/ast/normalize/` and `frontend/src/vyper/ast/normalize/`
are **deleted entirely**. All passes move to `sir::lower/normalize/` as an
internal implementation detail.

| Pass | Disposition |
|------|------------|
| `unroll_unary_tuple` | **move to `sir::lower/normalize/`** |
| `merge_pragmas` | **move to `sir::lower/normalize/`** |
| `eliminate_import` | **move to `sir::lower/normalize/`** |
| `rename_vars` | **move to `sir::lower/normalize/`** |
| `rename_defs` | **move to `sir::lower/normalize/`** |
| `rename_contracts` | **move to `sir::lower/normalize/`** |
| `rename_callees` | **move to `sir::lower/normalize/`** |
| `flatten_name_index` | **move to `sir::lower/normalize/`** |
| `resolve_inheritance` | **move to `sir::lower/normalize/`**, then also implemented in `cir::lower` on SIR types¹ |
| `elim_func_modifier` | **move to `sir::lower/normalize/`**, then merged into `cir::lower/elim_modifiers` on SIR¹ |
| `flatten_expr` | **move to `sir::lower/normalize/`**, then reimplemented in `cir::lower` on SIR types¹ |
| `elim_named_args` | **move to `sir::lower/normalize/`**, then reimplemented in `cir::lower` on SIR types¹ |
| `elim_using_directives` | **move to `sir::lower/normalize/`**, then reimplemented in `cir::lower` on SIR types¹ |

> ¹ These semantic passes run at the AST level inside `sir::lower` initially.
> Once `cir::lower` is implemented with SIR-level equivalents, the AST-level
> versions inside `sir::lower/normalize/` become redundant and can be removed,
> leaving `sir::lower/normalize/` with only the structural cleanup passes.

---

## Implementation Steps

1. **Move all AST normalize passes into `sir::lower`** — for both Solidity and
   Vyper, create `lower/normalize/` submodule, move all files from
   `ast/normalize/` into it, and call `normalize::run_passes` internally from
   `lower::lower_source_unit`. Delete `ast/normalize/` from both languages.
   Update all callers that previously called `ast::normalize::run_passes`
   manually (e.g. `vyper::compile_file`, `verazt::compile`).

2. **Define CIR types** in `mlir/src/cir/` — `module.rs`, `defs.rs`, `exprs.rs`, `stmts.rs`.
   Reuse SIR types where possible via re-exports or type aliases.

3. **Implement `cir::lower`** — implement each semantic normalization pass on
   SIR types to produce `cir::*` types.
   - Start with the structurally simplest: `elim_using`, `elim_named_args`.
   - Then `resolve_inheritance`, `elim_modifiers`, `flatten_expr`.

4. **Update `air::lower`** — change input type to `cir::Module`, delete Step 1
   (`modifier_expand`), delete `modifier_expand.rs`.

5. **Update scanner pipeline** — thread the new `cir::lower` step between
   `sir::lower` and `air::lower` in `scanner/src/pipeline/engine.rs`.

6. **Remove redundant AST-level semantic passes** — once `cir::lower` covers
   `resolve_inheritance`, `elim_modifiers`, `flatten_expr`, `elim_named_args`,
   and `elim_using`, remove their counterparts from
   `sir::lower/normalize/` in both Solidity and Vyper.

7. **Update `plan_structure.md`** — add `mlir/src/cir/` and the updated
   `frontend/src/*/lower/normalize/` structure.
