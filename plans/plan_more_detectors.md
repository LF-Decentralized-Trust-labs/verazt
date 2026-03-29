# Plan: Implement Missing SmartBugs-Curated Detectors

## 1. Coverage Gap Analysis

### SmartBugs-Curated Dataset (10 categories, ~143 annotated .sol files)

| # | SmartBugs Category | Annotation | Files | Existing Detectors | Status |
|---|---|---|---|---|---|
| 1 | `reentrancy/` | `REENTRANCY` | 31 | `ReentrancySirDetector`, `CeiViolationSirDetector` | ✅ Covered |
| 2 | `access_control/` | `ACCESS_CONTROL` | 19 | `MissingAccessControlSirDetector`, `DelegatecallSirDetector`, `TxOriginSirDetector`, `VisibilitySirDetector`, `CentralizationRiskSirDetector` | ✅ Covered |
| 3 | `unchecked_low_level_calls/` | `UNCHECKED_LL_CALLS` | 52 | `UncheckedCallSirDetector`, `LowLevelCallSirDetector` | ✅ Covered |
| 4 | `time_manipulation/` | `TIME_MANIPULATION` | 5 | `TimestampDependenceSirDetector` | ✅ Covered |
| 5 | `other/` | `OTHER` | 3 | `UninitializedSirDetector` | ✅ Covered |
| 6 | **`arithmetic/`** | `ARITHMETIC` | **16** | — | ❌ **Missing** |
| 7 | **`bad_randomness/`** | `BAD_RANDOMNESS` | **8** | — | ❌ **Missing** |
| 8 | **`denial_of_service/`** | `DENIAL_OF_SERVICE` | **6** | — | ❌ **Missing** |
| 9 | **`front_running/`** | `FRONT_RUNNING` | **4** | — | ❌ **Missing** |
| 10 | **`short_addresses/`** | `SHORT_ADDRESSES` | **1** | — | ❌ **Missing** |

**Summary**: 5 of 10 SmartBugs categories have **zero detector coverage**. These 5 missing
categories account for **35 vulnerable contract files** and **~55+ annotated vulnerability
instances**.

### Existing Detectors Not Mapped to SmartBugs

4 detectors target `CodeQuality` issues that are not part of the SmartBugs-curated taxonomy but
provide useful code-quality checks: `FloatingPragmaSirDetector`, `DeprecatedSirDetector`,
`DeadCodeSirDetector`, `ShadowingSirDetector`, `ConstantStateVarSirDetector`.

---

## 2. New Detectors to Implement

### 2.1 Integer Overflow/Underflow Detector (`ArithmeticOverflowSirDetector`)

**SmartBugs category**: `arithmetic/` (16 files, ~22 annotated instances)
**SWC**: SWC-101 (Integer Overflow and Underflow)
**CWE**: CWE-190 (Integer Overflow), CWE-191 (Integer Underflow)
**Risk**: High

#### Detection Strategy

The SIR already encodes overflow semantics on `BinOpExpr` via the `overflow: OverflowSemantics`
field. Solidity <0.8 uses `Wrapping` semantics; Solidity ≥0.8 uses `Checked` by default.

1. **Primary pattern**: Walk all `BinOpExpr` nodes with arithmetic ops (`Add`, `Sub`, `Mul`, `Pow`)
   and `overflow == OverflowSemantics::Wrapping`. Flag these as potential overflow/underflow.
2. **AugAssign pattern**: Also check `AugAssignStmt` nodes (`+=`, `-=`, `*=`) where the underlying
   operation would use wrapping semantics.
3. **False-positive reduction**:
   - Skip operations inside `SafeMath` library functions (detect by function name pattern or by
     checking if the result is guarded by a `require` assertion).
   - Skip operations where both operands are literals (compile-time computable).
   - Skip operations on `bool` or small constant ranges.

#### Files to Modify/Create

