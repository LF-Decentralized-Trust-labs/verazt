# Migration Plan: Reorganizing Bug Detectors into DFA and GREP

**Date**: 2026-02-09
**Status**: Planning Phase
**Author**: Claude Code

## Executive Summary

This document outlines the migration plan for reorganizing SmartHunt's 18 existing AST-based bug detectors into two specialized frameworks:

1. **DFA Framework** (`dfa/detectors/`) - For IR-based detectors requiring data flow analysis
2. **GREP Framework** (`grep/detectors/`) - For AST-based detectors using declarative pattern matching

## Current State

### Existing Detectors Location
All 18 detectors currently reside in:
```
smarthunt/src/detection/detectors/ast/
```

### Detector Inventory

| # | Detector | File | Risk Level | Status |
|---|----------|------|------------|--------|
| 1 | Tx.Origin Authentication | `tx_origin.rs` | High | Implemented |
| 2 | Floating Pragma | `floating_pragma.rs` | Low | Implemented |
| 3 | Visibility Issues | `visibility.rs` | Medium | Implemented |
| 4 | Deprecated Functions | `deprecated.rs` | Low | Implemented |
| 5 | Low-Level Calls | `low_level_call.rs` | Medium | Implemented |
| 6 | Unchecked Call Returns | `unchecked_call.rs` | High | Implemented |
| 7 | Variable Shadowing | `shadowing.rs` | Medium | Implemented |
| 8 | Timestamp Dependence | `timestamp_dependence.rs` | Medium | Implemented |
| 9 | Unsafe Delegatecall | `delegatecall.rs` | High | Implemented |
| 10 | Uninitialized Storage | `uninitialized.rs` | High | Implemented |
| 11 | Centralization Risk | `centralization_risk.rs` | Low | Implemented |
| 12 | CEI Violation | `cei_violation.rs` | High | Implemented |
| 13 | Reentrancy | `reentrancy.rs` | Critical | Stub only |
| 14 | Missing Access Control | `missing_access_control.rs` | Critical | Implemented |
| 15 | Dead Code | `dead_code.rs` | Low | Implemented |
| 16 | Constant State Variables | `constant_state_var.rs` | Low | Implemented |

### Framework Status

#### DFA Framework (`smarthunt/src/dfa/`)
**Status**: ✅ Fully implemented, ready for detectors

**Components**:
- `cfg.rs` - Control Flow Graph with basic blocks and terminators
- `lattice.rs` - Abstract domain framework (PowerSet, Flat, Map, Product lattices)
- `solver.rs` - Worklist-based data flow solver (forward/backward)
- `var.rs` - Variable tracking
- `analyses/` - Built-in analyses:
  - `reaching_defs.rs` - Reaching definitions
  - `liveness.rs` - Live variable analysis
  - `def_use.rs` - Def-use chains
  - `taint.rs` - Taint analysis
  - `state_mutation.rs` - State mutation tracking

**Target Directory**: `smarthunt/src/dfa/detectors/` (currently empty)

#### GREP Framework (`smarthunt/src/grep/`)
**Status**: ✅ Fully implemented, ready for detectors

**Components**:
- `core.rs` - Pattern trait, Match type, captures
- `primitives.rs` - Basic patterns (Ident, MemberAccess, Call, Any)
- `composite.rs` - Pattern combinators (And, Or, Not, Contains, Where)
- `builder.rs` - Fluent DSL for pattern construction
- `matcher.rs` - Multi-pattern single-pass matcher

**Target Directory**: `smarthunt/src/grep/detectors/` (currently empty)

## Classification Criteria

### DFA Framework (IR-based detectors)

Use DFA when the detector requires:
- **Data flow analysis** (tracking values across program paths)
- **Control flow analysis** (understanding execution paths)
- **State mutation tracking** (monitoring storage changes)
- **Def-use chains** (variable definitions and uses)
- **Taint analysis** (tracking data from sources to sinks)
- **Inter-procedural analysis** (analyzing across function calls)

**Characteristics**:
- Complex analysis requiring program semantics
- Need to understand execution order
- Track values through assignments and calls
- Reason about all possible execution paths

### GREP Framework (AST pattern-based detectors)

