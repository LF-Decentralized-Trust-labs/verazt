# Plan: Vyper Crate — Compiler Invocation, AST Parsing, and SCIR Lowering

## Overview

This plan describes the implementation of `crates/vyper`, a crate that:
1. Invokes the `vyper` compiler to produce a JSON AST from a `.vy` source file
2. Parses the JSON AST into an internal Vyper AST (Rust types)
3. Runs normalization passes on the internal AST
4. Lowers the normalized AST into SCIR (`crates/scir`)

The design mirrors `crates/solidity` as closely as possible, adapting for Vyper's simpler language model (no inheritance, no modifiers, no imports in 0.3.x/0.4.x, bounded types).

---

## Reference: Solidity Crate Architecture

The Solidity crate follows this pipeline:

```
solc (external) ──JSON AST──► AstParser ──internal AST──► normalize::run_passes ──► IrGen ──► SCIR Module
```

Key files:
- `crates/solidity/src/parser/mod.rs` — compiler invocation
- `crates/solidity/src/parser/json_ast_parser/ast_parser.rs` — JSON → internal AST
- `crates/solidity/src/ast/normalize/mod.rs` — normalization passes pipeline
- `crates/solidity/src/irgen/ir_gen.rs` — internal AST → SCIR

We follow the same structure for `crates/vyper`.

---

## 1. Crate Setup

### 1.1 Directory Layout

```
crates/vyper/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── main.rs              # CLI binary (optional, for debugging)
    ├── parser/
    │   ├── mod.rs           # Compiler invocation (vyper binary)
    │   └── json_ast_parser/
    │       ├── mod.rs
    │       └── ast_parser.rs  # JSON → internal VyperAST
    ├── ast/
    │   ├── mod.rs
    │   ├── types.rs         # Type system
    │   ├── exprs.rs         # Expression nodes
    │   ├── stmts.rs         # Statement nodes
    │   ├── defs.rs          # Definition nodes (FuncDef, EventDef, StructDef, etc.)
    │   ├── source_unit.rs   # Top-level SourceUnit
    │   └── normalize/
    │       ├── mod.rs       # run_passes() pipeline
    │       ├── rename_vars.rs
    │       ├── flatten_expr.rs
    │       └── ...          # Other passes as needed
    └── irgen/
        ├── mod.rs
        └── ir_gen.rs        # Internal AST → SCIR Module
```

### 1.2 Cargo.toml

```toml
[package]
name = "vyper"
version.workspace = true
authors.workspace = true
license.workspace = true
edition.workspace = true

[[bin]]
name = "vyper"
path = "src/main.rs"

[dependencies]
scir = { path = "../scir" }
extlib = { path = "../extlib" }   # or however the shared utils crate is named
serde_json = "1"
regex = "1"
log = "0.4"
tempfile = "3"
```

Add `"crates/vyper"` to the workspace `members` in the root `Cargo.toml`.

---

## 2. Phase 1: Compiler Invocation (`parser/mod.rs`)

### 2.1 Vyper Compiler Command

The Vyper compiler (`vyper`) produces a JSON AST with the `-f ast` flag:

```sh
vyper -f ast <contract.vy>
```

Output is a single JSON object:

```json
{
  "ast": {
    "ast_type": "Module",
    "body": [ ... ],
    "doc_string": null,
    "lineno": 1,
    "col_offset": 0,
    "end_lineno": 60,
    "end_col_offset": 0
  }
}
```

Each AST node carries:
- `"ast_type"`: string discriminant (e.g., `"FunctionDef"`, `"AnnAssign"`, `"EventDef"`)
- Source location fields: `"lineno"`, `"col_offset"`, `"end_lineno"`, `"end_col_offset"`
- Node-specific child fields

### 2.2 Version Management

Unlike Solidity, Vyper does not have a version-switching tool like `solc-select`. However, the version pragma (`# @version ^0.3.9`) can be read and validated. For now:
- Detect version pragma with a regex
- Run the installed `vyper` binary; fail with a clear error if the version is incompatible
- (Future) Support `vyper-select` or `vvm` (Vyper version manager) if needed

### 2.3 Public API (mirrors `crates/solidity/src/parser/mod.rs`)