| File | Action |
|------|--------|
| `crates/analyzer/src/detectors/sir/arithmetic_overflow.rs` | **[NEW]** Detector implementation |
| `crates/analyzer/src/detectors/sir/mod.rs` | Add `pub mod arithmetic_overflow` + re-export |
| `crates/analyzer/src/detectors/mod.rs` | Add re-export |
| `crates/analyzer/src/detectors/base/id.rs` | Add `ArithmeticOverflow` variant to `DetectorId` |
| `crates/analyzer/src/detectors/base/registry.rs` | Register the new detector |

#### Sample Detection Target

```solidity
// From integer_overflow_add.sol
function run(uint256 input) public {
    // <yes> <report> ARITHMETIC
    count += input;  // Wrapping add on uint256
}
```

---

### 2.2 Bad Randomness Detector (`BadRandomnessSirDetector`)

**SmartBugs category**: `bad_randomness/` (8 files, ~18 annotated instances)
**SWC**: SWC-120 (Weak Sources of Randomness from Chain Attributes)
**CWE**: CWE-330 (Use of Insufficiently Random Values)
**Risk**: High

#### Detection Strategy

Detect use of on-chain attributes as sources of randomness. These are predictable by miners and
other chain participants.

1. **Identify randomness-source expressions**: Walk the SIR tree for EVM dialect expressions:
   - `EvmExpr::Blockhash(…)` — `block.blockhash(n)` / `blockhash(n)`
   - `EvmExpr::Timestamp(…)` — `block.timestamp` / `now`
   - `EvmExpr::BlockNumber(…)` — `block.number`
   - `EvmExpr::BlockDifficulty(…)` — `block.difficulty` / `block.prevrandao`
   - `EvmExpr::BlockCoinbase(…)` — `block.coinbase`
   - `EvmExpr::BlockGaslimit(…)` — `block.gaslimit`

2. **Check usage context**: Flag when these expressions appear as:
   - Arguments to hash functions (`EvmExpr::Keccak256`, `EvmExpr::Sha256`,
     `FieldAccess` on `sha3`/`keccak256`)
   - Operands of modulo (`BinOp::Mod`) operations
   - Initializers of variables used in conditional branches guarding ETH transfers

3. **False-positive reduction**:
   - Allow `block.timestamp` used purely for time-gating (e.g., `require(block.timestamp > X)`)
     where there is no value-derivation.
   - The existing `TimestampDependenceSirDetector` already catches basic timestamp usage; this
     detector focuses specifically on the **randomness** pattern.

#### Files to Modify/Create

| File | Action |
|------|--------|
| `crates/analyzer/src/detectors/sir/bad_randomness.rs` | **[NEW]** Detector implementation |
| `crates/analyzer/src/detectors/sir/mod.rs` | Add `pub mod bad_randomness` + re-export |
| `crates/analyzer/src/detectors/mod.rs` | Add re-export |
| `crates/analyzer/src/detectors/base/id.rs` | Add `BadRandomness` variant |
| `crates/analyzer/src/detectors/base/registry.rs` | Register the new detector |

#### Sample Detection Target

```solidity
// From guess_the_random_number.sol
answer = uint8(keccak256(block.blockhash(block.number - 1), now));
//            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// block.blockhash + now fed into keccak256 for "random" value
```

---

### 2.3 Denial of Service Detector (`DenialOfServiceSirDetector`)

**SmartBugs category**: `denial_of_service/` (6 files, ~7 annotated instances)
**SWC**: SWC-113 (DoS with Failed Call), SWC-128 (DoS with Block Gas Limit)
**CWE**: CWE-400 (Uncontrolled Resource Consumption)
**Risk**: High

#### Detection Strategy

Three sub-patterns, all detectable via SIR tree walking:

1. **External call inside loop** (SWC-113):
   Walk `ForStmt` and `WhileStmt` bodies. If any statement contains an external call
   (`is_evm_external_call()`), report. Especially flag `send`/`transfer`/`call` inside loops
   iterating over dynamic arrays.

2. **`require` wrapping `send`/`transfer`** (SWC-113):
   Find `Stmt::Expr` containing a `require(addr.send(…))` pattern. The `send` returns `bool`;
   wrapping it in `require` means a single failed send reverts the entire transaction, enabling a
   DoS by a malicious recipient.