Use GREP when the detector requires:
- **Syntactic pattern matching** (looking for code patterns)
- **Declarative rule specification** (expressing "what" not "how")
- **Simple structural checks** (AST node properties)
- **Local scope analysis** (within single function/contract)

**Characteristics**:
- Can be expressed as patterns over AST structure
- No need for control/data flow reasoning
- Primarily syntactic checks
- Can benefit from declarative, composable patterns

## Migration Classification

### Phase 1: Move to DFA Framework (IR-based)

These detectors require data flow or control flow analysis:

#### 1.1 Reentrancy Detector
**File**: `reentrancy.rs`
**Target**: `dfa/detectors/reentrancy.rs`
**Risk**: Critical
**Rationale**:
- Requires inter-procedural data flow analysis
- Needs state mutation tracking across external calls
- Must analyze control flow paths for reentrancy patterns
- Uses taint analysis to track external call flows

**DFA Components Needed**:
- CFG for control flow paths
- State mutation analysis
- Taint analysis (track external calls)
- Call graph for inter-procedural analysis

**Implementation Approach**:
```rust
// Use DFA to detect:
// 1. External calls (taint sources)
// 2. State mutations after external calls
// 3. Reentrant paths back to the same function
```

---

#### 1.2 CEI Violation Detector
**File**: `cei_violation.rs`
**Target**: `dfa/detectors/cei_violation.rs`
**Risk**: High
**Rationale**:
- Requires control flow analysis to verify CEI pattern
- Needs state mutation tracking
- Must analyze execution order (checks → effects → interactions)

**DFA Components Needed**:
- CFG for execution order
- State mutation analysis
- Def-use chains for variable tracking

**Implementation Approach**:
```rust
// Use DFA to verify ordering:
// 1. Checks (conditions, requires)
// 2. Effects (state changes)
// 3. Interactions (external calls)
// Flag violations where effects come after interactions
```

---

#### 1.3 Uninitialized Storage Detector
**File**: `uninitialized.rs`
**Target**: `dfa/detectors/uninitialized.rs`
**Risk**: High
**Rationale**:
- Requires reaching definitions analysis
- Needs to track initialization across control flow paths
- Must handle all execution paths to function exit

**DFA Components Needed**:
- Reaching definitions analysis
- CFG for path analysis
- Def-use chains

**Implementation Approach**:
```rust
// Use DFA to check:
// 1. Storage variable declarations
// 2. All paths to first use
// 3. Ensure initialization on all paths
```

---

#### 1.4 Unchecked Call Return Detector
**File**: `unchecked_call.rs`
**Target**: `dfa/detectors/unchecked_call.rs`
**Risk**: High
**Rationale**:
- Requires def-use analysis for return values
- Needs to track if return value is checked on any path
- Must analyze control flow after call

**DFA Components Needed**:
- Def-use chains
- CFG for post-call analysis
- Liveness analysis (is return value used?)

**Implementation Approach**:
```rust
// Use DFA to verify:
// 1. Identify low-level calls (call, delegatecall)
// 2. Check if return value is used in any control flow
// 3. Flag unchecked returns
```

---

#### 1.5 Dead Code Detector
**File**: `dead_code.rs`
**Target**: `dfa/detectors/dead_code.rs`
**Risk**: Low
**Rationale**:
- Requires liveness analysis
- Needs CFG to identify unreachable code
- Must analyze all control flow paths

**DFA Components Needed**:
- Liveness analysis
- CFG for reachability analysis
- Backward data flow solver

**Implementation Approach**:
```rust
// Use DFA to find:
// 1. Unreachable basic blocks (after return/revert)
// 2. Dead assignments (variables never read)
// 3. Functions never called
```

---

### Phase 2: Move to GREP Framework (Pattern-based)

These detectors can be expressed as declarative AST patterns:

#### 2.1 Tx.Origin Authentication
**File**: `tx_origin.rs`
**Target**: `grep/detectors/tx_origin.rs`
**Risk**: High
**Pattern**:
```rust
// Match: tx.origin used in require/if conditions
// Pattern: MemberAccess("tx", "origin") in condition context
Pattern::member_access("tx", "origin")
    .in_context(|node| is_auth_check(node))
```

---

