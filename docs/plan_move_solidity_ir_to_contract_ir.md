# Plan: Move Solidity IR to `cir` Crate (ContractIR)

## Objective

Extract the existing `crates/solidity/src/ir/` module into a new standalone `crates/cir/` crate, then redesign it into a language-neutral **ContractIR (CIR)** as specified in the *Smart Contract Multi-Level IR Architecture v3* design document.

CIR is a **dialect-extensible IR**: a small, clean core grammar is extended by per-chain dialect definitions that plug in via four extension points (`dialect_expr`, `dialect_stmt`, `dialect_member_decl`, `dialect_type`). The first dialect implemented is `evm` (covering Solidity and Vyper).

The `cir` crate becomes the shared IR foundation consumed by `smarthunt` (analysis) and `smartproof` (verification), and populated by language-specific frontends (`solidity`, and future Move/Anchor/Cairo crates).

---

## Step 1: Create the `cir` Crate Scaffold

1. Create `crates/cir/` with:
   - `Cargo.toml` — dependencies: `num-bigint`, `serde`, `itertools`, `petgraph`, `indexmap`
   - `src/lib.rs` — top-level module with public re-exports

2. Register `crates/cir` in the workspace `Cargo.toml` `[workspace] members` list.

---

## Step 2: Move the Solidity IR Module into `cir`

Copy the entire `crates/solidity/src/ir/` tree into `crates/cir/src/` as the starting point, preserving the internal module structure:

```
crates/cir/src/
  lib.rs
  source_unit.rs      ← from solidity/src/ir/source_unit.rs
  defs.rs             ← from solidity/src/ir/defs.rs
  types.rs            ← from solidity/src/ir/types.rs
  exprs.rs            ← from solidity/src/ir/exprs.rs
  stmts.rs            ← from solidity/src/ir/stmts.rs
  lits.rs             ← from solidity/src/ir/lits.rs
  var.rs              ← from solidity/src/ir/var.rs
  block.rs            ← from solidity/src/ir/block.rs
  yul/                ← from solidity/src/ir/yul/ (temporarily kept)
    mod.rs
    ...
  utils/
    mod.rs
    visit.rs
    fold.rs
    map.rs
    export.rs
    printer.rs
```

After copying, update all `use crate::ir::` and `super::` paths to match the new crate root.

---

## Step 3: Redesign the Core Grammar

Replace the Solidity-centric IR structures with the clean, language-neutral core grammar from the v3 design document. This is the most substantial change.

### 3.1 Module and Contract Structure (`src/module.rs`, `src/defs.rs`)

```rust
pub struct Module {
    pub id: String,
    pub attrs: Vec<Attr>,
    pub decls: Vec<Decl>,
}

pub enum Decl {
    Contract(ContractDecl),
    Dialect(Box<dyn DialectDecl>),
}

pub struct ContractDecl {
    pub name: String,
    pub parents: Vec<String>,  // inheritance
    pub attrs: Vec<Attr>,
    pub members: Vec<MemberDecl>,
}

pub enum MemberDecl {
    Storage(StorageDecl),
    Function(FunctionDecl),
    TypeAlias(TypeAlias),
    GlobalInvariant(Expr),
    Dialect(Box<dyn DialectMemberDecl>),
}

pub struct StorageDecl {
    pub name: String,
    pub ty: Type,
    pub attrs: Vec<Attr>,
}

pub struct FunctionDecl {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub returns: Vec<Type>,
    pub attrs: Vec<Attr>,
    pub spec: Option<FuncSpec>,
    pub body: Option<Vec<Stmt>>,  // None = abstract/interface
}
```

### 3.2 Type System (`src/types.rs`)

Replace the current type enum with explicit bit-width integer types and dialect extension:

```rust
pub enum Type {
    // Unsigned integers
    I1, I8, I16, I32, I64, I128, I256,
    // Signed integers
    Si8, Si16, Si32, Si64, Si128, Si256,
    // Primitive
    Bool, String, Bytes, None, Index,
    // Composite
    Array(Box<Type>),
    FixedArray(Box<Type>, u64),
    Map(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Option(Box<Type>),
    // Named type reference
    TypeRef(String),
    // Dialect-specific types (chain-specific leaf node)
    Dialect(Box<dyn DialectType>),
}

// Type interfaces for linearity constraints (used by Move/Tezos)
pub trait Copyable {}
pub trait Droppable {}
pub trait Storable {}
pub trait KeyType {}
pub trait LinearValue {}
```