3. **Unbounded loop over dynamic storage array** (SWC-128):
   Detect for-loops where the bound is derived from a storage array's `.length` and the loop body
   performs gas-intensive operations (writes, external calls). The loop could exceed the block gas
   limit.

#### Files to Modify/Create

| File | Action |
|------|--------|
| `crates/analyzer/src/detectors/sir/denial_of_service.rs` | **[NEW]** Detector implementation |
| `crates/analyzer/src/detectors/sir/mod.rs` | Add `pub mod denial_of_service` + re-export |
| `crates/analyzer/src/detectors/mod.rs` | Add re-export |
| `crates/analyzer/src/detectors/base/id.rs` | Add `DenialOfService` variant |
| `crates/analyzer/src/detectors/base/registry.rs` | Register the new detector |

#### Sample Detection Targets

```solidity
// From auction.sol — require(send) pattern
require(currentFrontrunner.send(currentBid));  // single failure → full revert

// From send_loop.sol — external call in loop
for(uint x; x < refundAddresses.length; x++) {
    require(refundAddresses[x].send(refunds[refundAddresses[x]]));
}

// From dos_simple.sol — unbounded loop with storage writes
for(uint i=0; i<350; i++) {
    listAddresses.push(msg.sender);  // unbounded push to storage array
}
```

---

### 2.4 Front Running / Transaction Order Dependence Detector (`FrontRunningSirDetector`)

**SmartBugs category**: `front_running/` (4 files, ~7 annotated instances)
**SWC**: SWC-114 (Transaction Order Dependence)
**CWE**: CWE-362 (Race Condition)
**Risk**: Medium

#### Detection Strategy

Front-running vulnerabilities arise when the outcome of a transaction depends on state that can be
changed by another transaction between submission and mining. Key patterns:

1. **ERC-20 `approve` race condition** (most common):
   Detect functions named `approve` that directly set an allowance mapping
   (`_allowed[owner][spender] = value`) without first checking/resetting the old value.
   Flag the `approve` function if it doesn't implement an increaseAllowance/decreaseAllowance
   pattern or doesn't require the current allowance be zero.

2. **State-dependent ETH transfer**:
   Public functions that read a storage variable (e.g., `reward`) and use its value in a
   `transfer`/`send`/`call`, where another public function can modify that same variable.
   Detection: For each public function containing an ETH transfer, check if the transferred
   amount comes from a storage variable. If another public function writes to that variable,
   flag both functions.

3. **Commit-reveal absence**:
   Functions that accept a "solution" or "answer" parameter and compare it against a stored
   value, sending ETH on match — without any commit-reveal scheme. This is harder to detect
   with pure structural analysis; implement as a heuristic by matching function signatures like
   `solve(bytes32)`, `submitSolution(…)`, etc.

#### Files to Modify/Create

| File | Action |
|------|--------|
| `crates/analyzer/src/detectors/sir/front_running.rs` | **[NEW]** Detector implementation |
| `crates/analyzer/src/detectors/sir/mod.rs` | Add `pub mod front_running` + re-export |
| `crates/analyzer/src/detectors/mod.rs` | Add re-export |
| `crates/analyzer/src/detectors/base/id.rs` | Add `FrontRunning` variant |
| `crates/analyzer/src/detectors/base/registry.rs` | Register the new detector |

#### Sample Detection Targets

```solidity
// From ERC20.sol — approve race condition
function approve(address spender, uint256 value) public returns (bool) {
    _allowed[msg.sender][spender] = value;  // direct set without checking old value
}

// From eth_tx_order_dependence_minimal.sol — state-dependent transfer
function claimReward(uint256 submission) {
    msg.sender.transfer(reward);  // reward can be changed by setReward()
}
```

---

### 2.5 Short Address / Input Validation Detector (`ShortAddressSirDetector`)

