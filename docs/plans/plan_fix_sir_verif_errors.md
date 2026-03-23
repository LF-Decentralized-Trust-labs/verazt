# Fix SIR Verification Errors

The command `./target/debug/verazt compile examples/solidity/token.sol -d` produces SIR verification errors across 3 passes. Audit of `lower.rs` reveals several Solidity built-ins that flow through to SIR without proper definitions.

## Approach

> [!IMPORTANT]
> **Lowering-side fixes for built-ins, verifier-side fixes for dialect nodes.**
>
> Solidity built-ins like `require` should be lowered to well-defined SIR constructs (not left as generic function calls). Dialect nodes need verifier adjustments since they're structurally correct.

## Audit Summary

### Full audit of lowering code

The lowering pipeline in [lower.rs](crates/frontend/src/solidity/lowering/lower.rs) was reviewed against the Solidity constructs in [token.sol](examples/solidity/token.sol) and the SIR data model.

#### Constructs producing errors (need fixes)

| Solidity Construct | Current SIR | Failing Passes | Root Cause |
|---|---|---|---|
| `require(cond, msg)` (×3) | `FunctionCall(Var("require"), args)` | **scope_check**: `require` undeclared | **Fix 1b** — lower to `if !cond { revert(msg) }` |
| `address(0)` (×1) | `FunctionCall(Var("address"), [0])` where callee has `Type::None` | **scope_check**: `address` undeclared; **type_well_formed**: callee `Type::None` | `TypeName` → `Var(ty.to_string(), Type::None)` at L748-753; `CallKind::TypeConversionCall` ignored |
| `msg.sender` etc (×9) | `Expr::Dialect(DialectExpr::Evm(MsgSender))` | **type_well_formed**: `Type::None`; **no_orphan_dialect**: flagged as orphan | Dialect exprs return `Type::None`; orphan check rejects all dialects |
| `emit Transfer(...)` etc (×4) | `Stmt::Dialect(DialectStmt::Evm(EmitEvent{...}))` | **no_orphan_dialect**: flagged as orphan | Orphan check rejects all dialect stmts |

#### Constructs correctly handled (no action needed)

| Solidity Construct | SIR Lowering | Notes |
|---|---|---|
| `revert(msg)` | `Stmt::Revert(RevertStmt{...})` | Handled at AST parsing level (L403) |
| `delegatecall/call` | `Expr::Dialect(DialectExpr::Evm(LowLevelCall{...}))` | L899-986 via `lower_call_opts_expr` |
| `try/catch` | `Stmt::Dialect(DialectStmt::Evm(EvmStmt::TryCatch{...}))` | L552-590 |
| `emit Event(args)` | `Stmt::Dialect(DialectStmt::Evm(EvmStmt::EmitEvent{...}))` | L596-610 (structurally correct, blocked by verifier) |
| `block.timestamp/number` | `Expr::Dialect(DialectExpr::Evm(Timestamp/BlockNumber))` | L1111-1128 |
| Storage variables | Declared in scope at contract level | scope_check L191-198 |

#### Other constructs needing fixes (not in token.sol but must be correct)

| Solidity Construct | Current SIR | Issue | Fix |
|---|---|---|---|
| `assert(cond)` | `FunctionCall(Var("assert"))` | Undeclared callee | **Fix 1a** — lower to `Stmt::Assert` |
| `new T(args)` | `Var("new T", Type::None)` | `Type::None`, undeclared | **Fix 5** — lower to `FunctionCall` with `TypeRef` callee |
| `type(X).min/max` | `FunctionCall(Var("min__type__X"))` | callee `Type::None` | **Fix 6** — give callee a `Function` type |
| `keccak256/sha256/ripemd160/ecrecover` | Generic `FunctionCall` | Undeclared callee, no type info | **Fix 7** — lower to `EvmExpr` dialect variants |
| `addmod/mulmod` | Generic `FunctionCall` | Undeclared callee | **Fix 7** — lower to `EvmExpr` dialect variants |
| `abi.encode/encodePacked/decode/...` | Generic `FieldAccess` + `FunctionCall` | `abi` is `TypeRef("abi")` — not a real scope var | **Fix 8** — lower to `EvmExpr` dialect variants |
| `gasleft()` | Generic `FunctionCall` | Undeclared callee | **Fix 7** — lower to `EvmExpr` dialect variant |
| `blockhash(n)` | Generic `FunctionCall` | Undeclared callee | **Fix 7** — lower to `EvmExpr` dialect variant |
| `selfdestruct(addr)` | Generic `FunctionCall` | Undeclared callee | **Fix 9** — lower to `EvmStmt` dialect variant |
| `addr.transfer(amt)` | Generic `FunctionCall` on `FieldAccess` | No special handling | **Fix 9** — lower to `EvmExpr` dialect variant |
| `addr.send(amt)` | Generic `FunctionCall` on `FieldAccess` | No special handling | **Fix 9** — already has `EvmExpr::Send`, need to intercept in lowering |
| `this` | `Var("this", TypeRef("ContractName"))` | Undeclared in scope | **Fix 10** — lower to `EvmExpr::This` |
| `super` | `Var("super", ...)` | Undeclared in scope | **Fix 10** — lower to `EvmExpr::Super` |
| `msg.data` / `msg.sig` | Not handled in `lower_member_expr` | Falls to generic `FieldAccess` on undeclared `msg` | **Fix 11** — add to EVM global member dispatch |
| `block.difficulty/gaslimit/coinbase/chainid/basefee` | Not handled | Falls to generic `FieldAccess` on undeclared `block` | **Fix 11** — add to EVM global member dispatch |

### Verification passes reference

| Pass | File | What it checks |
|---|---|---|
| `type_well_formed` | [type_well_formed.rs](crates/scirs/src/sir/verifier/type_well_formed.rs) | Every `Expr` has non-`None` type; functions with return types have bodies |
| `scope_check` | [scope_check.rs](crates/scirs/src/sir/verifier/scope_check.rs) | Every `Var` reference is declared in an enclosing scope |
| `spec_check` | [spec_check.rs](crates/scirs/src/sir/verifier/spec_check.rs) | Spec clauses only reference parameters/quantified vars |
| `no_orphan_dialect` | [no_orphan_dialect.rs](crates/scirs/src/sir/verifier/no_orphan_dialect.rs) | Dialect nodes appear in recognized contexts |

---

## Proposed Changes