### 3.3 Statement Forms (`src/stmts.rs`)

Replace the current `Stmt` enum with the simplified core statement grammar:

```rust
pub enum Stmt {
    LocalVar(LocalVarStmt),     // let x: T = init
    Assign(AssignStmt),         // lhs = rhs
    AugAssign(AugAssignStmt),   // lhs += rhs etc.
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Return(ReturnStmt),
    Revert(RevertStmt),         // abstract failure/revert
    Assert(AssertStmt),
    Block(Vec<Stmt>),
    Dialect(Box<dyn DialectStmt>),
}

pub struct WhileStmt {
    pub cond: Expr,
    pub body: Vec<Stmt>,
    pub invariant: Option<Expr>,  // loop invariant annotation
}
```

### 3.4 Expression Forms (`src/exprs.rs`)

Replace the current `Expr` enum. Key additions: overflow semantics on `BinOp`, and spec expressions (`old`, `result`, `forall`, `exists`) as first-class expression variants:

```rust
pub enum Expr {
    Var(VarExpr),
    Lit(Lit),
    BinOp(BinOpExpr),
    UnOp(UnOpExpr),
    IndexAccess(IndexAccessExpr),   // array[i], mapping[k]
    FieldAccess(FieldAccessExpr),   // struct.field
    FunctionCall(CallExpr),
    TypeCast(TypeCastExpr),
    Ternary(TernaryExpr),
    Tuple(Vec<Expr>),
    // Spec-only expressions (valid in @requires/@ensures)
    Old(Box<Expr>),                 // pre-state value
    Result(u32),                    // return value by index
    Forall { var: String, ty: Type, body: Box<Expr> },
    Exists { var: String, ty: Type, body: Box<Expr> },
    // Dialect extension
    Dialect(Box<dyn DialectExpr>),
}

pub struct BinOpExpr {
    pub op: BinOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
    pub overflow: OverflowSemantics,
}

pub enum OverflowSemantics {
    Checked,     // panic/revert on overflow (Solidity ≥0.8, Vyper 0.3.9+)
    Wrapping,    // silent wrap (Solidity <0.8, Move u64, Rust release)
    Saturating,  // clamp at max/min
    Dialect(Box<dyn DialectOverflow>),
}
```

### 3.5 Function Specification (`src/spec.rs`)

Simplify from the previous `FunctionSpec`/`SpecExpr` design to expression-based annotations. `old(...)`, `result(n)`, `forall(...)`, `exists(...)` are now regular `Expr` variants (see §3.4), so `FuncSpec` only needs to hold expression lists:

```rust
pub struct FuncSpec {
    pub requires: Vec<Expr>,
    pub ensures: Vec<Expr>,
    pub modifies: Vec<StorageRef>,
    pub decreases: Option<Expr>,
}

// StorageRef identifies a storage slot for frame conditions
pub struct StorageRef {
    pub contract: Option<String>,
    pub field: String,
    pub index: Option<Expr>,  // None = wildcard (entire mapping/array)
}
```

### 3.6 Attribute System (`src/attrs.rs`)

Attributes carry metadata on any IR node. The `#cir.*` namespace is reserved for standard CIR annotations:

```rust
pub struct Attr {
    pub namespace: String,   // e.g. "cir", "evm", "move", "anchor"
    pub key: String,
    pub value: AttrValue,
}

pub enum AttrValue {
    Bool(bool),
    Int(i64),
    String(String),
    Map(Vec<(String, AttrValue)>),
}

// Standard #cir.* keys
pub mod cir_attrs {
    pub const VISIBILITY: &str = "visibility";      // "public" | "internal"
    pub const CALL_RISK: &str = "call_risk";        // reentrancy, delegate_storage, etc.
    pub const TAINT: &str = "taint";
    pub const SINK: &str = "sink";
    pub const ACCESS_CONTROL: &str = "access_control";
    pub const REENTRANCY_GUARD: &str = "reentrancy_guard";
    pub const UNCHECKED: &str = "unchecked";
    pub const OVERFLOW: &str = "overflow";
}
```