**SmartBugs category**: `short_addresses/` (1 file, 1 annotated instance)
**SWC**: SWC-118 (Incorrect Constructor Name) — related; SWC-130 (Short Address Attack)
**CWE**: CWE-20 (Improper Input Validation)
**Risk**: Low

#### Detection Strategy

The short address attack is fundamentally an **off-chain / ABI-encoding vulnerability**. It
exploits the EVM's tolerance for short calldata. Static analysis on the contract itself has limited
effectiveness, but we can flag the common vulnerable pattern:

1. **ERC-20 transfer without msg.data length check**:
   Detect public `transfer(address, uint256)` or `transferFrom(address, address, uint256)`
   functions that do NOT contain a `require(msg.data.length >= N)` check (where N is the
   expected calldata length: 4 + 32*params).

2. **Heuristic**: Walk function bodies of ERC-20-like `transfer`/`transferFrom` functions. If the
   function body contains no reference to `msg.data` or `msg.data.length`, flag it.

#### Scope Note

This is a low-priority detector with only 1 test case in SmartBugs. It targets a vulnerability
that is largely mitigated by modern Solidity compilers (≥0.5.0) and off-chain tooling. Consider
implementing it last, or as an informational-only check.

#### Files to Modify/Create

| File | Action |
|------|--------|
| `crates/analyzer/src/detectors/sir/short_address.rs` | **[NEW]** Detector implementation |
| `crates/analyzer/src/detectors/sir/mod.rs` | Add `pub mod short_address` + re-export |
| `crates/analyzer/src/detectors/mod.rs` | Add re-export |
| `crates/analyzer/src/detectors/base/id.rs` | Add `ShortAddress` variant |
| `crates/analyzer/src/detectors/base/registry.rs` | Register the new detector |

---

## 3. Shared Infrastructure Changes

### 3.1 `DetectorId` Enum Extensions

Add 5 new variants to `crates/analyzer/src/detectors/base/id.rs`:

```rust
pub enum DetectorId {
    // ... existing 16 variants ...
    ArithmeticOverflow,  // NEW
    BadRandomness,       // NEW
    DenialOfService,     // NEW
    FrontRunning,        // NEW
    ShortAddress,        // NEW
}
```

With corresponding `as_str()` mappings:
- `ArithmeticOverflow` → `"arithmetic-overflow"`
- `BadRandomness` → `"bad-randomness"`
- `DenialOfService` → `"denial-of-service"`
- `FrontRunning` → `"front-running"`
- `ShortAddress` → `"short-address"`

### 3.2 Detector Registry

Update `register_all_detectors()` in `crates/analyzer/src/detectors/base/registry.rs` to
include all 5 new detectors.

### 3.3 SIR Visitor Utilities

Some detectors (bad_randomness, denial_of_service) will benefit from shared visitor helpers.
Consider adding to `crates/scirs/src/sir/utils/visit.rs` or as local helpers:
- `contains_evm_expr(stmt, predicate)` — check if any sub-expression matches
- `contains_external_call_in_loop(stmts)` — walk nested loops for external calls

---

## 4. Implementation Order (by priority)

| Priority | Detector | SmartBugs Files | Rationale |
|----------|----------|-----------------|-----------|
| **P0** | `ArithmeticOverflowSirDetector` | 16 | Most files, well-defined SIR pattern via `OverflowSemantics` |
| **P1** | `BadRandomnessSirDetector` | 8 | High impact, clear EVM expr patterns |
| **P2** | `DenialOfServiceSirDetector` | 6 | High impact, multiple sub-patterns |
| **P3** | `FrontRunningSirDetector` | 4 | Medium complexity, heuristic-heavy |
| **P4** | `ShortAddressSirDetector` | 1 | Low priority, mostly mitigated by modern compilers |

---

## 5. Verification Plan

### Per-Detector Verification

For each new detector:

1. **Unit test**: Add `#[cfg(test)] mod tests` with a basic smoke test (detector ID, risk level,
   SWC/CWE IDs).