```rust
/// Parse a .vy file into internal VyperAST source units.
pub fn parse_input_file(input_file: &str) -> Result<SourceUnit>

/// Parse Vyper source code (string) into internal AST.
pub fn parse_vyper_source_code(source_code: &str) -> Result<SourceUnit>
```

Implementation steps:
1. Check that the input file exists
2. Run `vyper -f ast <input_file>`, capture stdout
3. Save raw JSON to `<output_dir>/<stem>.vy.json` for debugging
4. Pass the JSON string to `AstParser::parse()`

---

## 3. Phase 2: Internal Vyper AST (`ast/`)

Define Rust types for every Vyper language construct. The types closely mirror the Solidity AST but adapted for Vyper. Below is the full mapping.

### 3.1 `source_unit.rs` — Top-Level

```rust
pub struct SourceUnit {
    pub path: String,
    pub body: Vec<SourceUnitElem>,
    pub loc:  Option<Loc>,
}

pub enum SourceUnitElem {
    VersionAnnotation(VersionAnnotation),  // # @version
    Event(EventDef),
    Struct(StructDef),
    Interface(InterfaceDef),
    StateVar(StateVarDecl),
    Func(FuncDef),
    EnumDef(EnumDef),                     // Vyper 0.3.4+
    Flag(FlagDef),                        // Vyper flag type (like enum)
    TypeAlias(TypeAlias),                 // type Foo = Bar
    Import(ImportStmt),                   // Vyper 0.4.x module imports
}
```

### 3.2 `types.rs` — Type System

Vyper types to represent:

```rust
pub enum Type {
    // Primitives
    Bool,
    Address,
    Bytes32,           // bytes32
    Int(IntType),      // int8..int256
    UInt(UIntType),    // uint8..uint256
    Decimal,           // decimal fixed-point

    // Bounded composite types (Vyper-specific)
    BoundedString(u64),         // String[N]
    BoundedBytes(u64),          // Bytes[N]
    DynArray { elem: Box<Type>, max_len: u64 },  // DynArray[T, N]

    // Fixed-size
    FixedArray { elem: Box<Type>, len: u64 },    // T[N]

    // Named references
    HashMap(Box<Type>, Box<Type>),  // HashMap[K, V]
    Struct(String),
    Enum(String),
    Flag(String),
    Interface(String),

    // Visibility wrapper (for public state vars)
    Public(Box<Type>),

    // Tuple (for multi-returns)
    Tuple(Vec<Type>),
}

pub struct IntType  { pub bits: u16 }   // 8, 16, ..., 256
pub struct UIntType { pub bits: u16 }
```

### 3.3 `exprs.rs` — Expressions

```rust
pub enum Expr {
    // Basic
    Ident(Identifier),           // variable reference
    Lit(Lit),                    // literal
    Attribute(AttributeExpr),    // self.x, msg.sender, etc.
    Subscript(SubscriptExpr),    // arr[i], map[k]
    Slice(SliceExpr),            // bslice(x, start, len)  — Vyper 0.3
    Call(CallExpr),              // f(args)

    // Operators
    BinOp(BinOpExpr),
    BoolOp(BoolOpExpr),          // and / or
    Compare(CompareExpr),        // ==, !=, <, <=, >, >=
    UnaryOp(UnaryOpExpr),        // not, -, ~

    // Vyper builtins (complex enough to model explicitly)
    Convert { expr: Box<Expr>, to: Type },    // convert(x, T)
    Empty(Type),                              // empty(T)
    Len(Box<Expr>),                           // len(x)
    Concat(Vec<Expr>),                        // concat(a, b)
    RawCall { target: Box<Expr>, data: Box<Expr>, value: Option<Box<Expr>>, gas: Option<Box<Expr>> },
    Send { target: Box<Expr>, value: Box<Expr> },
    Keccak256(Box<Expr>),
    Sha256(Box<Expr>),
    Ecrecover { hash: Box<Expr>, v: Box<Expr>, r: Box<Expr>, s: Box<Expr> },

    // Tuple
    Tuple(Vec<Expr>),
}

pub struct Lit {
    pub kind: LitKind,
    pub loc: Option<Loc>,
}

pub enum LitKind {
    Bool(bool),
    Int(i128),
    Dec(f64),
    Hex(String),
    Str(String),
    Bytes(Vec<u8>),
}

pub struct BinOpExpr { pub left: Box<Expr>, pub op: BinOp, pub right: Box<Expr>, pub loc: Option<Loc> }
pub enum BinOp { Add, Sub, Mul, Div, FloorDiv, Mod, Pow, BitAnd, BitOr, BitXor, Shl, Shr }

pub enum UnaryOp { Not, Neg, Invert }
pub enum BoolOp { And, Or }
pub enum CmpOp  { Eq, NotEq, Lt, LtE, Gt, GtE, In, NotIn }
```