#### 2.2 Floating Pragma
**File**: `floating_pragma.rs`
**Target**: `grep/detectors/floating_pragma.rs`
**Risk**: Low
**Pattern**:
```rust
// Match: pragma solidity without exact version
// Pattern: Pragma with unlocked version (^, >, >=, <)
Pattern::pragma()
    .where(|p| has_unlocked_version(p))
```

---

#### 2.3 Visibility Issues
**File**: `visibility.rs`
**Target**: `grep/detectors/visibility.rs`
**Risk**: Medium
**Pattern**:
```rust
// Match: Functions without explicit visibility
// Pattern: FunctionDef with missing visibility attribute
Pattern::function()
    .where(|f| f.visibility.is_none() || is_public_default(f))
```

---

#### 2.4 Deprecated Functions
**File**: `deprecated.rs`
**Target**: `grep/detectors/deprecated.rs`
**Risk**: Low
**Pattern**:
```rust
// Match: Calls to deprecated functions
// Pattern: Call to known deprecated functions
Pattern::call()
    .where(|c| DEPRECATED_FUNCTIONS.contains(c.name))
```

---

#### 2.5 Low-Level Calls
**File**: `low_level_call.rs`
**Target**: `grep/detectors/low_level_call.rs`
**Risk**: Medium
**Pattern**:
```rust
// Match: address.call() / address.delegatecall()
// Pattern: MemberAccess with call/delegatecall
Pattern::member_access(Pattern::any(), "call")
    .or(Pattern::member_access(Pattern::any(), "delegatecall"))
```

---

#### 2.6 Variable Shadowing
**File**: `shadowing.rs`
**Target**: `grep/detectors/shadowing.rs`
**Risk**: Medium
**Pattern**:
```rust
// Match: Variables shadowing inherited/parent scope
// Pattern: Variable declaration matching name in outer scope
Pattern::var_declaration()
    .where(|v| scope_contains_name(v.name))
```

---

#### 2.7 Timestamp Dependence
**File**: `timestamp_dependence.rs`
**Target**: `grep/detectors/timestamp_dependence.rs`
**Risk**: Medium
**Pattern**:
```rust
// Match: block.timestamp or now in conditions
// Pattern: MemberAccess("block", "timestamp") or Ident("now")
Pattern::member_access("block", "timestamp")
    .or(Pattern::ident("now"))
    .in_context(|n| is_critical_logic(n))
```

---

#### 2.8 Unsafe Delegatecall
**File**: `delegatecall.rs`
**Target**: `grep/detectors/delegatecall.rs`
**Risk**: High
**Pattern**:
```rust
// Match: delegatecall to user-controlled address
// Pattern: MemberAccess with delegatecall
Pattern::member_access(Pattern::any(), "delegatecall")
    .where(|c| target_not_constant(c))
```

---

#### 2.9 Centralization Risk
**File**: `centralization_risk.rs`
**Target**: `grep/detectors/centralization_risk.rs`
**Risk**: Low
**Pattern**:
```rust
// Match: Functions with onlyOwner or privileged modifiers
// Pattern: Function with specific modifiers
Pattern::function()
    .with_modifier(Pattern::ident("onlyOwner"))
    .or_modifier_matching(|m| is_privileged_modifier(m))
```

---

#### 2.10 Missing Access Control
**File**: `missing_access_control.rs`
**Target**: `grep/detectors/missing_access_control.rs`
**Risk**: Critical
**Pattern**:
```rust
// Match: State-changing functions without access control modifiers
// Pattern: Function that mutates state without modifier
Pattern::function()
    .where(|f| mutates_state(f) && !has_access_modifier(f))
```

---

#### 2.11 Constant State Variables
**File**: `constant_state_var.rs`
**Target**: `grep/detectors/constant_state_var.rs`
**Risk**: Low
**Pattern**:
```rust
// Match: State variables that never change
// Pattern: State var without assignments
Pattern::state_variable()
    .where(|v| !v.is_constant && has_constant_initializer(v))
    .and(Pattern::not_assigned_to(v))
```

---

### Phase 3: Hybrid Detectors (Future)

Some detectors may benefit from both approaches:

#### Future Consideration: Enhanced Reentrancy
- **GREP** for quick pattern-based detection (low-hanging fruit)
- **DFA** for deep inter-procedural analysis (complex cases)

## Migration Steps

### Prerequisites

