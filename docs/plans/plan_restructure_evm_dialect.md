# Plan: Restructure Dialect Module into Per-Dialect Folders

## Goal

Convert the flat file layout in `crates/scirs/src/sir/dialect/` into a
**per-dialect sub-module folder** structure so that each dialect's types,
statements, expressions, declarations, and extension traits are grouped
together rather than scattered as sibling files.

Additionally, unify the scattered source location types (`Span`, `Loc`) into
a single **`common::Loc`** type shared across all crates.

---

## Current Layout

```
dialect/
├── mod.rs            # Dialect enums (DialectExpr, DialectStmt, …) + Display impls
├── evm.rs            # EvmType, EvmExpr, EvmStmt, EvmMemberDecl, EvmDecl  (480 lines)
├── evm_ext.rs        # EvmFunctionExt, EvmCallExt, EvmStorageExt traits   (103 lines)
├── anchor.rs         # AnchorType, AnchorExpr, AnchorStmt, …              (253 lines)
├── move_lang.rs      # MoveType, MoveExpr, MoveStmt, …                   (212 lines)
└── spec_dialect.rs   # SpecType                                           ( 31 lines)
```

**Pain points:**
- EVM-related code is split across `evm.rs` and `evm_ext.rs` with no
  logical grouping.
- As dialects grow, single-file modules become unwieldy.
- Adding new per-dialect files (e.g., queries, lowering helpers) would clutter
  the flat namespace further.
- Three separate source location types exist with no single source of truth:
  - `scirs::sir::loc::Span` — byte offsets (`start: u32, end: u32`)
  - `frontend::solidity::ast::loc::Loc` — line/col (`start_line, start_col, end_line, end_col`)
  - `frontend::vyper::ast::loc::Loc` — line/col (`lineno, col_offset, end_lineno, end_col_offset`)

---

## Proposed Layout

```
dialect/
├── mod.rs            # Dialect enums + Display impls + re-exports
├── evm/
│   ├── mod.rs        # Re-exports from child modules
│   ├── types.rs      # EvmType
│   ├── exprs.rs      # EvmExpr + Display + typ()
│   ├── stmts.rs      # EvmStmt, EvmCatchClause + Display
│   ├── decls.rs      # EvmMemberDecl, EvmDecl + Display
│   └── ext.rs        # EvmFunctionExt, EvmCallExt, EvmStorageExt
├── anchor/
│   ├── mod.rs        # Re-exports from child modules
│   ├── types.rs      # AnchorType
│   ├── exprs.rs      # AnchorExpr + Display
│   ├── stmts.rs      # AnchorStmt + Display
│   └── decls.rs      # AnchorMemberDecl, AnchorAccountField, AnchorDecl
├── move_lang/
│   ├── mod.rs        # Re-exports from child modules
│   ├── types.rs      # MoveType
│   ├── exprs.rs      # MoveExpr + Display
│   ├── stmts.rs      # MoveStmt + Display
│   └── decls.rs      # MoveMemberDecl, MoveTypeParam, MoveDecl
└── spec/
    └── mod.rs        # SpecType (small enough to stay as a single file)
```

---

## Detailed Changes

### 0. Unify `Span` → `common::Loc`

Create a single source location type in the `common` crate, rename `Span`
to `Loc`, and have all crates depend on it.

#### `common/src/loc.rs` (NEW)

```rust
/// A source span representing a range in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Loc {
    pub start: u32,
    pub end: u32,
}

impl Loc {
    pub fn new(start: u32, end: u32) -> Self {
        Loc { start, end }
    }
}

impl std::fmt::Display for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
```

#### Changes

- Add `pub mod loc;` to `common/src/lib.rs`
- Delete `scirs/src/sir/loc.rs`
- In `scirs`: replace all `use crate::sir::loc::Span` → `use common::loc::Loc`
  and rename field types from `Span` to `Loc` throughout
