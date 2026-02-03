# Plan: Merge Yul Crate Inside Solidity Crate

This document outlines the plan to merge the `yul` crate into the `solidity` crate, creating both Yul AST and Yul IR data structures with `Yul`-prefixed names.

## Overview

The goal is to consolidate the `yul` crate into `solidity` by:
1. Moving Yul AST files into `solidity/src/ast/yul/` subfolder
2. Creating **new Yul IR** data structures in `solidity/src/ir/yul/` (separate from AST)
3. All data structures will be prefixed with `Yul` to avoid naming conflicts
4. Yul utility methods merge into existing Solidity `Fold`/`Map`/`Visit` traits
5. **Rename** `solidity/src/parser` → `solidity/src/parsing`
6. Merge Yul parsing into `solidity/src/parsing/yul/`
7. Other Yul components (normalize) go into `solidity/src/yul/`

---

## Current State

### Yul Crate Structure

```
yul/src/
├── ast/           → Move to solidity/src/ast/yul/
├── normalize/     → Move to solidity/src/yul/normalize/
├── parsing/       → Move to solidity/src/parsing/yul/
└── util/          → Merge into solidity/src/ast/utils/ and ir/utils/
```

### Solidity Parser Structure (to be renamed)

```
solidity/src/parser/     → RENAME to solidity/src/parsing/
├── json_ast/
│   ├── mod.rs
│   └── ast_parser.rs
├── typ/
│   ├── mod.rs
│   └── ...
├── version/
│   ├── mod.rs
│   └── ...
└── mod.rs
```

### Yul Parsing Structure

```
yul/src/parsing/         → Move to solidity/src/parsing/yul/
├── keywords.rs
├── mod.rs
├── parser.rs
└── yul_grammar.pest
```

### Existing Solidity Utils Structure

```
solidity/src/ast/utils/
├── fold.rs        # Existing Fold trait  → Add fold_yul_* methods
├── map.rs         # Existing Map trait   → Add map_yul_* methods
├── visit.rs       # Existing Visit trait → Add visit_yul_* methods
└── ...

solidity/src/ir/utils/
├── fold.rs        # Existing Fold trait  → Add fold_yul_* methods
├── map.rs         # Existing Map trait   → Add map_yul_* methods
└── visit.rs       # Existing Visit trait → Add visit_yul_* methods
```

---

## Data Structures to Create

### Yul AST Data Structures (in `ast/yul/`)

| Original Name    | New Name           | File                |
|------------------|--------------------|---------------------|
| `SourceUnit`     | `YulSourceUnit`    | `source_unit.rs`    |
| `Block`          | `YulBlock`         | `block.rs`          |
| `FuncDef`        | `YulFuncDef`       | `definitions.rs`    |
| `VarDecl`        | `YulVarDecl`       | `definitions.rs`    |
| `Identifier`     | `YulIdentifier`    | `identifier.rs`     |
| `Stmt`           | `YulStmt`          | `statements.rs`     |
| `AssignStmt`     | `YulAssignStmt`    | `statements.rs`     |
| `ForStmt`        | `YulForStmt`       | `statements.rs`     |
| `IfStmt`         | `YulIfStmt`        | `statements.rs`     |
| `SwitchStmt`     | `YulSwitchStmt`    | `statements.rs`     |
| `SwitchValue`    | `YulSwitchValue`   | `statements.rs`     |
| `SwitchDefault`  | `YulSwitchDefault` | `statements.rs`     |
| `Expr`           | `YulExpr`          | `expressions.rs`    |
| `CallExpr`       | `YulCallExpr`      | `expressions.rs`    |
| `MemberExpr`     | `YulMemberExpr`    | `expressions.rs`    |
| `Lit`            | `YulLit`           | `literals.rs`       |
| `NumLit`         | `YulNumLit`        | `literals.rs`       |
| `BoolLit`        | `YulBoolLit`       | `literals.rs`       |
| `HexLit`         | `YulHexLit`        | `literals.rs`       |
| `StringLit`      | `YulStringLit`     | `literals.rs`       |
| `Type`           | `YulType`          | `types.rs`          |
| `IntType`        | `YulIntType`       | `types.rs`          |
| `TupleType`      | `YulTupleType`     | `types.rs`          |
| `Object`         | `YulObject`        | `sections.rs`       |
| `Code`           | `YulCode`          | `sections.rs`       |
| `Data`           | `YulData`          | `sections.rs`       |
| `Comment`        | `YulComment`       | `sections.rs`       |