1. ✅ DFA framework implemented (`dfa/`)
2. ✅ GREP framework implemented (`grep/`)
3. ✅ Empty detector directories created
4. ✅ Analysis framework supports both representations
5. ⬜ IR compilation pipeline ready (if not already)

### Step-by-Step Migration Process

#### Phase 1: DFA Detectors (IR-based)

**Step 1.1: Implement Reentrancy Detector** (Critical priority)
- [ ] Create `dfa/detectors/reentrancy.rs`
- [ ] Implement using DFA framework:
  - CFG construction
  - State mutation analysis
  - Taint analysis for external calls
  - Inter-procedural analysis
- [ ] Define transfer functions for data flow
- [ ] Implement `Pass` trait with `PassRepresentation::IR`
- [ ] Implement `BugDetectionPass` trait
- [ ] Add comprehensive tests
- [ ] Update `dfa/detectors/mod.rs` to export

**Step 1.2: Implement CEI Violation Detector**
- [ ] Create `dfa/detectors/cei_violation.rs`
- [ ] Use CFG to analyze execution order
- [ ] Track state mutations and external calls
- [ ] Detect violations of Checks-Effects-Interactions pattern
- [ ] Implement pass traits
- [ ] Add tests
- [ ] Export from module

**Step 1.3: Implement Uninitialized Storage Detector**
- [ ] Create `dfa/detectors/uninitialized.rs`
- [ ] Use reaching definitions analysis
- [ ] Track initialization across all paths
- [ ] Implement pass traits
- [ ] Add tests
- [ ] Export from module

**Step 1.4: Implement Unchecked Call Return Detector**
- [ ] Create `dfa/detectors/unchecked_call.rs`
- [ ] Use def-use analysis for return values
- [ ] Track usage in control flow
- [ ] Implement pass traits
- [ ] Add tests
- [ ] Export from module

**Step 1.5: Implement Dead Code Detector**
- [ ] Create `dfa/detectors/dead_code.rs`
- [ ] Use liveness analysis
- [ ] Identify unreachable blocks
- [ ] Implement pass traits
- [ ] Add tests
- [ ] Export from module

**Step 1.6: Clean up old AST versions**
- [ ] Remove old AST-based implementations from `detection/detectors/ast/`:
  - `reentrancy.rs`
  - `cei_violation.rs`
  - `uninitialized.rs`
  - `unchecked_call.rs`
  - `dead_code.rs`
- [ ] Update `detection/detectors/ast/mod.rs`
- [ ] Update detector registry

---

#### Phase 2: GREP Detectors (Pattern-based)

**Step 2.1: Implement High-Risk Pattern Detectors First**

Priority order (by risk level):

1. **Missing Access Control** (Critical)
   - [ ] Create `grep/detectors/missing_access_control.rs`
   - [ ] Define patterns for state-changing functions
   - [ ] Define patterns for access control modifiers
   - [ ] Implement composite pattern (function without modifier)
   - [ ] Implement pass traits with `PassRepresentation::Ast`
   - [ ] Add tests
   - [ ] Export from module

2. **Tx.Origin Authentication** (High)
   - [ ] Create `grep/detectors/tx_origin.rs`
   - [ ] Pattern: `tx.origin` in authentication contexts
   - [ ] Implement pass traits
   - [ ] Add tests
   - [ ] Export from module

3. **Unsafe Delegatecall** (High)
   - [ ] Create `grep/detectors/delegatecall.rs`
   - [ ] Pattern: delegatecall with non-constant target
   - [ ] Implement pass traits
   - [ ] Add tests
   - [ ] Export from module

**Step 2.2: Implement Medium-Risk Pattern Detectors**

4. **Low-Level Calls** (Medium)
   - [ ] Create `grep/detectors/low_level_call.rs`
   - [ ] Pattern: call/delegatecall usage
   - [ ] Implement pass traits
   - [ ] Add tests

5. **Timestamp Dependence** (Medium)
   - [ ] Create `grep/detectors/timestamp_dependence.rs`
   - [ ] Pattern: block.timestamp/now in critical logic
   - [ ] Implement pass traits
   - [ ] Add tests

6. **Variable Shadowing** (Medium)
   - [ ] Create `grep/detectors/shadowing.rs`
   - [ ] Pattern: variable names matching outer scope
   - [ ] Implement pass traits
   - [ ] Add tests