---

## Step 4: Implement the Dialect Extension System

Dialects are the mechanism by which chain-specific constructs (EVM opcodes, Move resources, Anchor accounts) plug into CIR without polluting the core grammar.

### 4.1 Dialect Traits (`src/dialect/mod.rs`)

```rust
pub trait DialectExpr: std::any::Any + Send + Sync {
    fn name(&self) -> &str;
    fn ty(&self) -> Type;
    fn attrs(&self) -> &[Attr];
}

pub trait DialectStmt: std::any::Any + Send + Sync {
    fn name(&self) -> &str;
    fn attrs(&self) -> &[Attr];
}

pub trait DialectMemberDecl: std::any::Any + Send + Sync {
    fn name(&self) -> &str;
}

pub trait DialectType: std::any::Any + Send + Sync {
    fn dialect(&self) -> &str;
    fn type_name(&self) -> &str;
}
```

### 4.2 EVM Dialect (`src/dialect/evm.rs`)

Implement the first dialect covering Solidity and Vyper:

**EVM dialect types:**
```rust
pub struct EvmAddress;      // !evm.address
pub struct EvmSlot;         // !evm.slot
```

**EVM dialect expressions:**
```rust
pub enum EvmExpr {
    MsgSender,              // evm.msg_sender()
    MsgValue,               // evm.msg_value()
    Timestamp,              // evm.timestamp()
    BlockNumber,            // evm.block_number()
    Sload { slot: Box<Expr> },               // evm.sload(slot)
    MappingKey { base: Box<Expr>, key: Box<Expr> }, // evm key derivation
    InlineAsm { bytecode: Vec<u8> },         // evm.inline_asm (opaque)
}
```

**EVM dialect statements:**
```rust
pub enum EvmStmt {
    Sstore { slot: Box<Expr>, value: Box<Expr> },  // evm.sstore(slot, val)
    EmitEvent { event: String, args: Vec<Expr> },   // evm.emit_event(...)
}
```

**EVM dialect member declarations:**
```rust
pub enum EvmMemberDecl {
    EventDef { name: String, params: Vec<(String, Type)>, indexed: Vec<bool> },
    ModifierDef { name: String, params: Vec<Param>, body: Vec<Stmt> },
}
```

**Note on Yul:** The current `yul/` module from the solidity IR is superseded by `EvmExpr::InlineAsm`. Inline assembly is represented as an opaque `evm.inline_asm` node with conservative attributes (`#cir.call_risk = {reentrancy: true}`, alias group = TOP). Full Yul lifting is deferred to a future pass. Remove the `yul/` module from `cir`.

---

## Step 5: Update Utility Traits

Extend the `Visit`, `Fold`, and `Map` traits in `utils/` to cover all new node types:

- `visit_func_spec` / `fold_func_spec`
- `visit_storage_ref`
- `visit_attr`
- `visit_dialect_expr` / `visit_dialect_stmt` / `visit_dialect_type` (dispatch by trait object)
- Update `visit_expr` to handle `Old`, `Result`, `Forall`, `Exists` variants
- Update `visit_stmt` to handle `While` with `invariant`, and `dialect_stmt`
- Update `visit_type` to handle new integer variants and `Dialect` variant

Default implementations (no-op / identity) keep existing callers from breaking.

---

## Step 6: Wire `solidity` Crate to Use `cir`

1. Add `cir` as a dependency in `crates/solidity/Cargo.toml`:
   ```toml
   [dependencies]
   cir = { path = "../cir" }
   ```

2. Remove `crates/solidity/src/ir/` directory.

3. In `crates/solidity/src/lib.rs`, replace `pub mod ir;` with `pub use cir;`.

4. Rename `crates/solidity/src/codegen/` to `crates/solidity/src/irgen/` and update the `mod irgen;` declaration in `lib.rs`. This better reflects the module's purpose as an AST-to-IR lowering pass (Pass 1) rather than a code generation backend.