2. **Integration test with SmartBugs samples**: Run the analyzer against the corresponding
   SmartBugs-curated subdirectory and verify that bugs are reported for annotated lines:

   ```bash
   # Example: test arithmetic detector against SmartBugs arithmetic/ samples
   cargo run -- analyze datasets/solidity/smartbugs-curated/arithmetic/integer_overflow_add.sol
   ```

3. **Benchmark tool**: Use the existing `benchmark` tool to run batch analysis:

   ```bash
   cargo run --bin benchmark -- datasets/solidity/smartbugs-curated/arithmetic/
   ```

### Full Regression

After all detectors are implemented:

```bash
cargo run --bin benchmark -- datasets/solidity/smartbugs-curated/
```

Verify that all 10 categories now produce at least one detection per annotated file.

### Build Verification

```bash
cargo check 2>&1
cargo test -p analyzer
```

---

## 6. Post-Implementation: Coverage Target

After implementing all 5 detectors, expected coverage:

| SmartBugs Category | Detector(s) | Status |
|---|---|---|
| reentrancy | Reentrancy + CeiViolation | ✅ |
| access_control | MissingAccessControl + Delegatecall + TxOrigin + Visibility + CentralizationRisk | ✅ |
| unchecked_low_level_calls | UncheckedCall + LowLevelCall | ✅ |
| time_manipulation | TimestampDependence | ✅ |
| other | Uninitialized | ✅ |
| **arithmetic** | **ArithmeticOverflow** | ✅ |
| **bad_randomness** | **BadRandomness** | ✅ |
| **denial_of_service** | **DenialOfService** | ✅ |
| **front_running** | **FrontRunning** | ✅ |
| **short_addresses** | **ShortAddress** | ✅ |

**Result**: Full 10/10 SmartBugs-curated category coverage with 21 total detectors.

---

## 7. Task Checklist

### Phase 1: Shared Infrastructure

- [x] Add 5 new variants to `DetectorId` enum in `crates/analyzer/src/detectors/base/id.rs`
  - [x] `ArithmeticOverflow` → `"arithmetic-overflow"`
  - [x] `BadRandomness` → `"bad-randomness"`
  - [x] `DenialOfService` → `"denial-of-service"`
  - [x] `FrontRunning` → `"front-running"`
  - [x] `ShortAddress` → `"short-address"`
- [x] Add `as_str()` match arms for all 5 new variants
- [x] Verify `cargo check` passes after `DetectorId` changes

### Phase 2: Arithmetic Overflow Detector (P0)

- [x] Create `crates/analyzer/src/detectors/sir/arithmetic_overflow.rs`
  - [x] Implement `ArithmeticOverflowSirDetector` struct
  - [x] Implement `Pass` trait
  - [x] Implement `BugDetectionPass` trait
  - [x] Walk `BinOpExpr` nodes for `Add`/`Sub`/`Mul`/`Pow` with `OverflowSemantics::Wrapping`
  - [x] Walk `AugAssignStmt` nodes (`+=`, `-=`, `*=`) with wrapping semantics
  - [x] Add false-positive reduction (skip SafeMath, skip literal-only ops)
  - [x] Set `BugCategory::Arithmetic`, SWC-101, CWE-190/191
  - [x] Add remediation advice
  - [x] Add unit test
- [x] Register in `crates/analyzer/src/detectors/sir/mod.rs`
- [x] Re-export in `crates/analyzer/src/detectors/mod.rs`
- [x] Register in `crates/analyzer/src/detectors/base/registry.rs`
- [x] Verify `cargo check` passes
- [ ] Test against `datasets/solidity/smartbugs-curated/arithmetic/` samples

### Phase 3: Bad Randomness Detector (P1)

- [x] Create `crates/analyzer/src/detectors/sir/bad_randomness.rs`
  - [x] Implement `BadRandomnessSirDetector` struct
  - [x] Implement `Pass` trait
  - [x] Implement `BugDetectionPass` trait
  - [x] Detect `EvmExpr::Blockhash`, `Timestamp`, `BlockNumber`, `BlockDifficulty`, `BlockCoinbase`, `BlockGaslimit` as randomness sources
  - [x] Check usage context: arguments to hash functions, modulo operands
  - [x] Add false-positive reduction (skip pure time-gating patterns)
  - [x] Set `BugCategory::BadRandomness`, SWC-120, CWE-330
  - [x] Add remediation advice
  - [x] Add unit test