7. **Visibility Issues** (Medium)
   - [ ] Create `grep/detectors/visibility.rs`
   - [ ] Pattern: functions with missing/default visibility
   - [ ] Implement pass traits
   - [ ] Add tests

**Step 2.3: Implement Low-Risk Pattern Detectors**

8. **Centralization Risk** (Low)
   - [ ] Create `grep/detectors/centralization_risk.rs`
   - [ ] Pattern: privileged modifiers
   - [ ] Implement pass traits
   - [ ] Add tests

9. **Deprecated Functions** (Low)
   - [ ] Create `grep/detectors/deprecated.rs`
   - [ ] Pattern: calls to deprecated functions
   - [ ] Implement pass traits
   - [ ] Add tests

10. **Floating Pragma** (Low)
    - [ ] Create `grep/detectors/floating_pragma.rs`
    - [ ] Pattern: unlocked pragma versions
    - [ ] Implement pass traits
    - [ ] Add tests

11. **Constant State Variables** (Low)
    - [ ] Create `grep/detectors/constant_state_var.rs`
    - [ ] Pattern: unmodified state variables
    - [ ] Implement pass traits
    - [ ] Add tests

12. **Dead Code** (Low) - Note: May also have DFA version
    - [ ] Create `grep/detectors/dead_code.rs` (simple syntactic version)
    - [ ] Pattern: unreachable statements after return
    - [ ] Implement pass traits
    - [ ] Add tests

**Step 2.4: Clean up old AST versions**
- [ ] Remove migrated detectors from `detection/detectors/ast/`
- [ ] Update `detection/detectors/ast/mod.rs`
- [ ] Update detector registry

---

#### Phase 3: Integration and Testing

**Step 3.1: Update Module Structure**
- [ ] Update `detection/detectors/mod.rs`:
  ```rust
  pub mod ast;      // Keep for any remaining AST detectors
  pub mod dfa;      // Enable DFA detectors module
  pub mod grep;     // Enable GREP detectors module
  // pub mod hybrid; // Future: for hybrid detectors
  ```

**Step 3.2: Update Detector Registry**
- [ ] Register all DFA detectors in `detection/registry.rs`
- [ ] Register all GREP detectors in `detection/registry.rs`
- [ ] Ensure correct pass dependencies
- [ ] Verify PassRepresentation is set correctly

**Step 3.3: Update Analysis Pass Scheduler**
- [ ] Ensure scheduler handles IR-based passes
- [ ] Verify dependency resolution for DFA passes
- [ ] Test pass ordering (AST → IR → Detection)

**Step 3.4: Comprehensive Testing**
- [ ] Unit tests for each migrated detector
- [ ] Integration tests for DFA framework
- [ ] Integration tests for GREP framework
- [ ] Regression tests against old implementations
- [ ] Performance benchmarks (compare old vs new)
- [ ] Test suite for edge cases

**Step 3.5: Documentation**
- [ ] Update README with new architecture
- [ ] Document DFA detector implementation guide
- [ ] Document GREP detector implementation guide
- [ ] Add migration examples
- [ ] Update API documentation

---

#### Phase 4: Cleanup and Optimization

**Step 4.1: Remove Old Code**
- [ ] Delete `detection/detectors/ast/` directory (if empty)
- [ ] Remove deprecated detector implementations
- [ ] Clean up unused imports
- [ ] Update exports

**Step 4.2: Performance Optimization**
- [ ] Profile DFA solver performance
- [ ] Optimize pattern matching in GREP
- [ ] Implement caching where beneficial
- [ ] Parallelize independent detectors if possible

**Step 4.3: Code Quality**
- [ ] Run clippy and fix warnings
- [ ] Format code with rustfmt
- [ ] Review and refactor common patterns
- [ ] Ensure consistent error handling

---

## Implementation Guidelines

### For DFA Detectors