5. Update `irgen` to emit the new CIR structures. Key Pass 1 transformations for Solidity:
   - Map `uint256` → `Type::I256`, `address` → `EvmAddress` dialect type
   - Lower `msg.sender` → `EvmExpr::MsgSender`, `msg.value` → `EvmExpr::MsgValue`
   - Attach `#evm.slot_index` attrs on storage fields via C layout rules
   - Lower `modifier` definitions to `EvmMemberDecl::ModifierDef` (not expanded yet)
   - Attach `#cir.call_risk = {reentrancy: true}` on `.call()`/`delegatecall()` expressions
   - Attach `#cir.overflow = Checked` for solc ≥0.8, `Wrapping` for <0.8 on all `BinOp` nodes
   - Lower inline assembly to `EvmExpr::InlineAsm`
   - Lower `@requires`/`@ensures`/`@modifies` NatSpec comments to `FuncSpec` fields

6. Update all remaining references in `crates/solidity/src/` (analysis passes, export, printer) from `crate::ir::` to `cir::`.

7. Fix compilation errors from the type system redesign (Step 3).

---

## Step 7: Update Downstream Crates

### 7.1 `smarthunt`

- Add `cir` dependency to `crates/smarthunt/Cargo.toml`.
- Update all `use solidity::ir::` imports to `use cir::`.
- Verify the updated `Visit`/`Fold`/`Map` traits (Step 5) satisfy smarthunt's traversal needs.
- Analysis passes that inspect storage ops will now match on `DialectExpr` / `DialectStmt` and downcast to `EvmExpr`/`EvmStmt` as needed.

### 7.2 `smartproof`

- Same dependency and import update as smarthunt.
- The `FuncSpec` fields (`requires`, `ensures`, `modifies`) and spec expression variants (`Old`, `Result`, `Forall`, `Exists` in `Expr`) are the primary interface for this crate.

---

## Step 8: Tests and Validation

1. `cargo build --workspace` — fix all compilation errors.
2. `cargo test --workspace` — fix regressions.
3. Verify `soljc` binary produces correct CIR output for existing Solidity test fixtures.
4. Ensure smarthunt analysis passes still produce identical results on the SmartBugs Curated dataset.

---

## File Change Summary

| Action | Path |
|--------|------|
| Create | `crates/cir/Cargo.toml` |
| Create | `crates/cir/src/lib.rs` |
| Move   | `crates/solidity/src/ir/**` → `crates/cir/src/**` (starting point) |
| Rewrite | `crates/cir/src/types.rs` (new bit-width type system) |
| Rewrite | `crates/cir/src/stmts.rs` (new core stmt grammar) |
| Rewrite | `crates/cir/src/exprs.rs` (new core expr grammar + overflow semantics) |
| Rewrite | `crates/cir/src/defs.rs` (Module/ContractDecl/FunctionDecl) |
| Create | `crates/cir/src/spec.rs` (FuncSpec, StorageRef) |
| Create | `crates/cir/src/attrs.rs` (Attr, AttrValue, cir_attrs constants) |
| Create | `crates/cir/src/dialect/mod.rs` (dialect traits) |
| Create | `crates/cir/src/dialect/evm.rs` (EVM dialect) |
| Delete | `crates/cir/src/yul/` (superseded by EvmExpr::InlineAsm) |
| Modify | `crates/cir/src/utils/` (extend visitor traits for new nodes) |
| Modify | `Cargo.toml` (add cir to workspace members) |
| Modify | `crates/solidity/Cargo.toml` (add cir dep) |
| Delete | `crates/solidity/src/ir/` |
| Rename | `crates/solidity/src/codegen/` → `crates/solidity/src/irgen/` |
| Modify | `crates/solidity/src/**` (update imports; update Pass 1 lowering) |
| Modify | `crates/smarthunt/Cargo.toml` |
| Modify | `crates/smarthunt/src/**` (update imports) |
| Modify | `crates/smartproof/Cargo.toml` |
| Modify | `crates/smartproof/src/**` (update imports) |

---

## Implementation Order

```
Step 1 → Step 2 → Step 3 → Step 4 → Step 5 → Step 6 → Step 7 → Step 8
```

- Steps 3–5 build up the full `cir` crate independently and can be worked on before touching dependents.
- Step 6 (solidity) and Step 7 (smarthunt, smartproof) should proceed sequentially to keep the build green at each stage.
- The EVM dialect (Step 4.2) should be implemented in lockstep with the Pass 1 lowering in Step 6 so that the two sides stay consistent.