### Yul IR Data Structures (NEW - in `ir/yul/`)

Create **separate IR data structures** that mirror AST but are independent:

| IR Type              | File                | Description                          |
|----------------------|---------------------|--------------------------------------|
| `YulIRBlock`         | `block.rs`          | IR block containing IR statements    |
| `YulIRFuncDef`       | `definitions.rs`    | IR function definition               |
| `YulIRVarDecl`       | `definitions.rs`    | IR variable declaration              |
| `YulIRStmt`          | `statements.rs`     | IR statement enum                    |
| `YulIRAssignStmt`    | `statements.rs`     | IR assignment statement              |
| `YulIRForStmt`       | `statements.rs`     | IR for loop statement                |
| `YulIRIfStmt`        | `statements.rs`     | IR if statement                      |
| `YulIRSwitchStmt`    | `statements.rs`     | IR switch statement                  |
| `YulIRExpr`          | `expressions.rs`    | IR expression enum                   |
| `YulIRCallExpr`      | `expressions.rs`    | IR function call expression          |
| `YulIRLit`           | `literals.rs`       | IR literal enum                      |
| `YulIRType`          | `types.rs`          | IR type enum                         |

> [!NOTE]
> Yul IR types are **separate** from Yul AST types to allow:
> - Different fields/structures optimized for IR operations
> - Independent evolution of AST vs IR representations
> - Proper lowering from AST to IR

---

### Utility Trait Methods (merge into existing traits)

Yul methods will be added to Solidity's existing traits in both AST and IR utils:

**AST Utils (`ast/utils/`):**
| Method                 | Trait   | New Method              |
|------------------------|---------|-------------------------|
| `fold_source_unit`     | `Fold`  | `fold_yul_source_unit`  |
| `fold_block`           | `Fold`  | `fold_yul_block`        |
| `map_*`, `visit_*`     | ...     | `*_yul_*`               |

**IR Utils (`ir/utils/`):**
| Method                 | Trait   | New Method                 |
|------------------------|---------|----------------------------|
| `fold_block`           | `Fold`  | `fold_yul_ir_block`        |
| `fold_stmt`            | `Fold`  | `fold_yul_ir_stmt`         |
| `map_*`, `visit_*`     | ...     | `*_yul_ir_*`               |

---

## Target Directory Structure

```
solidity/src/
├── ast/
│   ├── mod.rs              # [MODIFY] Add `pub mod yul;`
│   ├── utils/
│   │   ├── fold.rs         # [MODIFY] Add fold_yul_* methods
│   │   ├── map.rs          # [MODIFY] Add map_yul_* methods
│   │   ├── visit.rs        # [MODIFY] Add visit_yul_* methods
│   │   └── ...
│   └── yul/                # [NEW] Yul AST subfolder
│       ├── mod.rs
│       ├── block.rs
│       ├── definitions.rs
│       ├── expressions.rs
│       ├── identifier.rs
│       ├── literals.rs
│       ├── sections.rs
│       ├── source_unit.rs
│       ├── statements.rs
│       └── types.rs
├── ir/
│   ├── mod.rs              # [MODIFY] Add `pub mod yul;`
│   ├── utils/
│   │   ├── fold.rs         # [MODIFY] Add fold_yul_ir_* methods
│   │   ├── map.rs          # [MODIFY] Add map_yul_ir_* methods
│   │   ├── visit.rs        # [MODIFY] Add visit_yul_ir_* methods
│   │   └── ...
│   └── yul/                # [NEW] Yul IR subfolder
│       ├── mod.rs
│       ├── block.rs
│       ├── definitions.rs
│       ├── expressions.rs
│       ├── literals.rs
│       ├── statements.rs
│       └── types.rs
├── parsing/                # [RENAME from parser/] All parsing modules
│   ├── mod.rs              # [MODIFY] Add `pub mod yul;`
│   ├── json_ast/           # Existing Solidity JSON AST parser
│   │   ├── mod.rs
│   │   └── ast_parser.rs
│   ├── typ/                # Existing type parser
│   │   └── ...
│   ├── version/            # Existing version parser
│   │   └── ...
│   └── yul/                # [NEW] Yul parsing subfolder
│       ├── mod.rs
│       ├── keywords.rs
│       ├── parser.rs
│       └── yul_grammar.pest
├── yul/                    # [NEW] Other Yul components
│   ├── mod.rs
│   ├── globals.rs
│   └── normalize/
│       ├── mod.rs
│       └── rename_variables.rs
└── lib.rs                  # [MODIFY] Rename parser → parsing, add yul
```