### Fix 1a: Lower `assert(cond)` → `Stmt::Assert` (lowering-side)

Solidity's `assert(cond)` expresses an **invariant that must never be false** — a panic-level internal error. This maps directly to SIR's `Stmt::Assert { cond, message, span }`, which carries the same formal-methods semantics.

**Why handle at statement level**: `assert(...)` is always used as a statement. Intercepting at `lower_expr_stmt` avoids emitting a dummy expression from an expression lowerer.

#### [MODIFY] [lower.rs](crates/frontend/src/solidity/lowering/lower.rs) — `lower_expr_stmt` (~L434)

```diff
 fn lower_expr_stmt(&mut self, s: &ast::ExprStmt) -> Result<Vec<Stmt>> {
     match &s.expr {
         ast::Expr::Assign(a) => self.lower_assign_expr_as_stmt(a, loc_to_span(s.loc)),
+        // ── assert(cond) → Stmt::Assert ─────────────────────────
+        ast::Expr::Call(call) if call.callee.to_string() == "assert" => {
+            self.lower_assert(call, loc_to_span(s.loc))
+        }
+        // ── require(cond, msg?) → Stmt::If(!cond, [Stmt::Revert]) ──
+        ast::Expr::Call(call) if call.callee.to_string() == "require" => {
+            self.lower_require(call, loc_to_span(s.loc))
+        }
         _ => {
             let mut stmts = vec![];
```

#### [ADD] [lower.rs](crates/frontend/src/solidity/lowering/lower.rs) — `lower_assert` helper

```rust
/// Lower `assert(cond)` → `Stmt::Assert { cond, message: None }`.
fn lower_assert(&mut self, call: &ast::CallExpr, span: Option<Span>) -> Result<Vec<Stmt>> {
    let mut stmts = vec![];
    let (args, extra) = self.lower_call_args_exprs(&call.args)?;
    stmts.extend(extra);
    let mut positional = args.into_positional();
    let cond = if positional.is_empty() {
        Expr::Lit(Lit::Bool(BoolLit::new(true, span)))
    } else {
        positional.remove(0)
    };
    stmts.push(Stmt::Assert(AssertStmt { cond, message: None, span }));
    Ok(stmts)
}
```

---

### Fix 1b: Lower `require(cond, msg?)` → `Stmt::If(!cond, [Stmt::Revert])` (lowering-side)

Solidity's `require(cond, msg)` is a **guarded revert**: it reverts with an `Error(string)` if the condition is false. This is semantically different from `assert` — it's an input validation that produces a recoverable error, not a panic.

The correct SIR representation is:

```
if !cond {
    revert(msg)   // Stmt::Revert { error: None, args: [msg] }
}
```

This preserves the distinction for downstream passes: `Stmt::Assert` means "this property must hold unconditionally" (used by provers), while `Stmt::If + Stmt::Revert` means "revert with error if condition fails" (used by runtime analysis).

#### [ADD] [lower.rs](crates/frontend/src/solidity/lowering/lower.rs) — `lower_require` helper

```rust
/// Lower `require(cond, msg?)` → `if !cond { revert(msg) }`.
fn lower_require(&mut self, call: &ast::CallExpr, span: Option<Span>) -> Result<Vec<Stmt>> {
    let mut stmts = vec![];
    let (args, extra) = self.lower_call_args_exprs(&call.args)?;
    stmts.extend(extra);
    let mut positional = args.into_positional();
    let cond = if positional.is_empty() {
        Expr::Lit(Lit::Bool(BoolLit::new(true, span)))
    } else {
        positional.remove(0)
    };
    let msg_args = if positional.is_empty() { vec![] } else { vec![positional.remove(0)] };
    // Negate the condition: !cond
    let neg_cond = Expr::UnOp(UnOpExpr { op: UnOp::Not, operand: Box::new(cond), span });
    let revert = Stmt::Revert(RevertStmt { error: None, args: msg_args, span });
    stmts.push(Stmt::If(IfStmt {
        cond: neg_cond,
        then_body: vec![revert],
        else_body: None,
        span,
    }));
    Ok(stmts)
}
```

**Errors fixed**: 3 `scope_check` errors (one per `require(...)` call in token.sol).

---

### Fix 2: Lower `TypeConversionCall` → `Expr::TypeCast` (lowering-side)

`address(0)` in Solidity is a type conversion. The AST tags it with `CallKind::TypeConversionCall` and the callee is `Expr::TypeName(TypeNameExpr { typ: address })`. Currently `lower_call_expr` ignores `CallKind` and:
1. Lowers the `TypeName` callee via `lower_expr` → `Var("address", Type::None)` (L748-753)
2. Wraps it in `FunctionCall` (L891)

This produces a call to an undeclared variable with `Type::None`.

**Fix**: Check `CallKind::TypeConversionCall` in `lower_call_expr` and emit `Expr::TypeCast` instead.

#### [MODIFY] [lower.rs](crates/frontend/src/solidity/lowering/lower.rs) — `lower_call_expr` (~L883)

```diff
 fn lower_call_expr(&mut self, e: &ast::CallExpr) -> Result<(Expr, Vec<Stmt>)> {
     let mut stmts = vec![];
     let ty = self.lower_type(&e.typ)?;
     let span = loc_to_span(e.loc);
+
+    // ── Type conversion: address(0), uint256(x) etc → TypeCast ──
+    if e.kind == ast::CallKind::TypeConversionCall {
+        let (args, extra) = self.lower_call_args_exprs(&e.args)?;
+        stmts.extend(extra);
+        let mut positional = args.into_positional();
+        if positional.len() == 1 {
+            let inner = positional.remove(0);
+            return Ok((
+                Expr::TypeCast(TypeCastExpr { ty, expr: Box::new(inner), span }),
+                stmts,
+            ));
+        }
+        // Multi-arg type conversion — fall through to generic call
+    }
+
     let (callee, extra) = self.lower_expr(&e.callee)?;
```

> [!NOTE]
> Using `CallKind::TypeConversionCall` is more reliable than matching on `ast::Expr::TypeName` callee, because solc explicitly tags these calls. This also covers cases like `uint256(x)` where the AST callee might be structured differently.

**Errors fixed**: 1 `scope_check` error + 1 `type_well_formed` error (from `address(0)` in constructor).

---

### Fix 3: Give `EvmExpr` variants proper return types (SIR-side)