- In `frontend`: replace frontend-specific `Loc` types with `common::Loc`
  (Solidity and Vyper frontends will convert from their AST locations to
  `common::Loc` at lowering time)

**Files importing `sir::loc::Span` (9 files in `scirs`):**
- `sir/defs.rs`, `sir/stmts.rs`, `sir/exprs.rs`, `sir/lits.rs`
- `sir/dialect/evm.rs`, `sir/dialect/anchor.rs`
- `cir/defs.rs`, `cir/exprs.rs`, `cir/stmts.rs`

### 1. `dialect/mod.rs`

Update the child module declarations:

```diff
-pub mod anchor;
-pub mod evm;
-pub mod evm_ext;
-pub mod move_lang;
-pub mod spec_dialect;
+pub mod anchor;       // now a directory module
+pub mod evm;          // now a directory module
+pub mod move_lang;    // now a directory module
+pub mod spec;         // renamed from spec_dialect

-pub use evm_ext::{EvmCallExt, EvmFunctionExt, EvmStorageExt};
+pub use evm::{EvmCallExt, EvmFunctionExt, EvmStorageExt};
```

The body of `mod.rs` (enum definitions, Display impls) stays **unchanged**
because all existing type paths like `evm::EvmExpr` remain valid as long as
each sub-module re-exports everything from its `mod.rs`.

### 2. EVM sub-module (`dialect/evm/`)

| New file          | Content moved from          |
| ----------------- | --------------------------- |
| `evm/types.rs`    | `EvmType` + Display impl    |
| `evm/exprs.rs`    | `EvmExpr` + Display + `typ()` |
| `evm/stmts.rs`    | `EvmStmt`, `EvmCatchClause` + Display |
| `evm/decls.rs`    | `EvmMemberDecl`, `EvmDecl` + Display |
| `evm/ext.rs`      | Entire `evm_ext.rs` (extension traits) |

#### `evm/mod.rs`

```rust
mod types;
mod exprs;
mod stmts;
mod decls;
mod ext;

pub use types::*;
pub use exprs::*;
pub use stmts::*;
pub use decls::*;
pub use ext::*;
```

#### Refactor `EvmExpr` to enum-of-structs

Every variant becomes a dedicated struct wrapping its fields **plus a
`loc: Loc` field** (from `common::Loc`).  This applies universally —
even former unit variants now carry `loc`.

**Before:**

```rust
pub enum EvmExpr {
    MsgSender,
    Convert { expr: Box<Expr>, to: Type },
    Delegatecall { target: Box<Expr>, data: Box<Expr>, span: Option<Span> },
    // …
}
```

**After:**

```rust
use common::loc::Loc;

/// `evm.msg_sender()` — `msg.sender`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmMsgSender {
    pub loc: Loc,
}

/// Vyper's `convert(x, T)` — explicit type cast builtin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmConvert {
    pub expr: Box<Expr>,
    pub to: Type,
    pub loc: Loc,
}

/// `evm.delegatecall(target, data)` — low-level delegatecall.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmDelegatecall {
    pub target: Box<Expr>,
    pub data: Box<Expr>,
    pub loc: Loc,
}

pub enum EvmExpr {
    MsgSender(EvmMsgSender),
    Convert(EvmConvert),
    Delegatecall(EvmDelegatecall),
    // …
}
```

**Naming convention:** `Evm` + variant name (e.g. `EvmMsgSender`,
`EvmConvert`, `EvmSlice`, `EvmRawCall`, `EvmDelegatecall`, etc.).
The old `span: Option<Span>` fields are replaced by `loc: Loc`.

Apply the same enum-of-structs refactoring to **all** dialect enums —
every variant in every enum gets a struct with a `loc: Loc` field:

| Enum              | All variants extracted as structs |
| ----------------- | --------------------------------- |
| `EvmExpr`         | All 39 variants |
| `EvmStmt`         | `EmitEvent`, `TryCatch`, `Placeholder`, `Selfdestruct` |
| `EvmMemberDecl`   | `EventDef`, `ModifierDef`, `ErrorDef`, `EnumDef`, `StructDef` |
| `AnchorExpr`      | All 7 variants |
| `AnchorStmt`      | `EmitEvent` |
| `AnchorMemberDecl`| `AccountsContext`, `AccountStruct`, `EventStruct`, `ErrorCode` |
| `MoveExpr`        | All 8 variants |
| `MoveStmt`        | `Abort`, `SpecBlock` |
| `MoveMemberDecl`  | `StructDef`, `SpecFun`, `FriendDecl` |

### 3. Anchor sub-module (`dialect/anchor/`)

| New file            | Content moved from |
| ------------------- | -------------------- |
| `anchor/types.rs`   | `AnchorType`         |
| `anchor/exprs.rs`   | `AnchorExpr`         |
| `anchor/stmts.rs`   | `AnchorStmt`         |
| `anchor/decls.rs`   | `AnchorMemberDecl`, `AnchorAccountField`, `AnchorDecl` |

#### `anchor/mod.rs`

```rust
mod types;
mod exprs;
mod stmts;
mod decls;

pub use types::*;
pub use exprs::*;
pub use stmts::*;
pub use decls::*;
```

#### Refactor Anchor enums to enum-of-structs

Same pattern as EVM — every variant becomes a struct with `loc: Loc`.

**`AnchorExpr` example (before → after):**

```rust
// Before
pub enum AnchorExpr {
    AccountLoad(Box<Expr>),
    Cpi { program: Box<Expr>, accounts: Vec<Expr>, data: Box<Expr> },
    // …
}

// After
pub struct AnchorAccountLoad {
    pub expr: Box<Expr>,
    pub loc: Loc,
}
pub struct AnchorCpi {
    pub program: Box<Expr>,
    pub accounts: Vec<Expr>,
    pub data: Box<Expr>,
    pub loc: Loc,
}
pub enum AnchorExpr {
    AccountLoad(AnchorAccountLoad),
    Cpi(AnchorCpi),
    // …
}
```

Naming convention: `Anchor` + variant name → `AnchorAccountLoad`,
`AnchorAccountLoadMut`, `AnchorSignerKey`, `AnchorOk`, `AnchorCpi`,
`AnchorSystemTransfer`, `AnchorTokenTransfer`, `AnchorFindProgramAddress`.

`AnchorStmt::EmitEvent` → `AnchorEmitEvent { event, fields, loc }`.

`AnchorMemberDecl` variants → `AnchorAccountsContext`,
`AnchorAccountStruct`, `AnchorEventStruct`, `AnchorErrorCode`.

### 4. Move sub-module (`dialect/move_lang/`)

| New file              | Content moved from |
| --------------------- | -------------------- |
| `move_lang/types.rs`  | `MoveType`           |
| `move_lang/exprs.rs`  | `MoveExpr`           |
| `move_lang/stmts.rs`  | `MoveStmt`           |
| `move_lang/decls.rs`  | `MoveMemberDecl`, `MoveTypeParam`, `MoveDecl` |

#### `move_lang/mod.rs`

```rust
mod types;
mod exprs;
mod stmts;
mod decls;

pub use types::*;
pub use exprs::*;
pub use stmts::*;
pub use decls::*;
```

#### Refactor Move enums to enum-of-structs

Same pattern — every variant becomes a struct with `loc: Loc`.

**`MoveExpr` example (before → after):**

```rust
// Before
pub enum MoveExpr {
    BorrowGlobal { ty: Type, addr: Box<Expr> },
    SignerAddress(Box<Expr>),
    // …
}

// After
pub struct MoveBorrowGlobal {
    pub ty: Type,
    pub addr: Box<Expr>,
    pub loc: Loc,
}
pub struct MoveSignerAddress {
    pub expr: Box<Expr>,
    pub loc: Loc,
}
pub enum MoveExpr {
    BorrowGlobal(MoveBorrowGlobal),
    SignerAddress(MoveSignerAddress),
    // …
}
```