---

## File Changes

### 1. [RENAME] `solidity/src/parser/` → `solidity/src/parsing/`

Rename the directory and update `lib.rs`:

```diff
-pub mod parser;
+pub mod parsing;
```

### 2. [NEW] `solidity/src/ast/yul/mod.rs`

```rust
//! Yul AST data structures.

pub mod block;
pub mod definitions;
pub mod expressions;
pub mod identifier;
pub mod literals;
pub mod sections;
pub mod source_unit;
pub mod statements;
pub mod types;

pub use block::*;
pub use definitions::*;
pub use expressions::*;
pub use identifier::*;
pub use literals::*;
pub use sections::*;
pub use source_unit::*;
pub use statements::*;
pub use types::*;
```

### 3. [NEW] `solidity/src/ir/yul/mod.rs`

```rust
//! Yul IR data structures.

pub mod block;
pub mod definitions;
pub mod expressions;
pub mod literals;
pub mod statements;
pub mod types;

pub use block::*;
pub use definitions::*;
pub use expressions::*;
pub use literals::*;
pub use statements::*;
pub use types::*;
```

### 4. [NEW] `solidity/src/parsing/yul/mod.rs`

```rust
//! Yul parsing module.

pub mod keywords;
pub mod parser;

pub use keywords::*;
pub use parser::*;
```

### 5. [MODIFY] `solidity/src/ast/utils/fold.rs`

Add Yul AST methods to existing `Fold` trait:
- `fold_yul_source_unit`, `fold_yul_block`, `fold_yul_stmt`, etc.

### 6. [MODIFY] `solidity/src/ir/utils/fold.rs`

Add Yul IR methods to existing `Fold` trait:
- `fold_yul_ir_block`, `fold_yul_ir_stmt`, `fold_yul_ir_expr`, etc.

### 7. [MODIFY] `solidity/src/ast/mod.rs` & `solidity/src/ir/mod.rs`

```diff
+pub mod yul;
+pub use yul::*;
```

### 8. [MODIFY] `solidity/src/parsing/mod.rs`

```diff
+pub mod yul;
```

### 9. [MODIFY] `solidity/src/ir/statements.rs`

Update `AsmStmt` to use Yul IR types:

```diff
-pub yul_stmts: Vec<yast::Stmt>,
+pub yul_stmts: Vec<YulIRStmt>,
```

### 10. [MODIFY] `solidity/Cargo.toml`

```diff
-yul = { path = "../yul" }
```

---

## Update References

| File                                         | Change                                           |
|----------------------------------------------|--------------------------------------------------|
| `solidity/src/lib.rs`                        | `pub mod parser` → `pub mod parsing`             |
| All files using `crate::parser::*`           | Update to `crate::parsing::*`                    |
| `solidity/src/ast/statements.rs`             | `use yul::ast as yast` → `use crate::ast::yul`   |
| `solidity/src/ir/statements.rs`              | `yast::Stmt` → `YulIRStmt` from `crate::ir::yul` |
| `solidity/src/parsing/json_ast/ast_parser.rs`| Update Yul imports                               |
| `solidity/src/passes/rename_variables.rs`    | Update to use new paths                          |

---

## Implementation Order

1. **Rename** `solidity/src/parser/` → `solidity/src/parsing/`
2. Update all `crate::parser::*` references to `crate::parsing::*`
3. Copy Yul AST files to `solidity/src/ast/yul/`
4. Rename all AST data structures with `Yul` prefix
5. **Create Yul IR files in `solidity/src/ir/yul/`** (new structures)
6. Copy Yul parsing to `solidity/src/parsing/yul/`
7. Add `fold_yul_*` methods to AST `Fold` trait
8. Add `map_yul_*` methods to AST `Map` trait
9. Add `visit_yul_*` methods to AST `Visit` trait
10. Add `fold_yul_ir_*` methods to IR `Fold` trait
11. Add `map_yul_ir_*` methods to IR `Map` trait
12. Add `visit_yul_ir_*` methods to IR `Visit` trait
13. Copy normalize to `solidity/src/yul/`
14. Update `mod.rs` exports and imports
15. Update Solidity references (AST with IR)
16. Update `Cargo.toml`, remove yul dependency
17. Delete old `yul/` crate, run tests

---

## Verification

```bash
cargo build -p solidity
cargo test -p solidity
```