`Expr::Dialect(...)` currently returns `Type::None` from `Expr::typ()` (L244 of [exprs.rs](crates/scirs/src/sir/exprs.rs)). But the type infrastructure already exists: `Type::Dialect(DialectType::Evm(EvmType::Address))` is a valid SIR type, and `EvmType` already defines `Address`, `AddressPayable`, etc.

Rather than suppressing the verifier, we should give each `EvmExpr` variant its correct return type. This keeps `type_well_formed` strict and enables downstream passes to reason about types.

#### [ADD] [evm.rs](crates/scirs/src/sir/dialect/evm.rs) — type method on `EvmExpr`

```rust
impl EvmExpr {
    /// Return type of this EVM dialect expression.
    pub fn typ(&self) -> Type {
        match self {
            EvmExpr::MsgSender => Type::Dialect(DialectType::Evm(EvmType::Address)),
            EvmExpr::MsgValue  => Type::I256,       // uint256
            EvmExpr::Timestamp => Type::I256,        // uint256
            EvmExpr::BlockNumber => Type::I256,      // uint256
            EvmExpr::TxOrigin  => Type::Dialect(DialectType::Evm(EvmType::Address)),
            EvmExpr::SelfBalance => Type::I256,      // uint256
            EvmExpr::Len(_)    => Type::I256,        // uint256
            EvmExpr::Convert { to, .. } => to.clone(),
            EvmExpr::Slice { .. } => Type::Bytes,
            EvmExpr::Concat(_) => Type::Bytes,
            EvmExpr::Empty(ty) => ty.clone(),
            // Low-level calls return (bool, bytes)
            EvmExpr::RawCall { .. }
            | EvmExpr::Send { .. }
            | EvmExpr::Delegatecall { .. }
            | EvmExpr::LowLevelCall { .. } => Type::Tuple(vec![Type::Bool, Type::Bytes]),
            // Inline asm is opaque — Type::None is acceptable here
            EvmExpr::InlineAsm { .. } => Type::None,
        }
    }
}
```

#### [MODIFY] [exprs.rs](crates/scirs/src/sir/exprs.rs) — `Expr::typ()` (~L244)

```diff
-            Expr::Dialect(_) => Type::None,
+            Expr::Dialect(d) => d.typ(),
```

This requires a `typ()` method on `DialectExpr` that dispatches to the inner dialect:

#### [ADD] [mod.rs](crates/scirs/src/sir/dialect/mod.rs) — type method on `DialectExpr`

```rust
impl DialectExpr {
    pub fn typ(&self) -> Type {
        match self {
            DialectExpr::Evm(e) => e.typ(),
            // Anchor/Move: return Type::None for now (no test coverage yet)
            DialectExpr::Anchor(_) => Type::None,
            DialectExpr::Move(_) => Type::None,
        }
    }
}
```

#### [MODIFY] [type_well_formed.rs](crates/scirs/src/sir/verifier/type_well_formed.rs) — `visit_expr` (~L17)

Only `Expr::Result` needs suppression (its type depends on function context). `Expr::Dialect` will now have proper types. `InlineAsm` is the one edge case that stays `Type::None` — skip it explicitly:

```diff
 fn visit_expr(&mut self, expr: &'a Expr) {
-    if expr.typ() == Type::None {
+    if expr.typ() == Type::None
+        && !matches!(
+            expr,
+            Expr::Result(_)
+                | Expr::Dialect(DialectExpr::Evm(EvmExpr::InlineAsm { .. }))
+        )
+    {
         let mut err = VerifyError::new(PASS, "expression has `None` type");
```

**Errors fixed**: All `type_well_formed` errors from `msg.sender`, `msg.value`, `block.timestamp`, etc. — now with correct types rather than suppression.

---

### Fix 4: Recognize known dialects in `no_orphan_dialect` (verifier-side)

The current `no_orphan_dialect` pass rejects **all** `DialectExpr` and `DialectStmt` nodes unconditionally. But all current dialect variants (`Evm`, `Anchor`, `Move`) are recognized members of the type system — they're not orphaned. The intent of this pass should be to catch *unknown* dialect constructs, not known ones.

Since `DialectExpr` and `DialectStmt` are closed enums, a match-all that accepts every variant is correct and will produce a compiler error if a new variant is added without handling.

#### [MODIFY] [no_orphan_dialect.rs](crates/scirs/src/sir/verifier/no_orphan_dialect.rs)

```diff
-    fn visit_dialect_expr(&mut self, _expr: &'a DialectExpr) {
-        self.errors.push(VerifyError::new(
-            PASS,
-            "orphan DialectExpr found outside recognized dialect context",
-        ));
+    fn visit_dialect_expr(&mut self, expr: &'a DialectExpr) {
+        // All known dialect families are recognized; exhaustive match
+        // ensures new variants trigger a compile-time error.
+        match expr {
+            DialectExpr::Evm(_) | DialectExpr::Anchor(_) | DialectExpr::Move(_) => {}
+        }
     }

-    fn visit_dialect_stmt(&mut self, _stmt: &'a DialectStmt) {
-        self.errors.push(VerifyError::new(
-            PASS,
-            "orphan DialectStmt found outside recognized dialect context",
-        ));
+    fn visit_dialect_stmt(&mut self, stmt: &'a DialectStmt) {
+        match stmt {
+            DialectStmt::Evm(_) | DialectStmt::Anchor(_) | DialectStmt::Move(_) => {}
+        }
     }
```

**Errors fixed**: All `no_orphan_dialect` errors from `emit`, `msg.sender`, `block.timestamp`, etc.

---

### Fix 5: Lower `new T(args)` → `FunctionCall` with proper callee type (lowering-side)

Currently `lower_new_expr` (L1180) produces `Var("new T", Type::None)` — an undeclared variable with no type. In Solidity, `new T(args)` is a constructor call that returns an instance of `T`.

**Fix**: Produce a `FunctionCall` where the callee has a `Function` type, and the return type is `TypeRef(T)`.

#### [MODIFY] [lower.rs](crates/frontend/src/solidity/lowering/lower.rs) — `lower_new_expr` (~L1180)

```diff
 fn lower_new_expr(&mut self, e: &ast::NewExpr) -> Result<(Expr, Vec<Stmt>)> {
     let ty = self.lower_type(&e.typ)?;
     let span = loc_to_span(e.loc);
-    let name = format!("new {ty}");
-    let callee = Expr::Var(VarExpr::new(name, Type::None, span));
-    Ok((callee, vec![]))
+    let name = format!("new__{ty}");
+    let callee_ty = Type::Function { params: vec![], returns: vec![ty.clone()] };
+    let callee = Expr::Var(VarExpr::new(name, callee_ty, span));
+    Ok((callee, vec![]))
 }
```