Naming convention: `Move` + variant name → `MoveBorrowGlobal`,
`MoveBorrowGlobalMut`, `MoveExists`, `MoveSignerAddress`, `MoveMoveTo`,
`MoveMoveFrom`, `MoveWriteRef`, `MoveGhostVar`.

`MoveStmt` variants → `MoveAbort { expr, loc }`,
`MoveSpecBlock { assertions, loc }`.

`MoveMemberDecl` variants → `MoveStructDef`, `MoveSpecFun`,
`MoveFriendDecl`.

### 5. Spec sub-module (`dialect/spec/`)

Rename `spec_dialect.rs` → `spec/mod.rs`. Content stays unchanged.

### 6. Downstream Import Updates

Because each `<dialect>/mod.rs` re-exports with `pub use *`, the only
import paths that actually change are:

| Old path                           | New path                        | Affected crates    |
| ---------------------------------- | ------------------------------- | ------------------ |
| `dialect::evm_ext::EvmCallExt`     | `dialect::evm::EvmCallExt`      | `analyzer` (3 files) |
| `dialect::evm_ext::EvmFunctionExt` | `dialect::evm::EvmFunctionExt`  | `analyzer` (5 files) |
| `dialect::evm_ext::EvmStorageExt`  | `dialect::evm::EvmStorageExt`   | `analyzer` (2 files) |
| `dialect::spec_dialect::SpecType`  | `dialect::spec::SpecType`       | `dialect/mod.rs`     |

All other paths (`dialect::evm::EvmExpr`, `dialect::anchor::AnchorMemberDecl`,
etc.) remain unchanged.

**Affected downstream files (exhaustive list):**

Within **`scirs`** (internal):
- `sir/dialect/mod.rs` — module declarations + `spec_dialect` → `spec`
- `sir/lower/elim_modifiers.rs` — no change
- `sir/lower/mod.rs` — no change
- `sir/lower/elim_named_args.rs` — no change
- `sir/verifier/type_well_formed.rs` — no change

Within **`analyzer`** — update `evm_ext` → `evm` in import paths:
- `detectors/sir/reentrancy.rs`
- `detectors/sir/cei_violation.rs`
- `detectors/sir/missing_access_control.rs`
- `detectors/sir/sir_missing_access_control.rs`
- `detectors/sir/missing_modifies.rs`
- `detectors/sir/unchecked_call.rs`
- `detectors/sir/uninitialized.rs`
- `detectors/sir/constant_state_var.rs`

Within **`frontend`**:
- No changes needed (`dialect::evm::*` stays same)

---

## Verification Plan

### Automated

```bash
cargo build 2>&1
cargo test 2>&1
```

No new tests needed — this is a structural refactor with no behavior changes.

---

## Implementation Checklist

### Phase 0: Unify `Span` → `common::Loc`

- [x] Create `common/src/loc.rs` with `Loc` struct
- [x] Add `pub mod loc;` to `common/src/lib.rs`
- [x] Delete `scirs/src/sir/loc.rs`
- [x] Update `scirs` imports: `crate::sir::loc::Span` → `common::loc::Loc` (9 files)
- [x] Rename all `Span` field types/usages to `Loc` across `scirs`
- [x] Update `frontend` to use `common::Loc` instead of local `Loc` types
- [x] `cargo build` — verify clean build after rename

### Phase 1: Create EVM sub-module

- [x] Create `dialect/evm/` directory
- [x] Create `evm/types.rs` — move `EvmType` + Display from `evm.rs`
- [x] Create `evm/exprs.rs` — move `EvmExpr` + Display + `typ()` from `evm.rs`
  - [x] Refactor `EvmExpr` to enum-of-structs (extract structs for 3+ field variants)
  - [x] Update Display + `typ()` impls for new pattern
  - [x] Update all pattern matches on `EvmExpr` in the codebase