#### Template Structure
```rust
use crate::dfa::{cfg::ControlFlowGraph, solver::DataFlowSolver, lattice::*};
use crate::analysis::{Pass, PassId, PassLevel, PassRepresentation, AnalysisContext};
use crate::detection::{BugDetectionPass, DetectorResult, Bug};

pub struct <DetectorName>Detector;

impl Pass for <DetectorName>Detector {
    fn id(&self) -> PassId {
        PassId::<DetectorName>
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::IR  // Important: IR representation
    }

    fn level(&self) -> PassLevel {
        PassLevel::Function  // or Contract/Program as needed
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![
            PassId::CFG,           // Usually need CFG
            PassId::ReachingDefs,  // Add other DFA passes as needed
        ]
    }

    // ... other Pass methods
}

impl BugDetectionPass for <DetectorName>Detector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        // Get IR artifacts from context
        let cfg = context.get_artifact::<ControlFlowGraph>()?;

        // Run data flow analysis
        let result = DataFlowSolver::solve(/* ... */)?;

        // Analyze results and create bugs
        for violation in find_violations(&result) {
            bugs.push(create_bug(
                self,
                format!("Found issue: {}", violation),
                violation.location,
            ));
        }

        Ok(bugs)
    }

    // ... other BugDetectionPass methods
}
```

#### Best Practices
- Use appropriate lattice types (PowerSet, Flat, Map, Product)
- Implement efficient transfer functions
- Handle CFG terminators properly (Jump, Branch, Return, Revert)
- Consider forward vs backward analysis
- Cache results when possible
- Handle inter-procedural analysis carefully

---

### For GREP Detectors

#### Template Structure
```rust
use crate::grep::{Pattern, PatternMatcher, Match};
use crate::analysis::{Pass, PassId, PassLevel, PassRepresentation, AnalysisContext};
use crate::detection::{BugDetectionPass, DetectorResult, Bug};

pub struct <DetectorName>Detector {
    pattern: Box<dyn Pattern>,
}

impl <DetectorName>Detector {
    pub fn new() -> Self {
        Self {
            // Define pattern using builder or combinators
            pattern: Pattern::member_access("tx", "origin")
                .in_context(|node| is_auth_check(node))
                .build(),
        }
    }
}

impl Pass for <DetectorName>Detector {
    fn id(&self) -> PassId {
        PassId::<DetectorName>
    }

    fn representation(&self) -> PassRepresentation {
        PassRepresentation::Ast  // Important: AST representation
    }

    fn level(&self) -> PassLevel {
        PassLevel::Contract  // or Function/Expression as needed
    }

    fn dependencies(&self) -> Vec<PassId> {
        vec![
            PassId::SymbolTable,  // Common dependency
            // Add others as needed
        ]
    }

    // ... other Pass methods
}

impl BugDetectionPass for <DetectorName>Detector {
    fn detect(&self, context: &AnalysisContext) -> DetectorResult<Vec<Bug>> {
        let mut bugs = Vec::new();

        // Get AST from context
        let ast = context.get_ast()?;

        // Use pattern matcher
        let matcher = PatternMatcher::new();
        let matches = matcher.find_all(&self.pattern, ast)?;

        // Create bugs from matches
        for m in matches {
            bugs.push(create_bug(
                self,
                format!("Pattern matched: {}", m.description()),
                m.location(),
            ));
        }

        Ok(bugs)
    }

    // ... other BugDetectionPass methods
}
```

#### Best Practices
- Use declarative patterns when possible
- Compose patterns with And/Or/Not combinators
- Use captures to extract matched nodes
- Leverage single-pass matching for efficiency
- Keep patterns readable and maintainable
- Add helper functions for common pattern checks

---

## Testing Strategy

### Unit Tests
Each detector should have:
- Positive test cases (should detect)
- Negative test cases (should not detect)
- Edge cases
- False positive prevention tests

### Integration Tests
- Test DFA framework with detectors
- Test GREP framework with detectors
- Test detector combinations
- Test pass scheduling and dependencies

### Regression Tests
- Compare results with old implementations
- Ensure no loss of detection capability
- Verify performance improvements

### Performance Tests
- Benchmark DFA solver
- Benchmark pattern matching
- Compare old vs new implementation speed
- Memory usage profiling

---

## Success Criteria

### Functional Requirements
- [ ] All 18 detectors migrated successfully
- [ ] No regression in detection capability
- [ ] All tests passing
- [ ] Performance equal to or better than original

### Code Quality
- [ ] Clean separation between DFA and GREP
- [ ] Consistent implementation patterns
- [ ] Comprehensive documentation
- [ ] No clippy warnings