- [x] Register in `crates/analyzer/src/detectors/sir/mod.rs`
- [x] Re-export in `crates/analyzer/src/detectors/mod.rs`
- [x] Register in `crates/analyzer/src/detectors/base/registry.rs`
- [x] Verify `cargo check` passes
- [ ] Test against `datasets/solidity/smartbugs-curated/bad_randomness/` samples

### Phase 4: Denial of Service Detector (P2)

- [x] Create `crates/analyzer/src/detectors/sir/denial_of_service.rs`
  - [x] Implement `DenialOfServiceSirDetector` struct
  - [x] Implement `Pass` trait
  - [x] Implement `BugDetectionPass` trait
  - [x] Sub-pattern 1: Detect external calls (`is_evm_external_call()`) inside `ForStmt`/`WhileStmt` bodies
  - [x] Sub-pattern 2: Detect `require(addr.send(…))` pattern
  - [x] Sub-pattern 3: Detect unbounded loops over dynamic storage array `.length`
  - [x] Set `BugCategory::DenialOfService`, SWC-113/128, CWE-400
  - [x] Add remediation advice
  - [x] Add unit test
- [x] Register in `crates/analyzer/src/detectors/sir/mod.rs`
- [x] Re-export in `crates/analyzer/src/detectors/mod.rs`
- [x] Register in `crates/analyzer/src/detectors/base/registry.rs`
- [x] Verify `cargo check` passes
- [ ] Test against `datasets/solidity/smartbugs-curated/denial_of_service/` samples

### Phase 5: Front Running Detector (P3)

- [x] Create `crates/analyzer/src/detectors/sir/front_running.rs`
  - [x] Implement `FrontRunningSirDetector` struct
  - [x] Implement `Pass` trait
  - [x] Implement `BugDetectionPass` trait
  - [x] Sub-pattern 1: Detect ERC-20 `approve` functions that directly set allowance without checking old value
  - [x] Sub-pattern 2: Detect state-dependent ETH transfers where another public function can modify the state variable
  - [x] Set `BugCategory::FrontRunning`, SWC-114, CWE-362
  - [x] Add remediation advice
  - [x] Add unit test
- [x] Register in `crates/analyzer/src/detectors/sir/mod.rs`
- [x] Re-export in `crates/analyzer/src/detectors/mod.rs`
- [x] Register in `crates/analyzer/src/detectors/base/registry.rs`
- [x] Verify `cargo check` passes
- [ ] Test against `datasets/solidity/smartbugs-curated/front_running/` samples

### Phase 6: Short Address Detector (P4)

- [x] Create `crates/analyzer/src/detectors/sir/short_address.rs`
  - [x] Implement `ShortAddressSirDetector` struct
  - [x] Implement `Pass` trait
  - [x] Implement `BugDetectionPass` trait
  - [x] Detect ERC-20 `transfer`/`transferFrom` functions without `msg.data.length` check
  - [x] Set `BugCategory::ShortAddresses`, SWC-130, CWE-20
  - [x] Add remediation advice (informational severity)
  - [x] Add unit test
- [x] Register in `crates/analyzer/src/detectors/sir/mod.rs`
- [x] Re-export in `crates/analyzer/src/detectors/mod.rs`
- [x] Register in `crates/analyzer/src/detectors/base/registry.rs`
- [x] Verify `cargo check` passes
- [ ] Test against `datasets/solidity/smartbugs-curated/short_addresses/` samples

### Phase 7: Final Verification

- [x] Run `cargo check` on full workspace
- [x] Run `cargo test -p analyzer` — all tests pass
- [ ] Run full SmartBugs benchmark: `cargo run --bin benchmark -- datasets/solidity/smartbugs-curated/`
- [ ] Verify all 10 SmartBugs categories produce at least one detection per annotated file