### 3.4 `stmts.rs` — Statements

```rust
pub enum Stmt {
    Assign(AssignStmt),          // x = expr
    AugAssign(AugAssignStmt),    // x += expr
    AnnAssign(AnnAssignStmt),    // x: T = expr  (local var decl)
    If(IfStmt),
    For(ForStmt),
    While(WhileStmt),            // not present in Vyper 0.3, but in 0.4
    Return(ReturnStmt),
    Assert(AssertStmt),          // assert cond, "msg"
    Raise(RaiseStmt),            // raise  (same as revert)
    Log(LogStmt),                // log EventName(args)
    Pass,
    Break,
    Continue,
    Expr(ExprStmt),
}

pub struct ForStmt {
    pub target: Expr,
    pub iter: ForIter,          // range(n) or range(a, b) or array var
    pub body: Vec<Stmt>,
    pub loc: Option<Loc>,
}

pub enum ForIter {
    Range(RangeIter),           // range(start, stop, step)
    DynArray(Box<Expr>),        // for x in dynarray_var
}

pub struct IfStmt {
    pub cond: Expr,
    pub then_body: Vec<Stmt>,
    pub else_body: Vec<Stmt>,   // may be empty or contain another If
    pub loc: Option<Loc>,
}
```

### 3.5 `defs.rs` — Definitions

```rust
pub struct FuncDef {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub decorators: Vec<FuncDecorator>,
    pub doc_string: Option<String>,
    pub body: Vec<Stmt>,
    pub loc: Option<Loc>,
}

pub enum FuncDecorator {
    Deploy,         // @deploy (constructor)
    External,       // @external
    Internal,       // @internal
    View,           // @view
    Pure,           // @pure
    Payable,        // @payable
    NonReentrant(Option<String>),  // @nonreentrant / @nonreentrant("key")
    Custom(String),
}

pub struct StateVarDecl {
    pub name: String,
    pub typ: Type,              // may be wrapped in Public()
    pub constant: bool,
    pub immutable: bool,
    pub transient: bool,        // Vyper 0.4
    pub doc_string: Option<String>,
    pub loc: Option<Loc>,
}

pub struct EventDef {
    pub name: String,
    pub fields: Vec<EventField>,
    pub loc: Option<Loc>,
}

pub struct EventField {
    pub name: String,
    pub typ: Type,
    pub indexed: bool,
}

pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructField>,
    pub loc: Option<Loc>,
}

pub struct StructField {
    pub name: String,
    pub typ: Type,
}

pub struct InterfaceDef {
    pub name: String,
    pub funcs: Vec<InterfaceFunc>,
    pub loc: Option<Loc>,
}

pub struct EnumDef {
    pub name: String,
    pub variants: Vec<String>,
    pub loc: Option<Loc>,
}

pub struct FlagDef {       // Vyper's flag (bit-flag enum)
    pub name: String,
    pub variants: Vec<String>,
    pub loc: Option<Loc>,
}

pub struct Param {
    pub name: String,
    pub typ: Type,
    pub default: Option<Expr>,
    pub loc: Option<Loc>,
}

pub struct Loc {
    pub lineno: u32,
    pub col_offset: u32,
    pub end_lineno: u32,
    pub end_col_offset: u32,
}
```

---

## 4. Phase 3: JSON AST Parser (`parser/json_ast_parser/ast_parser.rs`)

### 4.1 JSON Node Structure

Every node from the Vyper JSON AST has:

```json
{
  "ast_type": "<NodeType>",
  "lineno": 1,
  "col_offset": 0,
  "end_lineno": 5,
  "end_col_offset": 0,
  ... node-specific fields ...
}
```

### 4.2 Parser Struct