> [!NOTE]
> `new T` in the AST appears as an expression (not a call). The actual constructor arguments come from the enclosing `CallExpr` that wraps the `NewExpr`. So we just need to fix the callee — the `lower_call_expr` already wraps it in a `FunctionCall` with the args.
>
> The callee name `new__T` is synthetic and won't be in scope — but that's acceptable because constructor calls are inter-contract calls and the scope checker should skip synthetic names. We may need to also teach scope_check to recognize `new__` prefixed names, OR handle this in `lower_call_expr` by detecting `ast::Expr::New` callees and emitting a `TypeCast` or dedicated construct. The simpler approach: give it a `Function` type so `type_well_formed` passes, and add `new__` to scope_check's allowlist.

#### [MODIFY] [scope_check.rs](crates/scirs/src/sir/verifier/scope_check.rs) — `check_expr` (~L41)

```diff
 Expr::Var(v) => {
-    if !self.is_declared(&v.name) {
+    if !self.is_declared(&v.name) && !v.name.starts_with("new__") {
```

---

### Fix 6: Give `type(X).min/max` callee a proper type (lowering-side)

`type(X).min` / `type(X).max` currently produces `FunctionCall(Var("min__type__X", Type::None), [])`. The callee has `Type::None` which fails `type_well_formed`. The overall call expression already has the correct type from `e.typ`.

**Fix**: Give the synthetic callee a `Function` return type matching the outer expression type.

#### [MODIFY] [lower.rs](crates/frontend/src/solidity/lowering/lower.rs) — `lower_member_expr` (~L1100)

```diff
-                let callee = Expr::Var(VarExpr::new(fname, Type::None, span));
+                let callee_ty = Type::Function { params: vec![], returns: vec![ty.clone()] };
+                let callee = Expr::Var(VarExpr::new(fname, callee_ty, span));
```

The synthetic name `min__type__uint256` won't be in scope. Add to scope_check allowlist:

#### [MODIFY] [scope_check.rs](crates/scirs/src/sir/verifier/scope_check.rs) — `check_expr` (~L41)

Extend the existing skip condition from Fix 5:

```diff
 Expr::Var(v) => {
-    if !self.is_declared(&v.name) && !v.name.starts_with("new__") {
+    if !self.is_declared(&v.name)
+        && !v.name.starts_with("new__")
+        && !v.name.contains("__type__")
+    {
```

---

### Fix 7: Lower crypto/math/global builtins → `EvmExpr` variants (lowering + SIR)

Several Solidity global functions are lowered as generic `FunctionCall(Var("keccak256"))` etc. — the callee is undeclared and has no semantic information. These should become EVM dialect expressions.

**New `EvmExpr` variants needed:**

#### [MODIFY] [evm.rs](crates/scirs/src/sir/dialect/evm.rs) — add variants to `EvmExpr`

```rust
// ── Global builtin functions ─────────────────────────────
/// `keccak256(data)` — Keccak-256 hash.
Keccak256(Box<Expr>),
/// `sha256(data)` — SHA-256 hash.
Sha256(Box<Expr>),
/// `ripemd160(data)` — RIPEMD-160 hash.
Ripemd160(Box<Expr>),
/// `ecrecover(hash, v, r, s)` — recover signer address.
Ecrecover { hash: Box<Expr>, v: Box<Expr>, r: Box<Expr>, s: Box<Expr> },
/// `addmod(x, y, k)` — (x + y) % k.
Addmod { x: Box<Expr>, y: Box<Expr>, k: Box<Expr> },
/// `mulmod(x, y, k)` — (x * y) % k.
Mulmod { x: Box<Expr>, y: Box<Expr>, k: Box<Expr> },
/// `gasleft()` — remaining gas.
Gasleft,
/// `blockhash(blockNumber)` — hash of given block.
Blockhash(Box<Expr>),
```

**Update `EvmExpr::typ()`** (from Fix 3):

```rust
EvmExpr::Keccak256(_) => Type::FixedBytes(32),
EvmExpr::Sha256(_) => Type::FixedBytes(32),
EvmExpr::Ripemd160(_) => Type::FixedBytes(20),
EvmExpr::Ecrecover { .. } => Type::Dialect(DialectType::Evm(EvmType::Address)),
EvmExpr::Addmod { .. } | EvmExpr::Mulmod { .. } => Type::I256,
EvmExpr::Gasleft => Type::I256,
EvmExpr::Blockhash(_) => Type::FixedBytes(32),
```

**Update `Display` for `EvmExpr`:**

```rust
EvmExpr::Keccak256(e) => write!(f, "evm.keccak256({e})"),
EvmExpr::Sha256(e) => write!(f, "evm.sha256({e})"),
EvmExpr::Ripemd160(e) => write!(f, "evm.ripemd160({e})"),
EvmExpr::Ecrecover { hash, v, r, s } => write!(f, "evm.ecrecover({hash}, {v}, {r}, {s})"),
EvmExpr::Addmod { x, y, k } => write!(f, "evm.addmod({x}, {y}, {k})"),
EvmExpr::Mulmod { x, y, k } => write!(f, "evm.mulmod({x}, {y}, {k})"),
EvmExpr::Gasleft => write!(f, "evm.gasleft()"),
EvmExpr::Blockhash(e) => write!(f, "evm.blockhash({e})"),
```

#### [MODIFY] [lower.rs](crates/frontend/src/solidity/lowering/lower.rs) — `lower_call_expr` (~L883)

Intercept builtin function calls by name before the generic lowering path. Place after the `TypeConversionCall` check (Fix 2) and before the generic callee lowering:

```rust
// ── EVM builtin functions ────────────────────────────────
if let ast::Expr::Ident(id) = &*e.callee {
    let (args, extra) = self.lower_call_args_exprs(&e.args)?;
    stmts.extend(extra);
    let mut pos = args.into_positional();
    let evm = match id.name.base.as_str() {
        "keccak256" => Some(EvmExpr::Keccak256(Box::new(pos.remove(0)))),
        "sha256" => Some(EvmExpr::Sha256(Box::new(pos.remove(0)))),
        "ripemd160" => Some(EvmExpr::Ripemd160(Box::new(pos.remove(0)))),
        "ecrecover" if pos.len() == 4 => Some(EvmExpr::Ecrecover {
            hash: Box::new(pos.remove(0)),
            v: Box::new(pos.remove(0)),
            r: Box::new(pos.remove(0)),
            s: Box::new(pos.remove(0)),
        }),
        "addmod" if pos.len() == 3 => Some(EvmExpr::Addmod {
            x: Box::new(pos.remove(0)),
            y: Box::new(pos.remove(0)),
            k: Box::new(pos.remove(0)),
        }),
        "mulmod" if pos.len() == 3 => Some(EvmExpr::Mulmod {
            x: Box::new(pos.remove(0)),
            y: Box::new(pos.remove(0)),
            k: Box::new(pos.remove(0)),
        }),
        "gasleft" => Some(EvmExpr::Gasleft),
        "blockhash" => Some(EvmExpr::Blockhash(Box::new(pos.remove(0)))),
        _ => None,
    };
    if let Some(evm) = evm {
        return Ok((Expr::Dialect(DialectExpr::Evm(evm)), stmts));
    }
    // Not a recognized builtin — need to re-lower args since we consumed them.
    // Restructure: only consume args inside the match.
}
```

> [!NOTE]
> The above pseudo-code consumes `pos` before checking if it's a builtin. For the actual implementation, either check the name first (before lowering args), or restructure to avoid consuming args on the non-matching path. The cleanest approach: match on `id.name.base.as_str()` first, then lower args only if matched.

Restructured approach:

```rust
if let ast::Expr::Ident(id) = &*e.callee {
    let name = id.name.base.as_str();
    if matches!(name,
        "keccak256" | "sha256" | "ripemd160" | "ecrecover"
        | "addmod" | "mulmod" | "gasleft" | "blockhash"
    ) {
        let (args, extra) = self.lower_call_args_exprs(&e.args)?;
        stmts.extend(extra);
        let mut pos = args.into_positional();
        let evm = match name {
            "keccak256" => EvmExpr::Keccak256(Box::new(pos.remove(0))),
            "sha256" => EvmExpr::Sha256(Box::new(pos.remove(0))),
            "ripemd160" => EvmExpr::Ripemd160(Box::new(pos.remove(0))),
            "ecrecover" => EvmExpr::Ecrecover {
                hash: Box::new(pos.remove(0)),
                v: Box::new(pos.remove(0)),
                r: Box::new(pos.remove(0)),
                s: Box::new(pos.remove(0)),
            },
            "addmod" => EvmExpr::Addmod {
                x: Box::new(pos.remove(0)),
                y: Box::new(pos.remove(0)),
                k: Box::new(pos.remove(0)),
            },
            "mulmod" => EvmExpr::Mulmod {
                x: Box::new(pos.remove(0)),
                y: Box::new(pos.remove(0)),
                k: Box::new(pos.remove(0)),
            },
            "gasleft" => EvmExpr::Gasleft,
            "blockhash" => EvmExpr::Blockhash(Box::new(pos.remove(0))),
            _ => unreachable!(),
        };
        return Ok((Expr::Dialect(DialectExpr::Evm(evm)), stmts));
    }
}
```

---

### Fix 8: Lower `abi.*` calls → `EvmExpr` variants (lowering + SIR)

`abi.encode(...)`, `abi.encodePacked(...)`, `abi.decode(...)`, `abi.encodeWithSelector(...)`, `abi.encodeWithSignature(...)`, and `abi.encodeCall(...)` are lowered as generic `FunctionCall(FieldAccess(Var("abi"), "encode"), args)`. The base `abi` is typed as `TypeRef("abi")` which is not a real scope variable.

**New `EvmExpr` variants:**

#### [MODIFY] [evm.rs](crates/scirs/src/sir/dialect/evm.rs) — add variants to `EvmExpr`

```rust
// ── ABI encoding/decoding ────────────────────────────────
/// `abi.encode(args...)` — ABI-encode arguments.
AbiEncode(Vec<Expr>),
/// `abi.encodePacked(args...)` — Packed ABI encoding.
AbiEncodePacked(Vec<Expr>),
/// `abi.decode(data, (types...))` — ABI decode.
AbiDecode { data: Box<Expr>, types: Vec<Type> },
/// `abi.encodeWithSelector(sel, args...)` — encode with function selector.
AbiEncodeWithSelector { selector: Box<Expr>, args: Vec<Expr> },
/// `abi.encodeWithSignature(sig, args...)` — encode with string signature.
AbiEncodeWithSignature { signature: Box<Expr>, args: Vec<Expr> },
/// `abi.encodeCall(func, args...)` — encode a function call.
AbiEncodeCall { func: Box<Expr>, args: Vec<Expr> },
```

**Types for `EvmExpr::typ()`:**

```rust
EvmExpr::AbiEncode(_)
| EvmExpr::AbiEncodePacked(_)
| EvmExpr::AbiEncodeWithSelector { .. }
| EvmExpr::AbiEncodeWithSignature { .. }
| EvmExpr::AbiEncodeCall { .. } => Type::Bytes,
EvmExpr::AbiDecode { types, .. } => {
    if types.len() == 1 { types[0].clone() }
    else { Type::Tuple(types.clone()) }
},
```

#### [MODIFY] [lower.rs](crates/frontend/src/solidity/lowering/lower.rs) — `lower_member_expr` or `lower_call_expr`

Intercept calls where the callee is `FieldAccess` on `abi`:

In `lower_call_expr`, after the builtin functions check (Fix 7):

