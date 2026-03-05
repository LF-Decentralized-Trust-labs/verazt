# Plan: Multi-Chain Contract Examples and SCIR Dialect Extensions

## Overview

This document outlines a plan for:
1. [Contract Examples](#1-contract-examples) — Solidity, Vyper, Solana (Anchor), Move (Aptos), Move (Sui)
2. [SCIR Dialect Extensions](#2-scir-dialect-extensions) — extending and adding dialects in SCIR
3. [Compilation Pipelines](#3-compilation-pipelines) — Vyper → SCIR, Anchor → SCIR, Move → SCIR

**Terminology note**: The architecture document (Version 6, 2026) calls Layer 1 "ContractIR (CIR)". The implementation crate is named **SCIR** (Smart Contract IR). This plan uses SCIR throughout. Attribute namespaces use `#scir.*` (matching the codebase) where the architecture document writes `#cir.*`.

---

## 1. Contract Examples

### Goal

Create a canonical set of contract examples for each platform. Each example set covers:
- A simple **token / fungible asset** contract
- A **vault** (deposit/withdraw with access-gated admin)
- An **intentional bug** variant of the vault (reentrancy, unchecked arithmetic, missing access control)

These examples will serve as test inputs for analysis pipelines, benchmarks, and IR validation.

### Directory Structure

```
examples/
├── solidity/
│   ├── token.sol
│   ├── vault.sol
│   └── vault_buggy.sol          # reentrancy bug
├── vyper/
│   ├── token.vy
│   ├── vault.vy
│   └── vault_buggy.vy           # missing nonreentrant
├── solana/
│   ├── token/                   # Anchor program
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── vault/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── vault_buggy/             # missing signer check on admin instruction
│       ├── Cargo.toml
│       └── src/lib.rs
├── move_aptos/
│   ├── token/
│   │   ├── Move.toml
│   │   └── sources/token.move
│   ├── vault/
│   │   ├── Move.toml
│   │   └── sources/vault.move
│   └── vault_buggy/             # missing capability check
│       ├── Move.toml
│       └── sources/vault.move
└── move_sui/
    ├── token/
    │   ├── Move.toml
    │   └── sources/token.move
    ├── vault/
    │   ├── Move.toml
    │   └── sources/vault.move
    └── vault_buggy/             # missing admin check on shared object drain
        ├── Move.toml
        └── sources/vault.move
```

### 1.1 Solidity Examples

Already supported by the existing pipeline. The examples should be clean, minimal, and cover:

**token.sol** — a minimal ERC-20 with `transfer`, `approve`, `transferFrom`, and `balanceOf`.

**vault.sol** — a vault where:
- Users `deposit(amount)` ERC-20 tokens
- Users `withdraw(amount)` back their tokens
- An `owner` can call `emergencyWithdraw()`
- Follows Checks-Effects-Interactions (CEI) pattern correctly

**vault_buggy.sol** — same as vault but the `withdraw` function makes the external call before updating the balance (classic reentrancy).

### 1.2 Vyper Examples

**token.vy** — ERC-20 using Vyper-style `HashMap`, `@external` and `@view` decorators, and `convert()` for type casting.

**vault.vy** — vault using `@nonreentrant("lock")` on `deposit`/`withdraw`. Should demonstrate Vyper's built-in reentrancy guard correctly applied.

**vault_buggy.vy** — the `@nonreentrant` decorator is intentionally omitted from `withdraw`, leaving it vulnerable.

### 1.3 Solana / Anchor Examples

Uses [Anchor](https://www.anchor-lang.com/) framework with Rust.

**token program** — a simple SPL-style token with:
- `initialize_mint(ctx, decimals)`
- `mint_to(ctx, amount)`
- `transfer(ctx, amount)`
- Account constraints: `#[account(mint::authority = authority)]`

**vault program** — a vault with:
- `initialize(ctx)` — creates the vault account (PDA)
- `deposit(ctx, amount)` — transfers tokens into vault
- `withdraw(ctx, amount)` — transfers tokens back to user
- `emergency_withdraw(ctx)` — admin-only, drains vault
- Uses `#[account(signer)]` and `#[account(constraint = ...)]`

**vault_buggy** — missing signer check on `emergency_withdraw`, allowing any caller to drain the vault.

### 1.4 Move (Aptos) Examples

Uses Aptos Move with `aptos_framework` dependencies.

**token module** — a simple coin-like module using `aptos_framework::coin`:
- `initialize(account)` — creates a new coin type with `CoinInfo`
- `mint(account, amount): Coin<T>` — mints coins
- `transfer(from, to, amount)` — uses `coin::transfer`

**vault module** — a vault using Aptos resources:
- `VaultStore { balance: u64, owner: address }` stored at owner address
- `deposit(account, amount)` — merges coin into vault
- `withdraw(account, amount)` — removes coin from vault
- `emergency_withdraw(admin, target)` — admin-only via capability pattern

**vault_buggy** — `emergency_withdraw` checks `signer::address_of(admin)` against a stored address but the comparison is done incorrectly (off-by-one logic or missing assertion), allowing a non-admin to call it.

### 1.5 Move (Sui) Examples

Uses Sui Move with `sui` framework, object-centric model.

**token module** — a simple wrapped coin:
- `create(ctx): (TreasuryCap<TOKEN>, CoinMetadata<TOKEN>)` — creates coin type
- Uses `sui::coin::mint`, `sui::coin::burn`, `sui::transfer::transfer`

**vault module** — a vault using shared objects:
- `Vault { id: UID, balance: Balance<SUI>, admin: address }` as a shared object
- `deposit(vault: &mut Vault, coin: Coin<SUI>, ctx: &TxContext)` — adds balance
- `withdraw(vault: &mut Vault, amount: u64, ctx: &mut TxContext): Coin<SUI>` — user withdrawal
- `admin_withdraw(vault: &mut Vault, ctx: &TxContext)` — admin-only drain

**vault_buggy** — `admin_withdraw` is missing the `ctx.sender() == vault.admin` check entirely, allowing any caller to drain the vault.

---

## 2. SCIR Dialect Extensions

### Current State

The SCIR crate defines one dialect, `EvmDialect`, in [crates/scir/src/dialect/evm.rs](crates/scir/src/dialect/evm.rs):
- `EvmType` — `Address`, `AddressPayable`, `Slot`
- `EvmExpr` — `MsgSender`, `MsgValue`, `Timestamp`, `BlockNumber`, `InlineAsm`
- `EvmStmt` — `EmitEvent`, `TryCatch`
- `EvmMemberDecl` — `EventDef`, `ModifierDef`, `ErrorDef`, `EnumDef`, `StructDef`

The four extension points are wired up in [crates/scir/src/dialect/mod.rs](crates/scir/src/dialect/mod.rs):
```
DialectType    → EvmType (and future move/anchor types)
DialectExpr    → EvmExpr
DialectStmt    → EvmStmt
DialectMemberDecl → EvmMemberDecl
```

### Dialect Namespace Map

The architecture document defines the following dialect-to-chain mapping. This determines which dialect module covers which source language:

| Dialect | Source Languages Covered |
|---|---|
| `evm` | Solidity, Vyper, Fe |
| `move` | Move (language-level), Aptos, Sui |
| `anchor` | Anchor framework, Solana |
| `spec` | All chains (formal verification) |

**Key implication**: Vyper does NOT get its own dialect. It is covered by the `evm` dialect with Vyper-specific extensions. Similarly, Aptos and Sui share a single `move` dialect; the `#scir.chain_target` module attribute (`"aptos"` or `"sui"`) distinguishes them at the compilation layer.

### Module-Level Dialect Attributes

Every SCIR module carries attributes identifying its source language and which dialects are loaded:

```
module ERC20 attributes {
    #scir.source_lang    = "solidity",
    #scir.chain_target   = "evm",
    #scir.compiler       = "solc-0.8.x",
    #scir.loaded_dialects = ["evm", "spec"]
}

module BasicCoin attributes {
    #scir.source_lang     = "move",
    #scir.chain_target    = "aptos",
    #scir.loaded_dialects = ["move", "spec"]
}

module escrow attributes {
    #scir.source_lang     = "rust/anchor",
    #scir.chain_target    = "solana",
    #scir.loaded_dialects = ["anchor", "spec"]
}
```

These attributes are set by the compilation frontend (Pass 1) and consumed by analysis passes and the AIR lowering layer.

### Type Interfaces

Rather than encoding Move's linear type system as a Rust enum, the architecture uses **Type Interfaces** that dialect types implement. These replace any `MoveAbility` enum in the plan:

```
interface Copyable     { op copy(%v: T) -> T }
interface Droppable    { op drop(%v: T) }
interface Storable     { op is_storable() -> bool }
interface KeyType      { op to_key(%v: T) -> bytes }
interface LinearValue  { op drain(%v: T) -> tuple<...> }
```

Example instantiations:
- `!evm.uint256` → implements Copyable, Droppable, Storable
- `!move.resource<T>` → implements Storable, KeyType (NOT Copyable, NOT Droppable)
- `!sierra.felt252` → implements Copyable, Droppable, Storable, LinearValue
- `!tezos.ticket<T>` → implements LinearValue (NOT Copyable, NOT Droppable)

Move struct abilities are expressed as **attributes** on struct declarations, not as a Rust enum:

```
// struct with store ability only — dialect_member_decl
move.struct_def Coin<phantom CoinType>
    attributes { #move.abilities=["store"] }   // NOT Copyable, NOT Droppable
{ value: i64 }

// struct with key ability — Storable + KeyType
move.struct_def Balance<phantom CoinType>
    attributes { #move.abilities=["key"] }
{ coin: Coin<CoinType> }
```

### 2.1 EVM Dialect Extensions (for Vyper)

**File to extend: [crates/scir/src/dialect/evm.rs](crates/scir/src/dialect/evm.rs)**

Vyper is covered by the `evm` dialect. The existing `EvmDialect` already covers `MsgSender`, `MsgValue`, `Timestamp`, `BlockNumber`, and `EmitEvent`, which Vyper shares. The following Vyper-specific constructs need to be added to the existing `evm` dialect enums.

#### New EvmType variants (for Vyper)

```rust
// Add to existing EvmType enum:
// Vyper's bounded dynamic array — DynArray[T, N]
DynArray { elem: Box<Type>, max_len: u64 },
// Vyper's bounded byte strings — Bytes[N] and String[N]
BoundedBytes(u64),
BoundedString(u64),
```

#### New EvmExpr variants (for Vyper)

```rust
// Add to existing EvmExpr enum:
// convert(x, T) — Vyper's explicit type cast builtin
Convert { expr: Box<Expr>, to: Type },
// slice(x, start, len) — byte slice
Slice { expr: Box<Expr>, start: Box<Expr>, length: Box<Expr> },
// len(x) — length of DynArray or Bytes
Len(Box<Expr>),
// raw_call(target, data, value?, gas?) — low-level call
RawCall { target: Box<Expr>, data: Box<Expr>, value: Option<Box<Expr>>, gas: Option<Box<Expr>> },
// send(target, value) — Vyper's send() builtin
Send { target: Box<Expr>, value: Box<Expr> },
// self.balance
SelfBalance,
// empty(T) — zero value of type T
Empty(Type),
// concat(a, b, ...) — byte/string concat
Concat(Vec<Expr>),
```

#### Attribute extensions for Vyper (within evm namespace)

Vyper decorators map to `Attr` values in the attribute system. These use the `evm` namespace (since Vyper targets the EVM) with Vyper-specific keys:

| Vyper decorator | SCIR attribute |
|---|---|
| `@external` | `#scir.visibility = "public"` |
| `@internal` | `#scir.visibility = "internal"` |
| `@view` | `#scir.mutability = "view"` |
| `@pure` | `#scir.mutability = "pure"` |
| `@payable` | `#evm.payable = true` |
| `@nonreentrant("key")` | `#evm.nonreentrant = "key"` |
| `@deploy` (constructor) | `#evm.is_constructor = true` |

Note: `@nonreentrant` is Vyper's built-in reentrancy guard, not a Solidity modifier. It maps to a single attribute rather than a modifier reference. Analysis passes should detect that a function lacking `#evm.nonreentrant` that also has an external call is a potential reentrancy risk.

#### No new VyperMemberDecl

Vyper's `interface` definitions and `constant`/`immutable` declarations can be represented with existing constructs:
- Interfaces → represented as contracts with abstract function declarations and `#scir.is_interface = true`
- Constants → `StorageDecl` with `#evm.is_constant = true` and an initializer expression
- Immutables → `StorageDecl` with `#evm.is_immutable = true`

### 2.2 Anchor Dialect

**New file: [crates/scir/src/dialect/anchor.rs](crates/scir/src/dialect/anchor.rs)**

Solana/Anchor uses a fundamentally different model from EVM: stateless programs, explicit account passing, and Cross-Program Invocations (CPIs). The naming conventions follow the architecture document.

#### AnchorType

```rust
pub enum AnchorType {
    // !anchor.pubkey — Solana public key (32 bytes)
    Pubkey,
    // !anchor.signer — must-sign account
    Signer,
    // !anchor.account<T> — program-owned account holding data of type T
    Account(Box<Type>),
    // !anchor.system_account — system-program-owned account
    SystemAccount,
    // !anchor.unchecked_account — no ownership validation (use with care)
    UncheckedAccount,
    // !anchor.program<T> — a program account (T = system/token/etc.)
    Program(Box<Type>),
    // !anchor.context<T> — the ctx parameter type for an instruction
    Context(Box<Type>),
    // !anchor.result<T> — Anchor's Result<T> return type
    Result(Box<Type>),
}
```

Sentinel program types for `!anchor.program<T>`:
- `anchor.System` — system program
- `anchor.Token` — SPL token program
- `anchor.AssociatedToken` — associated token program

#### AnchorExpr

```rust
pub enum AnchorExpr {
    // anchor.account_load(ctx.accounts.X) — immutable account data load
    AccountLoad(Box<Expr>),
    // anchor.account_load_mut(ctx.accounts.X) — mutable account data load
    AccountLoadMut(Box<Expr>),
    // anchor.signer_key(ctx.accounts.X) — get pubkey of a signer account
    SignerKey(Box<Expr>),
    // anchor.ok(value) — wrap value in Anchor's Result::Ok
    Ok(Box<Expr>),
    // anchor.cpi(...) — cross-program invocation
    Cpi {
        program: Box<Expr>,
        accounts: Vec<Expr>,
        data: Box<Expr>,
    },
    // anchor.system_transfer(from, to, lamports) — system program transfer CPI
    SystemTransfer { from: Box<Expr>, to: Box<Expr>, lamports: Box<Expr> },
    // anchor.token_transfer(from, to, authority, amount) — SPL token transfer CPI
    TokenTransfer { from: Box<Expr>, to: Box<Expr>, authority: Box<Expr>, amount: Box<Expr> },
    // PDA derivation
    FindProgramAddress { seeds: Vec<Expr>, program_id: Box<Expr> },
}
```

#### AnchorStmt

```rust
pub enum AnchorStmt {
    // anchor.emit_event — event emission via Anchor's emit! macro
    EmitEvent { event: String, fields: Vec<(String, Expr)> },
}
```

#### AnchorMemberDecl

```rust
pub enum AnchorMemberDecl {
    // anchor.accounts_context — #[derive(Accounts)] struct
    AccountsContext {
        name: String,
        accounts: Vec<AnchorAccountField>,
    },
    // anchor.account_struct — #[account] data struct (serialized to account data)
    AccountStruct {
        name: String,
        discriminator_size: u8,    // #anchor.discriminator_size (default 8)
        fields: Vec<(String, Type)>,
    },
    // anchor.event_struct — #[event] struct
    EventStruct {
        name: String,
        fields: Vec<(String, Type)>,
    },
    // anchor.error_code — #[error_code] enum
    ErrorCode {
        name: String,
        variants: Vec<(String, String)>,  // (variant_name, message)
    },
}

pub struct AnchorAccountField {
    pub name: String,
    pub ty: AnchorType,
    pub is_mut: bool,              // #anchor.constraint = "mut"
    pub constraint: Option<String>, // #anchor.constraint = "init" | "mut" | etc.
    pub payer: Option<String>,     // #anchor.payer = "field_name"
    pub space: Option<Expr>,       // #anchor.space = N
    pub check_note: Option<String>, // #anchor.check_note = "human readable explanation"
}
```

#### Anchor Attribute Conventions

Anchor constructs use the `#anchor.*` attribute namespace:

| Source | SCIR attribute |
|---|---|
| `#[program]` module | `#anchor.program = true` on ContractDecl |
| Instruction `entry` | `#anchor.entry = true` on FunctionDecl |
| `#[account(mut)]` | `#anchor.constraint = "mut"` on account field |
| `#[account(signer)]` | account field type is `!anchor.signer` |
| `#[account(init, payer=X, space=N)]` | `#anchor.constraint = "init"`, `#anchor.payer = "X"`, `#anchor.space = N` |
| `#[account(has_one = authority)]` | `#anchor.constraint = "has_one:authority"` |
| `#[account(constraint = expr)]` | `#anchor.constraint = "custom"` + attached Expr |
| `#[account(seeds=[...], bump)]` | `#anchor.seeds = [...]`, `#anchor.bump = true` |
| `#[account(close = target)]` | `#anchor.close = "target"` |
| `anchor.cpi(...)` call risk | `#scir.call_risk = {reentrancy: false, delegate_storage: false}` |

### 2.3 Move Dialect (single dialect for Aptos and Sui)

**New file: [crates/scir/src/dialect/move_lang.rs](crates/scir/src/dialect/move_lang.rs)**

The `move` dialect covers Move (the language), Aptos, and Sui. The chain-specific distinction is made only via the `#scir.chain_target` module attribute (`"aptos"` or `"sui"`). There is no separate Aptos or Sui dialect — framework-level calls (e.g., `aptos_framework::coin::deposit` vs `sui::transfer::transfer`) are represented as regular `FunctionCall` expressions with fully qualified names, resolved at the compilation frontend.

#### MoveType

```rust
pub enum MoveType {
    // !move.resource<T> — struct type with key ability (lives in global storage)
    Resource(Box<Type>),
    // !move.signer — signer capability passed to entry functions
    Signer,
    // !move.type_tag — phantom type tag (used in forall quantifiers in specs)
    TypeTag,
}
```

Note: `vector<T>` maps to the existing core `Type::Array(T)`. Native integer types (`u8`, `u64`, `u128`, `u256`) map to existing `Type::I8`, `Type::I64`, etc. `address` and `bool` map to core types extended through the move namespace.

#### MoveExpr

```rust
pub enum MoveExpr {
    // move.borrow_global<T>(addr) — immutable global storage borrow
    BorrowGlobal { ty: Type, addr: Box<Expr> },
    // move.borrow_global_mut<T>(addr) — mutable global storage borrow
    BorrowGlobalMut { ty: Type, addr: Box<Expr> },
    // move.exists<T>(addr) — check if resource exists at address
    Exists { ty: Type, addr: Box<Expr> },
    // move.signer_address(signer) — get address from signer
    SignerAddress(Box<Expr>),
    // move.move_to(resource, signer) — publish resource to signer's address
    MoveTo { resource: Box<Expr>, signer: Box<Expr> },
    // move.move_from<T>(addr) — remove resource from address
    MoveFrom { ty: Type, addr: Box<Expr> },
    // move.write_ref(ref, value) — write through a mutable reference (*ref = value)
    WriteRef { reference: Box<Expr>, value: Box<Expr> },
    // move.ghost_total_supply<T> — spec-only ghost variable (used in @invariant)
    GhostVar(String),
}
```

#### MoveStmt

```rust
pub enum MoveStmt {
    // abort code — terminate with error code
    Abort(Box<Expr>),
    // spec block embedded in function body
    SpecBlock { assertions: Vec<Expr> },
}
```

#### MoveMemberDecl

```rust
pub enum MoveMemberDecl {
    // move.struct_def — struct declaration with abilities as #move.abilities attribute
    StructDef {
        name: String,
        // Type parameters; phantom params marked with #move.phantom = true
        type_params: Vec<MoveTypeParam>,
        fields: Vec<(String, Type)>,
    },
    // move.spec_fun — spec-only function (not compiled, used in proofs)
    SpecFun {
        name: String,
        params: Vec<(String, Type)>,
        ret: Type,
        body: Option<Expr>,
    },
    // friend <module_path>
    FriendDecl(String),
}

pub struct MoveTypeParam {
    pub name: String,
    pub is_phantom: bool,    // #move.phantom = true
    // Ability constraints expressed as attributes on the type param,
    // NOT as a Rust enum — see Type Interfaces section
}
```

#### Move Attribute Conventions

| Source concept | SCIR attribute |
|---|---|
| `struct S has store` | `#move.abilities = ["store"]` on struct |
| `struct S has key` | `#move.abilities = ["key"]` on struct |
| `struct S has copy, drop` | `#move.abilities = ["copy","drop"]` on struct |
| `public entry fun f()` | `#move.entry = true` on FunctionDecl |
| `#[view]` | `#move.view = true` |
| `acquires Balance<CoinType>` | `#move.acquires = ["Balance<CoinType>"]` on FunctionDecl |
| `native fun` | `#move.native = true` |
| `public(friend) fun` | `#scir.visibility = "friend"` |
| `phantom T` in type params | `#move.phantom = true` on type parameter |
| Aptos `#[event]` | `#move.is_event = true` on struct |
| Sui `init` function | `#move.is_init = true` on FunctionDecl |
| Sui shared object | `#move.shared = true` on StorageDecl or struct |

#### Move Expression Conventions in SCIR

The Move dialect uses qualified expression names for framework-level operations:

**Aptos framework calls** (when `#scir.chain_target = "aptos"`) are represented as `FunctionCall` nodes with the fully qualified Aptos module path. Analysis passes recognize these via name patterns:

| Aptos source | SCIR representation |
|---|---|
| `coin::mint(&cap, amount)` | `FunctionCall("aptos_framework::coin::mint", [cap, amount])` |
| `coin::deposit(addr, coin)` | `FunctionCall("aptos_framework::coin::deposit", [addr, coin])` |
| `coin::withdraw(account, amount)` | `FunctionCall("aptos_framework::coin::withdraw", [account, amount])` |
| `coin::balance<T>(addr)` | `FunctionCall("aptos_framework::coin::balance", [addr])` |
| `object::address_from_object(obj)` | `FunctionCall("aptos_framework::object::address_from_object", [obj])` |
| `event::emit_event(&mut handle, ev)` | `FunctionCall("aptos_framework::event::emit_event", [handle, ev])` |
| `table::add(&mut t, k, v)` | `FunctionCall("aptos_std::table::add", [t, k, v])` |

**Sui framework calls** (when `#scir.chain_target = "sui"`) are similarly represented as `FunctionCall` nodes:

| Sui source | SCIR representation |
|---|---|
| `object::new(ctx)` | `FunctionCall("sui::object::new", [ctx])` |
| `transfer::transfer(obj, addr)` | `FunctionCall("sui::transfer::transfer", [obj, addr])` |
| `transfer::share_object(obj)` | `FunctionCall("sui::transfer::share_object", [obj])` with `#move.shared = true` |
| `transfer::freeze_object(obj)` | `FunctionCall("sui::transfer::freeze_object", [obj])` |
| `coin::mint(cap, amount, ctx)` | `FunctionCall("sui::coin::mint", [cap, amount, ctx])` |
| `balance::split(&mut b, amount)` | `FunctionCall("sui::balance::split", [b, amount])` |
| `tx_context::sender(ctx)` | `FunctionCall("sui::tx_context::sender", [ctx])` with `TaintSource: SignerArg` |
| `clock::timestamp_ms(clock)` | `FunctionCall("sui::clock::timestamp_ms", [clock])` |

Sui-specific types (`UID`, `ID`, `Coin<T>`, `Balance<T>`, `TxContext`, `Clock`) are represented as `TypeRef` referring to their fully qualified Sui framework type names, recognized by the AIR lowering step.

### 2.4 Spec Dialect

**Extend: [crates/scir/src/dialect/mod.rs](crates/scir/src/dialect/mod.rs)**

The `spec` dialect provides formal verification constructs used across all chains. Some of these already exist in the SCIR core (`old`, `result`, `forall`, `exists` in `exprs.rs`). The spec dialect adds types for ghost variables and propositions:

```rust
pub enum SpecType {
    // !spec.prop — type of a logical proposition
    Prop,
    // !spec.ghost<T> — ghost variable type (spec-only, not compiled)
    Ghost(Box<Type>),
}
```

Ghost variables appear in `move.ghost_total_supply<ct>` and similar spec annotations. They are declared using `move.spec_fun` or as special `StorageDecl` nodes tagged with `#spec.ghost = true`.

---

## 3. AIR Dialect Lowering Requirements

Each dialect must register **lowering rules** that map its SCIR constructs to AIR interface implementations. This is required for the Pass 2a transformation (SCIR → AnalysisIR). Analysis passes only interact with the generic interfaces — they never branch on dialect names.

The four AIR interfaces are:

| Interface | Purpose | Used for |
|---|---|---|
| `StorageOp` | State reads/writes | Alias analysis, reaching defs |
| `CallOp` | External calls | ICFG edges, reentrancy detection |
| `TaintSource` | User-controlled inputs | Taint propagation |
| `TaintSink` | Security-relevant sinks | Taint detection |

### 3.1 EVM Dialect AIR Lowering

| SCIR form | AIR lowering | Interface | Notes |
|---|---|---|---|
| `evm.msg_sender()` | `EVM_CALLER` pseudo-value | `TaintSource: UserControlled` | Seeds taint graph |
| `evm.msg_value()` | `EVM_CALLVALUE` pseudo-value | `TaintSource: UserControlled` | Seeds taint graph |
| `balances[k]` (read) | `evm.sload(keccak(slot_index, k))` | `StorageOp` | `AliasGroupId = "balances[*]"` |
| `balances[k] = v` (write) | `evm.sstore(keccak(slot_index, k), v)` | `StorageOp` | Alias group write |
| `evm.emit_event(...)` | `evm.emit_event` (retained) | `TaintSink: EventLog` | No CFG edge |
| `evm.call(...)` or raw external call | `ExternalCallNode` | `CallOp` | Reads `#scir.call_risk` for reentrancy edge |

### 3.2 Move Dialect AIR Lowering

| SCIR form | AIR lowering | Interface | Notes |
|---|---|---|---|
| `move.borrow_global<T>(addr)` | `move.borrow_global` (retained) | `StorageOp` | `AliasGroupId = "T[*]"` |
| `move.borrow_global_mut<T>(addr)` | `move.borrow_global_mut` (retained) | `StorageOp` | Read+write alias group |
| `move.signer_address(s)` | `MOVE_SIGNER_ADDR` pseudo-value | `TaintSource: SignerArg` | Signer is runtime-verified — NOT `UserControlled` |
| `move.write_ref(ref, v)` | `move.write_ref(ref, v)` | `StorageOp` | Linearity check: ref consumed |
| `move.move_from<T>(addr)` | `move.move_from` (retained) | `StorageOp` | Read+remove from alias group |
| `move.move_to(resource, signer)` | `move.move_to` (retained) | `StorageOp` | Write to alias group |
| `FunctionCall("aptos_framework::coin::*")` | framework `CallSite` | `CallOp` | Cross-module; modifies summary |
| `FunctionCall("sui::tx_context::sender")` | `SUI_TX_SENDER` pseudo-value | `TaintSource: SignerArg` | |
| `FunctionCall("sui::transfer::transfer")` | `ExternalCallNode` | `CallOp` | `reentrancy: false` (no re-entry on Sui) |

### 3.3 Anchor Dialect AIR Lowering

| SCIR form | AIR lowering | Interface | Notes |
|---|---|---|---|
| `anchor.account_load(ctx.X)` | `anchor.account_load` (retained) | `StorageOp` | Makes data flow from raw bytes explicit |
| `anchor.account_load_mut(ctx.X)` | `anchor.account_load_mut` (retained) | `StorageOp` | Read+write alias group |
| `anchor.signer_key(ctx.X)` | `ANCHOR_SIGNER_KEY` pseudo-value | `TaintSource: SignerArg` | Runtime-verified signer |
| `anchor.cpi(...)` | `ExternalCallNode` | `CallOp` | `#scir.call_risk = {reentrancy: false}` |
| `anchor.system_transfer(...)` | `ExternalCallNode` | `CallOp` | `value_transfer: true` |

---

## 4. Compilation Pipelines

All pipelines produce **SCIR** as output. Downstream transformation to AIR (for analysis) and VIR (for verification) is handled by existing and future passes in the `smarthunt` / `smartproof` crates.

### 4.1 Vyper → SCIR

#### Approach

Vyper provides an official JSON AST output via `vyper -f ast`. The approach mirrors the existing Solidity pipeline: parse the JSON AST into a Rust AST, normalize it, then lower to SCIR using the **extended `evm` dialect** (since Vyper targets the EVM).

#### New Crate: `crates/vyper/`

```
crates/vyper/
├── Cargo.toml
└── src/
    ├── main.rs                  # CLI: vyper compiler wrapper
    ├── parser/
    │   ├── mod.rs
    │   └── json_ast_parser.rs   # Parse `vyper -f ast` JSON output
    ├── ast/
    │   ├── mod.rs               # Vyper AST types
    │   ├── normalize/
    │   │   ├── mod.rs
    │   │   ├── resolve_interfaces.rs
    │   │   ├── expand_for_range.rs   # for i in range(N) → while loop
    │   │   └── elim_augmented_assign.rs
    │   └── utils/
    ├── irgen/
    │   ├── mod.rs
    │   └── ir_gen.rs            # Vyper AST → SCIR Module (evm dialect)
    └── lib.rs
```

#### Compilation Phases

**Phase 1 — Invoke Vyper Compiler**

Run `vyper -f ast <file>.vy` to get the JSON AST. Use `vyper-select` for version management (analogous to `solc-select`). The `-f ast` output uses Python-like AST node names (`ast_type`, `col_offset`, `lineno`).

**Phase 2 — Parse JSON AST**

Key Vyper AST nodes:

| Vyper AST node | Rust struct |
|---|---|
| `Module` | `SourceUnit` |
| `FunctionDef` (with decorators) | `FunctionDef` |
| `AnnAssign` (state var) | `StateVariableDef` |
| `EventDef` | `EventDef` (→ `EvmMemberDecl::EventDef`) |
| `StructDef` | `StructDef` (→ `EvmMemberDecl::StructDef`) |
| `InterfaceDef` | `InterfaceDef` (→ abstract ContractDecl) |
| `For` | `ForStmt` |
| `If` | `IfStmt` |
| `Return` | `ReturnStmt` |
| `Assign` / `AugAssign` | `AssignStmt` |
| `Log` | `LogStmt` (→ `EvmStmt::EmitEvent`) |
| `Raise` | `RaiseStmt` (→ `Stmt::Revert`) |

**Phase 3 — Normalize**

1. `expand_for_range` — Desugar `for i: uint256 in range(N)` to a `while` loop with counter
2. `elim_augmented_assign` — Convert `x += y` to `x = x + y`
3. `resolve_interfaces` — Inline interface definitions for call-site resolution
4. `annotate_overflow` — Tag arithmetic with `Checked` semantics (default in Vyper >= 0.3.9) or `Wrapping` for older versions via `#scir.overflow = "wrapping"` on `BinOp` nodes

**Phase 4 — Lower to SCIR (evm dialect)**

```
lower_source_unit(SourceUnit) -> scir::Module
```

Key mappings:

| Vyper concept | SCIR concept |
|---|---|
| Module | `scir::Module` with `#scir.source_lang = "vyper"`, `#scir.chain_target = "evm"`, `#scir.loaded_dialects = ["evm","spec"]` |
| State var | `StorageDecl` |
| `@external def f()` | `FunctionDecl` + `#scir.visibility = "public"` |
| `@internal def f()` | `FunctionDecl` + `#scir.visibility = "internal"` |
| `@payable` | `#evm.payable = true` |
| `@nonreentrant("key")` | `#evm.nonreentrant = "key"` |
| `@view` / `@pure` | `#scir.mutability = "view"` / `"pure"` |
| `__init__` | `FunctionDecl` + `#evm.is_constructor = true` |
| `HashMap[K, V]` | `Type::Map(K, V)` |
| `DynArray[T, N]` | `EvmType::DynArray { elem: T, max_len: N }` |
| `Bytes[N]` | `EvmType::BoundedBytes(N)` |
| `address` | `EvmType::Address` |
| `msg.sender` | `EvmExpr::MsgSender` |
| `msg.value` | `EvmExpr::MsgValue` |
| `convert(x, T)` | `EvmExpr::Convert { expr: x, to: T }` |
| `log MyEvent(...)` | `EvmStmt::EmitEvent` |
| `raise` | `Stmt::Revert` |
| `interface Foo: ...` | Abstract `ContractDecl` with `#scir.is_interface = true` |

### 4.2 Anchor (Solana) → SCIR

#### Approach

Anchor programs are Rust source files with procedural macros. Two options exist:

**Option A — Parse Anchor IDL (JSON)**: Easy, but only covers the interface, not the implementation.

**Option B — Parse Rust source with `syn`**: Preserves full implementation semantics, required for vulnerability detection.

**Chosen: Option B**, with optional IDL mode for interface-only analysis.

#### New Crate: `crates/anchor/`

```
crates/anchor/
├── Cargo.toml                   # depends on: syn, scir
└── src/
    ├── main.rs
    ├── parser/
    │   ├── mod.rs
    │   ├── rust_parser.rs       # Parse .rs → syn::File
    │   ├── anchor_extractor.rs  # Extract Anchor items from syn AST
    │   └── idl_parser.rs        # (Optional) Parse Anchor IDL JSON
    ├── ast/
    │   ├── mod.rs               # Anchor intermediate AST
    │   └── normalize/
    │       ├── mod.rs
    │       ├── resolve_ctx.rs   # Expand ctx.accounts.X
    │       └── elim_macros.rs   # Desugar require!, emit!, etc.
    ├── irgen/
    │   ├── mod.rs
    │   └── ir_gen.rs            # Anchor AST → SCIR Module (anchor dialect)
    └── lib.rs
```

#### Compilation Phases

**Phase 1 — Parse Rust Source**

Use `syn::parse_file()` on the `.rs` file. Locate:
- `#[program]` module → instruction handler functions
- `#[derive(Accounts)]` structs → `anchor.accounts_context` declarations
- `#[account]` structs → `anchor.account_struct` declarations
- `#[event]` structs → `anchor.event_struct` declarations
- `#[error_code]` enums → `anchor.error_code` declarations

**Phase 2 — Build Anchor AST**

```
AnchorProgram {
    name: String,
    instructions: Vec<AnchorInstruction>,
    accounts_contexts: Vec<AnchorAccountsContext>,
    account_structs: Vec<AnchorAccountStruct>,
    events: Vec<AnchorEvent>,
    errors: Vec<AnchorErrorVariant>,
}
```

Each `AnchorInstruction` holds: name, parameter types, body as `syn::Block`, reference to its `Context<AccountsContext>`.

**Phase 3 — Normalize**

1. `resolve_ctx` — Expand `ctx.accounts.X` into named variable references
2. `resolve_constraints` — Convert `#[account(has_one = authority)]` and similar into explicit `assert` stmts at function entry
3. `elim_macros` — Desugar:
   - `require!(cond, Error::X)` → `if !cond { return Err(Error::X) }`
   - `emit!(MyEvent { ... })` → `anchor.emit_event` stmt

**Phase 4 — Lower to SCIR (anchor dialect)**

```
lower_program(AnchorProgram) -> scir::Module
```

Key mappings:

| Anchor source | SCIR concept |
|---|---|
| `#[program]` module | `ContractDecl` with `#anchor.program = true` |
| Instruction handler `fn f(ctx: Context<A>, ...)` | `FunctionDecl` + `#anchor.entry = true` + param `ctx: !anchor.context<A>` |
| `#[derive(Accounts)]` struct | `AnchorMemberDecl::AccountsContext` |
| `#[account]` data struct | `AnchorMemberDecl::AccountStruct` |
| `ctx.accounts.X` | `anchor.account_load(ctx.accounts.X)` or `anchor.account_load_mut(...)` |
| `ctx.accounts.X.key()` | `anchor.signer_key(ctx.accounts.X)` |
| `anchor_lang::prelude::invoke(...)` | `anchor.cpi(...)` with `#scir.call_risk = {reentrancy: false}` |
| `anchor_spl::token::transfer(...)` | `anchor.token_transfer(...)` |
| `system_program::transfer(...)` | `anchor.system_transfer(...)` |
| `emit!(Event { ... })` | `AnchorStmt::EmitEvent` |
| `return Ok(())` | `Return` + `anchor.ok(none)` |
| `#[account(mut)]` | `#anchor.constraint = "mut"` |
| `#[account(signer)]` | field type = `!anchor.signer` |
| `#[account(seeds=[...], bump)]` | `#anchor.seeds = [...]`, `#anchor.bump = true` |
| `#[account(init, payer=X, space=N)]` | `#anchor.constraint = "init"`, `#anchor.payer = "X"`, `#anchor.space = N` |
| `Pubkey` type | `!anchor.pubkey` |

#### Challenges

- `syn` gives the pre-macro-expansion AST. Anchor macros follow strict structural patterns, so they can be handled without expansion.
- `Account<'info, T>` lifetime parameters must be stripped; only the data type `T` is relevant.
- CPI helpers from `anchor_spl` must be recognized by name and mapped to `anchor.token_transfer` / `anchor.system_transfer` builtins rather than generic `anchor.cpi`.

### 4.3 Move (Aptos) → SCIR

#### Approach

Parse Move source files directly using the `move-compiler` crate as a library (Option A), falling back to a custom parser (Option B) if the dependency is too heavyweight.

#### New Crate: `crates/move_aptos/`

```
crates/move_aptos/
├── Cargo.toml                   # depends on: move-compiler (or custom), scir
└── src/
    ├── main.rs
    ├── parser/
    │   ├── mod.rs
    │   └── move_parser.rs       # Wrap move-compiler parser or custom
    ├── ast/
    │   ├── mod.rs               # Intermediate Move AST
    │   └── normalize/
    │       ├── mod.rs
    │       ├── resolve_use.rs   # Resolve use declarations
    │       ├── extract_specs.rs # Separate spec blocks
    │       └── annotate_abilities.rs
    ├── irgen/
    │   ├── mod.rs
    │   └── ir_gen.rs            # Move AST → SCIR Module (move dialect, Aptos target)
    └── lib.rs
```

#### Compilation Phases

**Phase 1 — Parse Move Source**

Move modules have this structure:
```move
module <address>::<name> {
    use <address>::<module>::{<items>};
    struct Foo has key, store { ... }
    public entry fun bar(account: &signer, ...) acquires Foo { ... }
    spec bar { ... }
}
```

**Phase 2 — Build Intermediate AST**

```
MoveModule {
    address: String,
    name: String,
    uses: Vec<UseDecl>,
    structs: Vec<MoveStruct>,
    functions: Vec<MoveFunction>,
    specs: Vec<SpecBlock>,
    friends: Vec<String>,
}
```

**Phase 3 — Normalize**

1. `resolve_use` — Resolve module-relative names to fully qualified paths
2. `extract_specs` — Separate `spec` blocks; carry their contents into `FuncSpec` annotations
3. `annotate_abilities` — Set `#move.abilities=[...]` on struct declarations
4. `resolve_phantom` — Mark phantom type parameters with `#move.phantom = true`

**Phase 4 — Lower to SCIR (move dialect, Aptos target)**

```
lower_module(MoveModule) -> scir::Module
```

Module attributes: `#scir.source_lang = "move"`, `#scir.chain_target = "aptos"`, `#scir.loaded_dialects = ["move","spec"]`.

Key mappings:

| Move concept | SCIR concept |
|---|---|
| Module | `ContractDecl` (one module = one contract) |
| `struct Foo has key { ... }` | `MoveMemberDecl::StructDef` + `#move.abilities = ["key"]` |
| `struct Coin<phantom T> has store` | `MoveMemberDecl::StructDef` + `#move.abilities = ["store"]` + phantom type param |
| `public fun f()` | `FunctionDecl` + `#scir.visibility = "public"` |
| `public entry fun f()` | `FunctionDecl` + `#move.entry = true` |
| `fun f()` (private) | `FunctionDecl` + `#scir.visibility = "internal"` |
| `acquires Balance<T>` | `#move.acquires = ["Balance<T>"]` on FunctionDecl |
| `let x: T = ...` | `Stmt::LocalDecl` |
| `move.borrow_global<T>(addr)` | `MoveExpr::BorrowGlobal` |
| `move.borrow_global_mut<T>(addr)` | `MoveExpr::BorrowGlobalMut` |
| `*ref = *ref - amount` | `MoveExpr::WriteRef` |
| `move.exists<T>(addr)` | `MoveExpr::Exists` |
| `move.signer_address(s)` | `MoveExpr::SignerAddress` |
| `abort code` | `MoveStmt::Abort` |
| `assert!(cond, code)` | `Stmt::Assert` |
| `coin::mint(cap, amount)` | `FunctionCall("aptos_framework::coin::mint", ...)` |
| `coin::deposit(addr, coin)` | `FunctionCall("aptos_framework::coin::deposit", ...)` |
| `spec requires P` | `FuncSpec.requires` |
| `spec ensures Q` | `FuncSpec.ensures` |
| `spec modifies s` | `FuncSpec.modifies` |
| `u8`/`u64`/`u128`/`u256` | `Type::I8`/`Type::I64`/`Type::I128`/`Type::I256` |
| `address` (Move type) | `MoveType::Resource` or `TypeRef("address")` |
| `signer` | `MoveType::Signer` |
| `vector<T>` | `Type::Array(T)` |

### 4.4 Move (Sui) → SCIR

#### Approach

Sui Move is syntactically identical to Aptos Move, with differences only in the standard library. The compilation pipeline is **shared with the Aptos Move crate** at phases 1–3. A separate `crates/move_sui/` crate provides only the lowering phase (Phase 4) with Sui-specific framework call mappings.

A `crates/move_core/` shared crate should be extracted to avoid duplication: both `move_aptos` and `move_sui` depend on it for parsing and normalization.

#### Revised Crate Structure

```
crates/move_core/               # shared Move parser + AST + normalizations
├── Cargo.toml                  # depends on: move-compiler (or custom), scir
└── src/
    ├── parser/, ast/, lib.rs

crates/move_aptos/              # Aptos-specific lowering only
├── Cargo.toml                  # depends on: move_core, scir
└── src/
    ├── main.rs
    ├── irgen/ir_gen.rs         # Move AST → SCIR (chain_target = "aptos")
    └── lib.rs

crates/move_sui/                # Sui-specific lowering only
├── Cargo.toml                  # depends on: move_core, scir
└── src/
    ├── main.rs
    ├── irgen/ir_gen.rs         # Move AST → SCIR (chain_target = "sui")
    └── lib.rs
```

#### Phase 4 — Lower to SCIR (move dialect, Sui target)

Module attributes: `#scir.source_lang = "move"`, `#scir.chain_target = "sui"`, `#scir.loaded_dialects = ["move","spec"]`.

All common Move mappings from Section 4.3 apply. Additionally:

| Sui source | SCIR concept |
|---|---|
| `object::new(ctx)` | `FunctionCall("sui::object::new", [ctx])` — returns `UID` |
| `transfer::transfer(obj, addr)` | `FunctionCall("sui::transfer::transfer", [obj, addr])` |
| `transfer::share_object(obj)` | `FunctionCall("sui::transfer::share_object", [obj])` + `#move.shared = true` on the transferred object |
| `transfer::freeze_object(obj)` | `FunctionCall("sui::transfer::freeze_object", [obj])` |
| `transfer::public_transfer(obj, addr)` | `FunctionCall("sui::transfer::public_transfer", [obj, addr])` |
| `coin::mint(cap, amount, ctx)` | `FunctionCall("sui::coin::mint", [cap, amount, ctx])` |
| `balance::split(&mut b, amount)` | `FunctionCall("sui::balance::split", [b, amount])` |
| `balance::join(&mut b, other)` | `FunctionCall("sui::balance::join", [b, other])` |
| `tx_context::sender(ctx)` | `FunctionCall("sui::tx_context::sender", [ctx])` — `TaintSource: SignerArg` in AIR |
| `clock::timestamp_ms(clock)` | `FunctionCall("sui::clock::timestamp_ms", [clock])` |
| `UID` type | `TypeRef("sui::object::UID")` |
| `TxContext` type | `TypeRef("sui::tx_context::TxContext")` |
| `Coin<T>` type | `TypeRef("sui::coin::Coin")` applied to `T` |
| `Balance<T>` type | `TypeRef("sui::balance::Balance")` applied to `T` |
| Module `init` function | `FunctionDecl` + `#move.is_init = true` |
| `entry fun` | `FunctionDecl` + `#move.entry = true` |

#### Key Differences from Aptos

1. **No global storage**: Sui has no `move.move_to` / `move.move_from` / `move.borrow_global`. All state is in objects passed as function arguments. The AIR lowering for Sui therefore has no `StorageOp` from global storage — only `anchor.account_load`-style ops from object arguments.

2. **Object ownership model**: `transfer::share_object` makes an object accessible by anyone but introduces concurrency semantics. This is captured as `#move.shared = true` on the affected object declaration and used by the access-control analysis pass.

3. **UID tracking**: Every Sui object struct must contain a `UID` field. The lowering must detect structs containing `TypeRef("sui::object::UID")` and tag them with `#move.is_object = true`.

4. **TxContext as signer source**: Unlike Aptos where `&signer` is the authority source, in Sui the authority comes from `tx_context::sender(ctx)`. The AIR lowering must map this to `TaintSource: SignerArg` — the same label used for `anchor.signer_key()` and `move.signer_address()`.

---

## 5. Implementation Order

1. **Contract examples** — Write first; needed for testing everything downstream
   - Solidity examples (no new code needed)
   - Vyper examples
   - Anchor examples
   - Move (Aptos) examples
   - Move (Sui) examples

2. **SCIR dialect extensions** — Extend SCIR before writing compilers
   - Extend `evm.rs` with Vyper-specific `EvmType` and `EvmExpr` variants
   - Add `move_lang.rs` (the `move` dialect)
   - Add `anchor.rs` (the `anchor` dialect)
   - Add `spec` dialect types to `mod.rs`
   - Update `utils/visit.rs`, `utils/fold.rs`, `utils/map.rs` to handle new dialect nodes
   - Update `dialect/mod.rs` to wire new dialects into `DialectType`, `DialectExpr`, etc.

3. **Vyper → SCIR** — Closest to existing Solidity pipeline; good first target

4. **Move core** — Extract `move_core` crate with shared parser/normalizer

5. **Move (Aptos) → SCIR** — Lower using `move_core` output

6. **Move (Sui) → SCIR** — Share `move_core`; Sui lowering is mostly framework call mapping

7. **Anchor → SCIR** — Most complex due to Rust macro patterns; do last

---

## 6. Open Questions

- **`move-compiler` dependency**: Evaluate whether the official Move compiler crate's transitive dependencies are acceptable, or whether a custom `pest`/`nom`-based Move parser is preferable.
- **Anchor macro expansion**: How much of Anchor's proc-macro-generated code needs analysis? Discriminator verification and account constraint checks are generated — the plan currently re-derives these structurally.
- **Vyper version support**: Vyper 0.3.x and 0.4.x have AST differences (e.g., `@deploy` decorator is 0.4.x). Define a minimum supported version.
- **Sui concurrency semantics**: Shared objects in Sui can be accessed by concurrent transactions. The current plan marks them `#move.shared = true` but does not model concurrency in SCIR. This is acceptable for single-transaction analysis but should be noted as an out-of-scope limitation.
- **`move_core` vs feature-flagged single crate**: Decide early whether `move_aptos` and `move_sui` share a `move_core` crate or are merged with feature flags.