```rust
pub struct AstParser;

impl AstParser {
    pub fn parse(json_str: &str) -> Result<SourceUnit> { ... }

    fn parse_source_unit(node: &Value) -> Result<SourceUnit> { ... }
    fn parse_source_unit_elem(node: &Value) -> Result<Option<SourceUnitElem>> { ... }

    fn parse_func_def(node: &Value) -> Result<FuncDef> { ... }
    fn parse_state_var(node: &Value) -> Result<StateVarDecl> { ... }
    fn parse_event_def(node: &Value) -> Result<EventDef> { ... }
    fn parse_struct_def(node: &Value) -> Result<StructDef> { ... }
    fn parse_interface_def(node: &Value) -> Result<InterfaceDef> { ... }
    fn parse_enum_def(node: &Value) -> Result<EnumDef> { ... }
    fn parse_flag_def(node: &Value) -> Result<FlagDef> { ... }

    fn parse_stmt(node: &Value) -> Result<Stmt> { ... }
    fn parse_expr(node: &Value) -> Result<Expr> { ... }
    fn parse_type(node: &Value) -> Result<Type> { ... }
    fn parse_decorator(node: &Value) -> Result<FuncDecorator> { ... }
    fn parse_loc(node: &Value) -> Option<Loc> { ... }
}
```

### 4.3 Node Type Dispatch

In `parse_source_unit_elem()`, dispatch on `ast_type`:

| `ast_type`                     | Internal type          |
|-------------------------------|------------------------|
| `"FunctionDef"`               | `SourceUnitElem::Func` |
| `"AnnAssign"` (module-level)  | `SourceUnitElem::StateVar` |
| `"EventDef"`                  | `SourceUnitElem::Event` |
| `"StructDef"`                 | `SourceUnitElem::Struct` |
| `"InterfaceDef"`              | `SourceUnitElem::Interface` |
| `"EnumDef"`                   | `SourceUnitElem::EnumDef` |
| `"FlagDef"`                   | `SourceUnitElem::Flag` |
| `"Import"` / `"ImportFrom"`   | `SourceUnitElem::Import` |
| `"VariableDecl"` (0.4.x)     | `SourceUnitElem::StateVar` |
| `"DocStr"` / `"Expr"` (string literal at module scope) | skip or attach as doc string |

In `parse_stmt()`, dispatch on `ast_type`:

| `ast_type`       | Internal type          |
|-----------------|------------------------|
| `"Assign"`      | `Stmt::Assign`         |
| `"AugAssign"`   | `Stmt::AugAssign`      |
| `"AnnAssign"`   | `Stmt::AnnAssign` (local var) |
| `"If"`          | `Stmt::If`             |
| `"For"`         | `Stmt::For`            |
| `"Return"`      | `Stmt::Return`         |
| `"Assert"`      | `Stmt::Assert`         |
| `"Raise"`       | `Stmt::Raise`          |
| `"Log"`         | `Stmt::Log`            |
| `"Pass"`        | `Stmt::Pass`           |
| `"Break"`       | `Stmt::Break`          |
| `"Continue"`    | `Stmt::Continue`       |
| `"Expr"`        | `Stmt::Expr`           |

In `parse_expr()`, dispatch on `ast_type`:

| `ast_type`        | Internal type              |
|------------------|----------------------------|
| `"Name"`         | `Expr::Ident`              |
| `"Attribute"`    | `Expr::Attribute`          |
| `"Subscript"`    | `Expr::Subscript`          |
| `"Call"`         | `Expr::Call` (or builtin)  |
| `"BinOp"`        | `Expr::BinOp`              |
| `"BoolOp"`       | `Expr::BoolOp`             |
| `"Compare"`      | `Expr::Compare`            |
| `"UnaryOp"`      | `Expr::UnaryOp`            |
| `"Constant"`     | `Expr::Lit`                |
| `"Tuple"`        | `Expr::Tuple`              |
| `"IfExp"`        | (ternary, model as `Expr::Call` or new variant) |

Special handling for `"Call"` nodes: detect builtin function names (`convert`, `empty`, `len`, `concat`, `raw_call`, `send`, `keccak256`, `sha256`, `ecrecover`, `slice`, `extract32`, `as_wei_value`, etc.) and map to dedicated `Expr` variants.