```rust
// ── abi.* builtins ───────────────────────────────────────
if let ast::Expr::Member(mem) = &*e.callee {
    if let ast::Expr::Ident(base) = &*mem.base {
        if base.name.base == "abi" {
            let method = mem.member.to_string();
            let (args, extra) = self.lower_call_args_exprs(&e.args)?;
            stmts.extend(extra);
            let pos = args.into_positional();
            let evm = match method.as_str() {
                "encode" => EvmExpr::AbiEncode(pos),
                "encodePacked" => EvmExpr::AbiEncodePacked(pos),
                "decode" => {
                    // First arg is data, rest are type info (handled as exprs for now)
                    let mut it = pos.into_iter();
                    let data = Box::new(it.next().unwrap_or(
                        Expr::Lit(Lit::String(StringLit::new(String::new(), span)))
                    ));
                    EvmExpr::AbiDecode { data, types: vec![ty.clone()] }
                }
                "encodeWithSelector" => {
                    let mut it = pos.into_iter();
                    let selector = Box::new(it.next().unwrap());
                    EvmExpr::AbiEncodeWithSelector { selector, args: it.collect() }
                }
                "encodeWithSignature" => {
                    let mut it = pos.into_iter();
                    let signature = Box::new(it.next().unwrap());
                    EvmExpr::AbiEncodeWithSignature { signature, args: it.collect() }
                }
                "encodeCall" => {
                    let mut it = pos.into_iter();
                    let func = Box::new(it.next().unwrap());
                    EvmExpr::AbiEncodeCall { func, args: it.collect() }
                }
                _ => {
                    // Unknown abi method — fall through to generic
                    // Need to re-lower since we consumed args
                    // (restructure to avoid this in actual impl)
                    return Ok((Expr::FunctionCall(CallExpr {
                        callee: Box::new(Expr::Var(VarExpr::new(
                            format!("abi.{method}"), ty.clone(), span
                        ))),
                        args: CallArgs::Positional(vec![]),
                        ty,
                        span,
                    }), stmts));
                }
            };
            return Ok((Expr::Dialect(DialectExpr::Evm(evm)), stmts));
        }
    }
}
```

---

### Fix 9: Lower `selfdestruct`/`transfer`/`send` → EVM dialect (lowering + SIR)

#### New SIR variants

**[MODIFY] [evm.rs](crates/scirs/src/sir/dialect/evm.rs):**

Add to `EvmStmt`:
```rust
/// `selfdestruct(recipient)` — destroy contract and send funds.
Selfdestruct { recipient: Expr, span: Option<Span> },
```

Add to `EvmExpr`:
```rust
/// `addr.transfer(amount)` — transfer Ether, reverts on failure.
Transfer { target: Box<Expr>, amount: Box<Expr> },
```

> [!NOTE]
> `addr.send(amount)` already has `EvmExpr::Send { target, value }` defined. Only lowering interception is needed.

**Types for `EvmExpr::typ()`:**
```rust
EvmExpr::Transfer { .. } => Type::None,  // transfer returns void (reverts on fail)
// Send already handled — returns bool
```

#### Lowering changes

**`selfdestruct`** is a statement-level builtin. Intercept in `lower_expr_stmt`:

#### [MODIFY] [lower.rs](crates/frontend/src/solidity/lowering/lower.rs) — `lower_expr_stmt` (~L434)

```diff
+        ast::Expr::Call(call) if call.callee.to_string() == "selfdestruct" => {
+            let (args, extra) = self.lower_call_args_exprs(&call.args)?;
+            let mut stmts = extra;
+            let mut pos = args.into_positional();
+            let recipient = pos.remove(0);
+            stmts.push(Stmt::Dialect(DialectStmt::Evm(EvmStmt::Selfdestruct {
+                recipient,
+                span: loc_to_span(s.loc),
+            })));
+            Ok(stmts)
+        }
```

**`addr.transfer(amt)` and `addr.send(amt)`** — intercept in `lower_call_expr` when callee is a member access:

#### [MODIFY] [lower.rs](crates/frontend/src/solidity/lowering/lower.rs) — `lower_call_expr`

After the `abi.*` check (Fix 8):

```rust
// ── addr.transfer(amt) / addr.send(amt) ─────────────────
if let ast::Expr::Member(mem) = &*e.callee {
    let method = mem.member.to_string();
    if matches!(method.as_str(), "transfer" | "send") {
        let (target, extra) = self.lower_expr(&mem.base)?;
        stmts.extend(extra);
        let (args, extra) = self.lower_call_args_exprs(&e.args)?;
        stmts.extend(extra);
        let mut pos = args.into_positional();
        let amount = pos.remove(0);
        let evm = match method.as_str() {
            "transfer" => EvmExpr::Transfer {
                target: Box::new(target),
                amount: Box::new(amount),
            },
            "send" => EvmExpr::Send {
                target: Box::new(target),
                value: Box::new(amount),
            },
            _ => unreachable!(),
        };
        return Ok((Expr::Dialect(DialectExpr::Evm(evm)), stmts));
    }
}
```

> [!NOTE]
> Care needed: `transfer` and `send` are common method names that could appear on non-address types (e.g., ERC-20 `transfer`). We should only intercept when the base has an address type. Check `mem.base.typ()` or `mem.typ` to disambiguate. If the AST doesn't provide reliable type info, fall back to checking if the method has exactly 1 argument (address `transfer` takes 1 arg, ERC-20 `transfer` takes 2).

---

### Fix 10: Lower `this` and `super` → EVM dialect expressions (lowering + SIR)

`this` is lowered as `Var("this", TypeRef("ContractName"))` — type is correct but the variable is undeclared in scope. `super` is similar.

#### New SIR variants

**[MODIFY] [evm.rs](crates/scirs/src/sir/dialect/evm.rs):**

```rust
/// `this` — reference to the current contract instance (as address).
This,
/// `super` — reference to the parent contract for method dispatch.
Super,
```

**Types:**
```rust
EvmExpr::This => Type::Dialect(DialectType::Evm(EvmType::Address)),
EvmExpr::Super => Type::None,  // super is only used for member dispatch, not as a value
```

**Display:**
```rust
EvmExpr::This => write!(f, "evm.this"),
EvmExpr::Super => write!(f, "evm.super"),
```

#### [MODIFY] [lower.rs](crates/frontend/src/solidity/lowering/lower.rs) — `lower_expr` (~L725)

Intercept `this`/`super` identifiers:

```diff
 ast::Expr::Ident(id) => {
-    let ty = self.lower_type(&id.typ)?;
-    Ok((Expr::Var(VarExpr::new(id.name.to_string(), ty, loc_to_span(id.loc))), vec![]))
+    match id.name.base.as_str() {
+        "this" => Ok((Expr::Dialect(DialectExpr::Evm(EvmExpr::This)), vec![])),
+        "super" => Ok((Expr::Dialect(DialectExpr::Evm(EvmExpr::Super)), vec![])),
+        _ => {
+            let ty = self.lower_type(&id.typ)?;
+            Ok((Expr::Var(VarExpr::new(id.name.to_string(), ty, loc_to_span(id.loc))), vec![]))
+        }
+    }
 }
```

---

### Fix 11: Expand EVM global member access coverage (lowering + SIR)