### Architecture
- [ ] Clear framework boundaries
- [ ] Proper use of DFA for IR analysis
- [ ] Proper use of GREP for pattern matching
- [ ] Extensible for future detectors

---

## Timeline Estimates

**Note**: Actual implementation time will vary based on complexity and testing thoroughness.

### Phase 1: DFA Detectors (5 detectors)
- Reentrancy: Complex (high priority)
- CEI Violation: Complex
- Uninitialized: Medium
- Unchecked Call: Medium
- Dead Code: Simple

### Phase 2: GREP Detectors (11 detectors)
- Most are relatively straightforward pattern matching
- Focus on high-risk detectors first
- Can be parallelized across multiple developers

### Phase 3: Integration & Testing
- Critical phase, cannot be rushed
- Comprehensive testing required

### Phase 4: Cleanup
- Straightforward once migration is complete

---

## Risk Assessment

### High Risk Items
1. **Reentrancy detector complexity** - Most complex detector, critical functionality
   - Mitigation: Start early, allocate sufficient time, comprehensive testing

2. **IR compilation pipeline** - May need fixes/improvements
   - Mitigation: Verify IR is production-ready before migration

3. **Performance regressions** - DFA analysis can be expensive
   - Mitigation: Benchmark early, optimize as needed

### Medium Risk Items
1. **Pattern expressiveness** - Some patterns may be hard to express declaratively
   - Mitigation: Enhance GREP framework if needed

2. **False positives/negatives** - Pattern matching may be too strict/loose
   - Mitigation: Extensive testing, tuning

### Low Risk Items
1. **Simple pattern detectors** - Low-hanging fruit
2. **Module reorganization** - Straightforward file moves

---

## Future Enhancements

### Hybrid Detectors
- Combine DFA and GREP for best of both worlds
- Quick pattern-based screening + deep analysis

### Incremental Analysis
- Cache DFA results
- Only reanalyze changed functions
- Significant performance improvement for large codebases

### Parallel Detection
- Run independent detectors in parallel
- Leverage multi-core processors

### User-Defined Patterns
- Allow users to define custom GREP patterns
- Configuration-based detector customization

---

## Questions to Resolve

1. **IR Compilation Status**: Is the IR compilation pipeline fully ready for production use?
2. **Performance Requirements**: What are acceptable analysis times for large contracts?
3. **Backward Compatibility**: Do we need to maintain old detector implementations during transition?
4. **Testing Coverage**: What is the minimum acceptable test coverage percentage?
5. **Documentation**: What level of documentation is required for each detector?

---

## References

### Related Files
- Analysis framework: `smarthunt/src/analysis/`
- DFA framework: `smarthunt/src/dfa/`
- GREP framework: `smarthunt/src/grep/`
- Current detectors: `smarthunt/src/detection/detectors/ast/`
- Detector registry: `smarthunt/src/detection/registry.rs`

### Related Commits
- `1c138f1` - Fix compilation errors
- `8dae377` - Separate DFA for IR and pattern matching for AST
- `5296ce8` - Implementing DFA framework
- `352b1a1` - Implementing new analysis framework using DFA

---

## Appendix: Quick Reference

### DFA vs GREP Decision Tree
```
Does the detector need to...
├─ Track values across program paths? → DFA
├─ Analyze control flow? → DFA
├─ Perform inter-procedural analysis? → DFA
├─ Track state mutations? → DFA
├─ Match syntactic patterns? → GREP
├─ Check simple structural properties? → GREP
└─ Both? → Consider Hybrid (future)
```

### Detector Classification Summary

**DFA (5 detectors)**:
1. Reentrancy (Critical)
2. CEI Violation (High)
3. Uninitialized Storage (High)
4. Unchecked Call Returns (High)
5. Dead Code (Low)

**GREP (11 detectors)**:
1. Missing Access Control (Critical)
2. Tx.Origin Authentication (High)
3. Unsafe Delegatecall (High)
4. Low-Level Calls (Medium)
5. Timestamp Dependence (Medium)
6. Variable Shadowing (Medium)
7. Visibility Issues (Medium)
8. Centralization Risk (Low)
9. Deprecated Functions (Low)
10. Floating Pragma (Low)
11. Constant State Variables (Low)

---

**End of Migration Plan**