- [x] Create `evm/stmts.rs` — move `EvmStmt`, `EvmCatchClause` + Display from `evm.rs`
  - [x] Refactor `EvmStmt` to enum-of-structs
- [x] Create `evm/decls.rs` — move `EvmMemberDecl`, `EvmDecl` + Display from `evm.rs`
  - [x] Refactor `EvmMemberDecl` to enum-of-structs
- [x] Create `evm/ext.rs` — move entire `evm_ext.rs` content
- [x] Create `evm/mod.rs` — re-export all child modules
- [x] Delete old `dialect/evm.rs`
- [x] Delete old `dialect/evm_ext.rs`

### Phase 2: Create Anchor sub-module

- [x] Create `dialect/anchor/` directory
- [x] Create `anchor/types.rs` — move `AnchorType` from `anchor.rs`
- [x] Create `anchor/exprs.rs` — move `AnchorExpr` from `anchor.rs`
  - [x] Refactor `AnchorExpr` to enum-of-structs
- [x] Create `anchor/stmts.rs` — move `AnchorStmt` from `anchor.rs`
- [x] Create `anchor/decls.rs` — move `AnchorMemberDecl`, `AnchorAccountField`, `AnchorDecl` from `anchor.rs`
  - [x] Refactor `AnchorMemberDecl` to enum-of-structs
- [x] Create `anchor/mod.rs` — re-export all child modules
- [x] Delete old `dialect/anchor.rs`

### Phase 3: Create Move sub-module

- [x] Create `dialect/move_lang/` directory
- [x] Create `move_lang/types.rs` — move `MoveType` from `move_lang.rs`
- [x] Create `move_lang/exprs.rs` — move `MoveExpr` from `move_lang.rs`
  - [x] Refactor `MoveExpr` to enum-of-structs
- [x] Create `move_lang/stmts.rs` — move `MoveStmt` from `move_lang.rs`
- [x] Create `move_lang/decls.rs` — move `MoveMemberDecl`, `MoveTypeParam`, `MoveDecl` from `move_lang.rs`
  - [x] Refactor `MoveMemberDecl` to enum-of-structs
- [x] Create `move_lang/mod.rs` — re-export all child modules
- [x] Delete old `dialect/move_lang.rs`

### Phase 4: Create Spec sub-module

- [x] Create `dialect/spec/` directory
- [x] Move `spec_dialect.rs` → `spec/mod.rs`
- [x] Delete old `dialect/spec_dialect.rs`

### Phase 5: Update `dialect/mod.rs`

- [x] Update module declarations (`evm_ext` removed, `spec_dialect` → `spec`)
- [x] Update re-export: `pub use evm_ext::…` → `pub use evm::…`
- [x] Update `DialectType::Spec` path: `spec_dialect::SpecType` → `spec::SpecType`

### Phase 6: Update downstream imports in `analyzer`

- [x] `detectors/sir/reentrancy.rs` — `evm_ext` → `evm`
- [x] `detectors/sir/cei_violation.rs` — `evm_ext` → `evm`
- [x] `detectors/sir/missing_access_control.rs` — `evm_ext` → `evm`
- [x] `detectors/sir/sir_missing_access_control.rs` — `evm_ext` → `evm`
- [x] `detectors/sir/missing_modifies.rs` — `evm_ext` → `evm`
- [x] `detectors/sir/unchecked_call.rs` — `evm_ext` → `evm`
- [x] `detectors/sir/uninitialized.rs` — `evm_ext` → `evm`
- [x] `detectors/sir/constant_state_var.rs` — `evm_ext` → `evm`

### Phase 7: Verification

- [x] `cargo build` succeeds
- [x] `cargo test` passes (21 pre-existing Solidity frontend test failures, unrelated to restructuring)