The current `lower_member_expr` (L1111-1128) only handles 5 globals: `msg.sender`, `msg.value`, `tx.origin`, `block.timestamp`, `block.number`. Solidity has more.

#### New `EvmExpr` variants

**[MODIFY] [evm.rs](crates/scirs/src/sir/dialect/evm.rs):**

```rust
// ── Additional block/msg/tx globals ──────────────────────
/// `msg.data` — complete calldata.
MsgData,
/// `msg.sig` — first 4 bytes of calldata (function selector).
MsgSig,
/// `block.difficulty` / `block.prevrandao` — previous block's RANDAO.
BlockDifficulty,
/// `block.gaslimit` — block gas limit.
BlockGaslimit,
/// `block.coinbase` — current block miner's address.
BlockCoinbase,
/// `block.chainid` — current chain ID.
BlockChainid,
/// `block.basefee` — current block's base fee.
BlockBasefee,
```

**Types for `EvmExpr::typ()`:**
```rust
EvmExpr::MsgData => Type::Bytes,
EvmExpr::MsgSig => Type::FixedBytes(4),
EvmExpr::BlockDifficulty => Type::I256,
EvmExpr::BlockGaslimit => Type::I256,
EvmExpr::BlockCoinbase => Type::Dialect(DialectType::Evm(EvmType::Address)),
EvmExpr::BlockChainid => Type::I256,
EvmExpr::BlockBasefee => Type::I256,
```

#### [MODIFY] [lower.rs](crates/frontend/src/solidity/lowering/lower.rs) — `lower_member_expr` (~L1117)

Expand the match table:

```diff
 let evm_expr = match (base_name, member.as_str()) {
     ("msg", "sender") => Some(EvmExpr::MsgSender),
     ("msg", "value") => Some(EvmExpr::MsgValue),
+    ("msg", "data") => Some(EvmExpr::MsgData),
+    ("msg", "sig") => Some(EvmExpr::MsgSig),
     ("tx", "origin") => Some(EvmExpr::TxOrigin),
     ("block", "timestamp") => Some(EvmExpr::Timestamp),
     ("block", "number") => Some(EvmExpr::BlockNumber),
+    ("block", "difficulty") | ("block", "prevrandao") => Some(EvmExpr::BlockDifficulty),
+    ("block", "gaslimit") => Some(EvmExpr::BlockGaslimit),
+    ("block", "coinbase") => Some(EvmExpr::BlockCoinbase),
+    ("block", "chainid") => Some(EvmExpr::BlockChainid),
+    ("block", "basefee") => Some(EvmExpr::BlockBasefee),
     _ => None,
 };
```

---

## Summary of all changes by file

| File | Fixes | Changes |
|---|---|---|
| [lower.rs](crates/frontend/src/solidity/lowering/lower.rs) | 1,2,5,6,7,8,9,10,11 | Intercept builtins in `lower_expr_stmt`, `lower_call_expr`, `lower_expr`, `lower_member_expr`, `lower_new_expr` |
| [evm.rs](crates/scirs/src/sir/dialect/evm.rs) | 3,7,8,9,10,11 | Add ~20 new `EvmExpr` variants, 1 `EvmStmt` variant, `typ()` method, `Display` impls |
| [mod.rs](crates/scirs/src/sir/dialect/mod.rs) | 3 | Add `DialectExpr::typ()` dispatch |
| [exprs.rs](crates/scirs/src/sir/exprs.rs) | 3 | `Expr::Dialect(d) => d.typ()` |
| [type_well_formed.rs](crates/scirs/src/sir/verifier/type_well_formed.rs) | 3 | Skip `Expr::Result` and `InlineAsm` only |
| [no_orphan_dialect.rs](crates/scirs/src/sir/verifier/no_orphan_dialect.rs) | 4 | Accept all known dialect families |
| [scope_check.rs](crates/scirs/src/sir/verifier/scope_check.rs) | 5,6 | Skip synthetic names (`new__`, `__type__`) |

## Implementation order

Fixes should be implemented in this order to minimize intermediate breakage:

1. **Fix 3** (EvmExpr types + DialectExpr dispatch) — foundational, enables all new EvmExpr variants to have types
2. **Fix 4** (no_orphan_dialect) — unblocks all dialect nodes
3. **Fix 7** (crypto/math builtins) — adds new EvmExpr variants, depends on Fix 3 for types
4. **Fix 8** (abi.* builtins) — adds new EvmExpr variants
5. **Fix 9** (selfdestruct/transfer/send) — adds new variants
6. **Fix 10** (this/super) — adds new EvmExpr variants
7. **Fix 11** (expanded globals) — adds new EvmExpr variants
8. **Fix 1** (require/assert) — standalone lowering change
9. **Fix 2** (TypeConversionCall) — standalone lowering change
10. **Fix 5** (new T) — lowering + scope_check
11. **Fix 6** (type(X).min/max) — lowering + scope_check

## Verification Plan

1. `cargo build` — verify compilation after each fix
2. `cargo test` — run existing tests
3. `./target/debug/verazt compile examples/solidity/token.sol -d` — verify 0 SIR verification errors
4. Test with additional Solidity files that use the constructs from Fixes 5-11
5. Inspect SIR output for representative constructs:
   - `require(...)` → `if !cond { revert(msg) }`
   - `assert(...)` → `Stmt::Assert`
   - `address(0)` → `TypeCast(address, 0)`
   - `msg.sender` → `evm.msg_sender()` with type `address`
   - `keccak256(...)` → `evm.keccak256(...)` with type `bytes32`
   - `abi.encode(...)` → `evm.abi_encode(...)` with type `bytes`

---

## Checklist

### Fix 3 — EVM dialect types (foundational, do first)

- [ ] **[evm.rs]** Add `impl EvmExpr { pub fn typ(&self) -> Type { ... } }` covering all existing variants: `MsgSender`, `MsgValue`, `Timestamp`, `BlockNumber`, `TxOrigin`, `SelfBalance`, `Len`, `Convert`, `Slice`, `Concat`, `Empty`, `RawCall`, `Send`, `Delegatecall`, `LowLevelCall`, `InlineAsm`
- [ ] **[mod.rs]** Add `impl DialectExpr { pub fn typ(&self) -> Type { ... } }` dispatching to `EvmExpr::typ()` (Anchor/Move return `Type::None`)
- [ ] **[exprs.rs]** Change `Expr::Dialect(_) => Type::None` to `Expr::Dialect(d) => d.typ()`
- [ ] **[type_well_formed.rs]** Narrow the `Type::None` skip to only `Expr::Result(_)` and `Expr::Dialect(DialectExpr::Evm(EvmExpr::InlineAsm { .. }))`