In `parse_type()`, dispatch on `ast_type`:

| `ast_type`          | Internal type              |
|--------------------|----------------------------|
| `"Name"`           | primitive types by name    |
| `"Subscript"`      | `HashMap`, `DynArray`, `String[N]`, `Bytes[N]`, `T[N]` |
| `"Attribute"`      | interface member reference |

---

## 5. Phase 4: Normalization Passes (`ast/normalize/`)

Vyper is simpler than Solidity (no inheritance, no modifiers, no using directives), so fewer passes are needed. The following passes should be implemented:

### 5.1 Required Passes

| Pass | Purpose |
|------|---------|
| `rename_vars` | Give unique names to all local variables to avoid shadowing issues |
| `rename_defs` | Qualify function and definition names with their contract (file) scope |
| `flatten_expr` | Hoist complex sub-expressions into temporary local variables to produce a 3-address-like form |
| `elim_augassign` | Optionally desugar `x += e` into `x = x + e` for simpler lowering |
| `expand_public_accessors` | Generate implicit getter functions for `public(T)` state variables |
| `unroll_unary_tuple` | Unwrap single-element tuples |

### 5.2 Passes to Skip (Not Needed for Vyper)

- `resolve_inheritance` — Vyper has no inheritance
- `elim_func_modifier` — Vyper has no modifiers
- `elim_using_directives` — Vyper has no using directives
- `merge_pragmas` — single-file model

### 5.3 `run_passes()` Pipeline

```rust
pub fn run_passes(source_unit: &SourceUnit) -> SourceUnit {
    let su = unroll_unary_tuple(source_unit);
    let su = rename_vars(&su);
    let su = rename_defs(&su);
    let su = expand_public_accessors(&su);
    let su = flatten_expr(&su);
    unroll_unary_tuple(&su)
}
```

---

## 6. Phase 5: IR Lowering (`irgen/ir_gen.rs`)

Lower the normalized Vyper AST into a SCIR `Module`.

### 6.1 Pipeline

```
SourceUnit ──► IrGen::lower_source_unit() ──► scir::Module
```

Each `.vy` file becomes one `scir::Module` containing exactly one `scir::ContractDecl` (Vyper has one contract per file).

### 6.2 Type Lowering

| Vyper type             | SCIR type                              |
|------------------------|----------------------------------------|
| `bool`                 | `Type::Bool`                           |
| `address`              | `Type::Dialect(EvmType::Address)`      |
| `uint8..uint256`       | `Type::I8..Type::I256` (unsigned)      |
| `int8..int256`         | `Type::Si8..Type::Si256` (signed)      |
| `decimal`              | `Type::Si256` (approximation, 10^10)   |
| `bytes32`              | `Type::FixedBytes(32)`                 |
| `String[N]`            | `Type::Dialect(EvmType::BoundedString(N))` |
| `Bytes[N]`             | `Type::Dialect(EvmType::BoundedBytes(N))` |
| `DynArray[T, N]`       | `Type::Dialect(EvmType::DynArray { elem, max_len: N })` |
| `T[N]` (fixed array)   | `Type::FixedArray(Box<Type>, N)`       |
| `HashMap[K, V]`        | `Type::Map(Box<Type>, Box<Type>)`      |
| `Struct S`             | `Type::TypeRef("S")`                   |
| `Enum E`               | `Type::TypeRef("E")`                   |
| `Flag F`               | `Type::TypeRef("F")`                   |
| `Tuple(T1, T2)`        | `Type::Tuple(vec![...])`               |

### 6.3 Expression Lowering

| Vyper expression                  | SCIR expression                                  |
|----------------------------------|--------------------------------------------------|
| `Ident(x)`                       | `Expr::Var(x)`                                   |
| `Lit(v)`                         | `Expr::Lit(v)`                                   |
| `Attribute(self, field)`         | `Expr::Var(field)` (storage access)              |
| `Attribute(obj, field)`          | `Expr::FieldAccess(obj, field)`                  |
| `Subscript(arr, idx)`            | `Expr::IndexAccess(arr, idx)`                    |
| `BinOp(l, op, r)`               | `Expr::BinOp(l, op, r)`                          |
| `BoolOp(And, [a, b])`           | `Expr::BinOp(a, And, b)`                         |
| `Compare(l, op, r)`             | `Expr::BinOp(l, op, r)`                          |
| `UnaryOp(Not, e)`               | `Expr::UnOp(Not, e)`                             |
| `Call(f, args)`                  | `Expr::FunctionCall(f, args)`                    |
| `Convert(e, T)`                  | `Expr::Dialect(EvmExpr::Convert { expr, to })`   |
| `Empty(T)`                       | `Expr::Dialect(EvmExpr::Empty(T))`               |
| `Len(e)`                         | `Expr::Dialect(EvmExpr::Len(e))`                 |
| `Concat(parts)`                  | `Expr::Dialect(EvmExpr::Concat(parts))`          |
| `Slice(e, start, len)`          | `Expr::Dialect(EvmExpr::Slice { expr, start, length })` |
| `RawCall(...)`                   | `Expr::Dialect(EvmExpr::RawCall {...})`          |
| `Send(target, value)`            | `Expr::Dialect(EvmExpr::Send {...})`             |
| `Tuple(elems)`                   | `Expr::Tuple(elems)`                             |
| `msg.sender`                     | `Expr::Dialect(EvmExpr::MsgSender)`              |
| `msg.value`                      | `Expr::Dialect(EvmExpr::MsgValue)`               |
| `block.timestamp`                | `Expr::Dialect(EvmExpr::Timestamp)`              |
| `block.number`                   | `Expr::Dialect(EvmExpr::BlockNumber)`            |
| `self.balance`                   | `Expr::Dialect(EvmExpr::SelfBalance)`            |

### 6.4 Statement Lowering

| Vyper statement          | SCIR statement                                  |
|--------------------------|--------------------------------------------------|
| `Assign(target, val)`    | `Stmt::Assign`                                  |
| `AugAssign(t, op, val)`  | `Stmt::AugAssign`                               |
| `AnnAssign(name, T, val)`| `Stmt::LocalVar`                                |
| `If(cond, then, else)`   | `Stmt::If`                                      |
| `For(target, iter, body)`| `Stmt::For`                                     |
| `Return(val)`            | `Stmt::Return`                                  |
| `Assert(cond, msg)`      | `Stmt::Assert`                                  |
| `Raise`                  | `Stmt::Revert`                                  |
| `Log(event, args)`       | `Stmt::Dialect(EvmStmt::EmitEvent { ... })`     |
| `Pass`                   | (no-op, skip)                                   |
| `Break`                  | `Stmt::Break`                                   |
| `Continue`               | `Stmt::Continue`                                |
| `Expr(e)`                | `Stmt::Expr(e)`                                 |

### 6.5 Definition Lowering

| Vyper definition         | SCIR member                                      |
|--------------------------|--------------------------------------------------|
| `StateVarDecl`           | `MemberDecl::Storage(StorageDecl)`              |
| `FuncDef` (@external, etc.) | `MemberDecl::Function(FunctionDecl)`         |
| `EventDef`               | `MemberDecl::Dialect(EvmMemberDecl::EventDef)`  |
| `StructDef`              | `MemberDecl::Dialect(EvmMemberDecl::StructDef)` |
| `EnumDef`                | `MemberDecl::Dialect(EvmMemberDecl::EnumDef)`   |
| `FlagDef`                | `MemberDecl::Dialect(EvmMemberDecl::EnumDef)`   |

### 6.6 Function Attributes Mapping

| Vyper decorator          | SCIR function attribute              |
|--------------------------|--------------------------------------|
| `@deploy`                | `Attr::Constructor`                  |
| `@external`              | `Attr::Public` (visibility)         |
| `@internal`              | `Attr::Private` (visibility)        |
| `@view`                  | `Attr::View`                         |
| `@pure`                  | `Attr::Pure`                         |
| `@payable`               | `Attr::Payable`                      |
| `@nonreentrant`          | custom attribute or dialect attr     |

### 6.7 Source Unit Lowering

```
SourceUnit
  → Module {
      id: file_path,
      decls: [
        ContractDecl {
          name: file_stem,   // e.g. "token" for token.vy
          parents: [],
          members: [
            StorageDecl for each state var,
            EvmMemberDecl::EventDef for each event,
            EvmMemberDecl::StructDef for each struct,
            EvmMemberDecl::EnumDef for each enum/flag,
            FunctionDecl for each function,
          ],
        }
      ],
    }
```