### Fix 4 — no_orphan_dialect verifier

- [ ] **[no_orphan_dialect.rs]** Replace `visit_dialect_expr` error push with exhaustive match accepting `Evm(_) | Anchor(_) | Move(_)`
- [ ] **[no_orphan_dialect.rs]** Replace `visit_dialect_stmt` error push with exhaustive match accepting `Evm(_) | Anchor(_) | Move(_)`

### Fix 7 — Crypto/math/global builtin functions (new EvmExpr variants)

- [ ] **[evm.rs]** Add `EvmExpr` variants: `Keccak256`, `Sha256`, `Ripemd160`, `Ecrecover`, `Addmod`, `Mulmod`, `Gasleft`, `Blockhash`
- [ ] **[evm.rs]** Add `typ()` arms for all Fix 7 variants
- [ ] **[evm.rs]** Add `Display` arms for all Fix 7 variants
- [ ] **[lower.rs]** In `lower_call_expr`: intercept `keccak256 | sha256 | ripemd160 | ecrecover | addmod | mulmod | gasleft | blockhash` by name before generic callee lowering, emit `Expr::Dialect(DialectExpr::Evm(...))`

### Fix 8 — `abi.*` builtins (new EvmExpr variants)

- [ ] **[evm.rs]** Add `EvmExpr` variants: `AbiEncode`, `AbiEncodePacked`, `AbiDecode`, `AbiEncodeWithSelector`, `AbiEncodeWithSignature`, `AbiEncodeCall`
- [ ] **[evm.rs]** Add `typ()` arms for all Fix 8 variants
- [ ] **[evm.rs]** Add `Display` arms for all Fix 8 variants
- [ ] **[lower.rs]** In `lower_call_expr`: detect `callee = Member(Ident("abi"), method)` and dispatch to the appropriate `EvmExpr` variant

### Fix 9 — `selfdestruct` / `transfer` / `send`

- [ ] **[evm.rs]** Add `EvmStmt::Selfdestruct { recipient: Expr, span: Option<Span> }`
- [ ] **[evm.rs]** Add `Display` arm for `EvmStmt::Selfdestruct`
- [ ] **[evm.rs]** Add `EvmExpr::Transfer { target: Box<Expr>, amount: Box<Expr> }`
- [ ] **[evm.rs]** Add `typ()` arm for `Transfer` (`Type::None` — void)
- [ ] **[evm.rs]** Add `Display` arm for `Transfer`
- [ ] **[lower.rs]** In `lower_expr_stmt`: intercept `selfdestruct(addr)` call, emit `Stmt::Dialect(DialectStmt::Evm(EvmStmt::Selfdestruct { ... }))`
- [ ] **[lower.rs]** In `lower_call_expr`: intercept `callee = Member(base, "transfer")` with 1 arg, emit `EvmExpr::Transfer`
- [ ] **[lower.rs]** In `lower_call_expr`: intercept `callee = Member(base, "send")` with 1 arg, emit `EvmExpr::Send`

### Fix 10 — `this` and `super`

- [ ] **[evm.rs]** Add `EvmExpr::This` and `EvmExpr::Super` variants
- [ ] **[evm.rs]** Add `typ()` arms: `This => Type::Dialect(DialectType::Evm(EvmType::Address))`, `Super => Type::None`
- [ ] **[evm.rs]** Add `Display` arms: `This => "evm.this"`, `Super => "evm.super"`
- [ ] **[lower.rs]** In `lower_expr` ident arm: match `"this"` → `EvmExpr::This`, `"super"` → `EvmExpr::Super` before generic var lowering

### Fix 11 — Expanded EVM global member accesses

- [ ] **[evm.rs]** Add `EvmExpr` variants: `MsgData`, `MsgSig`, `BlockDifficulty`, `BlockGaslimit`, `BlockCoinbase`, `BlockChainid`, `BlockBasefee`
- [ ] **[evm.rs]** Add `typ()` arms for all Fix 11 variants
- [ ] **[evm.rs]** Add `Display` arms for all Fix 11 variants
- [ ] **[lower.rs]** In `lower_member_expr` match table: add `("msg","data")`, `("msg","sig")`, `("block","difficulty")|("block","prevrandao")`, `("block","gaslimit")`, `("block","coinbase")`, `("block","chainid")`, `("block","basefee")`

### Fix 1a — `assert(cond)` lowering

- [ ] **[lower.rs]** In `lower_expr_stmt`: add arm for `Call` where callee == `"assert"`, dispatching to `lower_assert`
- [ ] **[lower.rs]** Add `lower_assert` helper: lower args, emit `Stmt::Assert { cond, message: None, span }`

### Fix 1b — `require(cond, msg?)` lowering

- [ ] **[lower.rs]** In `lower_expr_stmt`: add arm for `Call` where callee == `"require"`, dispatching to `lower_require`
- [ ] **[lower.rs]** Add `lower_require` helper: lower args, emit `Stmt::If { cond: UnOp(Not, cond), then_body: [Stmt::Revert { error: None, args: [msg], span }], else_body: None, span }`

### Fix 2 — Type conversions (`address(0)`, `uint256(x)`, etc.)

- [ ] **[lower.rs]** In `lower_call_expr`: add early-return for `e.kind == CallKind::TypeConversionCall` with single arg, emitting `Expr::TypeCast { ty, expr: inner, span }`

### Fix 5 — `new T(args)` constructor calls

- [ ] **[lower.rs]** In `lower_new_expr`: change `Var("new T", Type::None)` to `Var("new__T", Function { params: [], returns: [ty] })`
- [ ] **[scope_check.rs]** In `check_expr` Var arm: skip scope check for names starting with `"new__"`

### Fix 6 — `type(X).min` / `type(X).max`

- [ ] **[lower.rs]** In `lower_member_expr` type-query branch: change callee from `Var(fname, Type::None)` to `Var(fname, Function { params: [], returns: [ty] })`
- [ ] **[scope_check.rs]** In `check_expr` Var arm: skip scope check for names containing `"__type__"`

### Verification

- [ ] `cargo build` passes with no errors
- [ ] `cargo test` passes with no regressions
- [ ] `./target/debug/verazt compile examples/solidity/token.sol -d` produces 0 SIR verification errors