---

## 7. Public API (`lib.rs`)

```rust
pub mod ast;
pub mod irgen;
pub mod parser;

pub use scir;

/// Full pipeline: .vy file → SCIR Module
pub fn compile_file(input_file: &str) -> Result<scir::Module> {
    let source_unit = parser::parse_input_file(input_file)?;
    let normalized = ast::normalize::run_passes(&source_unit);
    irgen::ir_gen::lower_source_unit(&normalized)
}

/// Full pipeline: Vyper source string → SCIR Module
pub fn compile_source(source_code: &str) -> Result<scir::Module> {
    let source_unit = parser::parse_vyper_source_code(source_code)?;
    let normalized = ast::normalize::run_passes(&source_unit);
    irgen::ir_gen::lower_source_unit(&normalized)
}
```

---

## 8. Testing Strategy

### 8.1 Unit Tests

- `ast_parser.rs`: test individual node parsing functions against hand-crafted JSON fragments
- `types.rs`: test type lowering from Vyper type strings to SCIR types
- `ir_gen.rs`: test lowering of individual statements and expressions

### 8.2 Integration Tests

Use the existing Vyper examples in `examples/vyper/`:
- `token.vy` — ERC-20 token (covers events, mappings, public accessors, @external, @deploy, log)
- `vault.vy` — covers `assert`, `raw_call`, `send`, `self.balance`
- `vault_buggy.vy` — same as vault but with intentional bugs (verify the AST still parses correctly)

Integration test steps for each example:
1. Run `vyper -f ast <file.vy>` and capture JSON
2. Parse JSON → internal AST
3. Run normalization
4. Lower to SCIR
5. Assert the SCIR module has the expected number of contracts, functions, and state variables

### 8.3 Snapshot Tests (optional)

Use `insta` to snapshot the JSON AST output and the resulting SCIR module for each example file, to catch regressions.

---

## 9. Implementation Order

Implement in this order to enable incremental testing:

1. **Crate skeleton** — `Cargo.toml`, `lib.rs`, module stubs
2. **Internal AST types** — `ast/types.rs`, `ast/exprs.rs`, `ast/stmts.rs`, `ast/defs.rs`, `ast/source_unit.rs`
3. **JSON AST parser** — `parser/json_ast_parser/ast_parser.rs` starting with types, then exprs, stmts, defs
4. **Compiler invocation** — `parser/mod.rs` (`parse_input_file`, `parse_vyper_source_code`)
5. **Normalization** — `ast/normalize/mod.rs` with minimal passes (`rename_vars`, `flatten_expr`)
6. **IR lowering** — `irgen/ir_gen.rs` in the order: types → exprs → stmts → defs → source unit
7. **Integration tests** — run the full pipeline on `token.vy`, `vault.vy`
8. **Workspace integration** — add `crates/vyper` to root `Cargo.toml` and wire into `smarthunt`/`smartproof` as needed

---

## 10. Known Vyper Quirks to Handle

| Quirk | Notes |
|-------|-------|
| One contract per file | Each `.vy` file is one implicit contract; no `contract Foo { ... }` block |
| Bounded types | `String[N]`, `Bytes[N]`, `DynArray[T, N]` carry max-length bounds that must be preserved in SCIR via EVM dialect types |
| `self.` prefix | All state variable accesses use `self.`; strip `self.` in storage access lowering |
| `public()` on state vars | Auto-generates a getter function; model in `expand_public_accessors` normalization pass |
| `msg.*` / `block.*` | Are `Attribute` nodes in JSON AST; detect and lower to EVM dialect expressions |
| `log EventName(args)` | Is a `Log` statement node distinct from a function call |
| `for x in range(n)` | `ForIter::Range` with no collection variable |
| `for x in array_var` | `ForIter::DynArray` iterating over a storage array |
| No `null` / `None` in Vyper | `empty(T)` is the zero-value idiom; lower to `EvmExpr::Empty(T)` |
| `@nonreentrant` | May carry a key string argument; encode as custom SCIR attribute |
| Version pragma | `# @version ^0.3.9` is a comment not a JSON node; parse from raw source before invoking compiler |
