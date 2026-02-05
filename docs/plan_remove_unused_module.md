# Plan: Remove Unused Legacy Modules

**Date**: 2026-02-05
**Status**: Planning Phase
**Context**: After implementing the LLVM-inspired analysis framework (commit `1731977`), the old analysis and detection system is now obsolete.

---

## Executive Summary

The project currently maintains **TWO PARALLEL SYSTEMS**:
- **OLD**: Legacy smarthunt modules (~8,639 lines)
- **NEW**: LLVM-inspired framework in `solidity/src/analysis/` and `smarthunt/src/detection/` (~5,564 lines)

This plan outlines the removal of ~8,639 lines of legacy code across multiple phases, reducing codebase complexity by ~35%.

---

## Architecture Overview

### New LLVM-Inspired Framework (Keep)

**Core Analysis Framework** (`solidity/src/analysis/`):
- [manager.rs](../solidity/src/analysis/manager.rs) - Analysis orchestrator (354 lines)
- [scheduler.rs](../solidity/src/analysis/scheduler.rs) - Dependency-based scheduling (283 lines)
- [executor.rs](../solidity/src/analysis/executor.rs) - Pass execution engine (289 lines)
- [dependency.rs](../solidity/src/analysis/dependency.rs) - Dependency graph (249 lines)
- [context.rs](../solidity/src/analysis/context.rs) - Type-erased artifact storage (338 lines)
- [pass.rs](../solidity/src/analysis/pass.rs) - Base pass traits
- [pass_id.rs](../solidity/src/analysis/pass_id.rs) - Unified pass identifiers (305 lines)

**Built-in Analysis Passes** (`solidity/src/analysis/passes/ast/`):
- [symbol_table.rs](../solidity/src/analysis/passes/ast/symbol_table.rs) (293 lines)
- [type_index.rs](../solidity/src/analysis/passes/ast/type_index.rs) (239 lines)
- [inheritance_graph.rs](../solidity/src/analysis/passes/ast/inheritance_graph.rs) (251 lines)
- [call_graph.rs](../solidity/src/analysis/passes/ast/call_graph.rs) (473 lines)
- [modifier_analysis.rs](../solidity/src/analysis/passes/ast/modifier_analysis.rs) (316 lines)

**Detection Framework** (`smarthunt/src/detection/`):
- [pass.rs](../smarthunt/src/detection/pass.rs) - BugDetectionPass trait (177 lines)
- [manager.rs](../smarthunt/src/detection/manager.rs) - Detection orchestrator (353 lines)
- [registry.rs](../smarthunt/src/detection/registry.rs) - Detector registry (114 lines)

**New Detectors** (`smarthunt/src/detection/detectors/ast/`):
- [tx_origin.rs](../smarthunt/src/detection/detectors/ast/tx_origin.rs) (319 lines)
- [floating_pragma.rs](../smarthunt/src/detection/detectors/ast/floating_pragma.rs) (105 lines)
- [visibility.rs](../smarthunt/src/detection/detectors/ast/visibility.rs) (79 lines)
- [deprecated.rs](../smarthunt/src/detection/detectors/ast/deprecated.rs) (79 lines)
- [low_level_call.rs](../smarthunt/src/detection/detectors/ast/low_level_call.rs) (79 lines)
- [unchecked_call.rs](../smarthunt/src/detection/detectors/ast/unchecked_call.rs) (79 lines)
- [shadowing.rs](../smarthunt/src/detection/detectors/ast/shadowing.rs) (79 lines)
- [timestamp_dependence.rs](../smarthunt/src/detection/detectors/ast/timestamp_dependence.rs) (79 lines)
- [delegatecall.rs](../smarthunt/src/detection/detectors/ast/delegatecall.rs) (79 lines)

### Legacy System (Remove)

**Old Engine** (`smarthunt/src/engine/`) - **1,339 lines**:
- [context.rs](../smarthunt/src/engine/context.rs) (365 lines) - Hardcoded artifact fields
- [scheduler.rs](../smarthunt/src/engine/scheduler.rs) - Old scheduler
- [parallel.rs](../smarthunt/src/engine/parallel.rs) - Old parallel execution
- [analyzer.rs](../smarthunt/src/engine/analyzer.rs) - Old analyzer
- [config.rs](../smarthunt/src/engine/config.rs) - Old config
- [task_generator_ast.rs](../smarthunt/src/engine/task_generator_ast.rs) - Old task generation
- [task_generator_ir.rs](../smarthunt/src/engine/task_generator_ir.rs) - Old IR tasks

**Old Passes** (`smarthunt/src/passes/`) - **1,694 lines**:
- [pass.rs](../smarthunt/src/passes/pass.rs) (111 lines) - Old pass trait
- [symbol_table.rs](../smarthunt/src/passes/symbol_table.rs) (64 lines)
- [type_index.rs](../smarthunt/src/passes/type_index.rs) (54 lines)
- [cfg.rs](../smarthunt/src/passes/cfg.rs) (467 lines)
- [call_graph.rs](../smarthunt/src/passes/call_graph.rs) (61 lines)
- [data_flow.rs](../smarthunt/src/passes/data_flow.rs) (567 lines)
- [state_mutation.rs](../smarthunt/src/passes/state_mutation.rs) (327 lines)
- [access_control.rs](../smarthunt/src/passes/access_control.rs) (232 lines)

**Old Graph Structures** (`smarthunt/src/graph/`) - **1,377 lines**:
- [symbol_table.rs](../smarthunt/src/graph/symbol_table.rs) (315 lines)
- [type_index.rs](../smarthunt/src/graph/type_index.rs) (166 lines)
- [cfg.rs](../smarthunt/src/graph/cfg.rs) (278 lines)
- [call_graph.rs](../smarthunt/src/graph/call_graph.rs) (439 lines)
- [inheritance.rs](../smarthunt/src/graph/inheritance.rs) (238 lines)

**Old Detectors** (`smarthunt/src/detectors/`) - **4,229 lines**:
- [detector.rs](../smarthunt/src/detectors/detector.rs) (72 lines) - Old trait
- [registry.rs](../smarthunt/src/detectors/registry.rs) (184 lines) - Old registry
- [confidence.rs](../smarthunt/src/detectors/confidence.rs) (21 lines)

**Migrated detectors** (can remove - 9 files):
- [tx_origin.rs](../smarthunt/src/detectors/tx_origin.rs) (150 lines)
- [floating_pragma.rs](../smarthunt/src/detectors/floating_pragma.rs) (121 lines)
- [visibility.rs](../smarthunt/src/detectors/visibility.rs) (150 lines)
- [deprecated.rs](../smarthunt/src/detectors/deprecated.rs) (152 lines)
- [low_level_call.rs](../smarthunt/src/detectors/low_level_call.rs) (151 lines)
- [unchecked_call.rs](../smarthunt/src/detectors/unchecked_call.rs) (154 lines)
- [shadowing.rs](../smarthunt/src/detectors/shadowing.rs) (202 lines)
- [timestamp_dependence.rs](../smarthunt/src/detectors/timestamp_dependence.rs) (152 lines)
- [delegatecall.rs](../smarthunt/src/detectors/delegatecall.rs) (154 lines)

**Not yet migrated** (migrate first, then remove - 8 files):
- [reentrancy.rs](../smarthunt/src/detectors/reentrancy.rs) (356 lines)
- [cei_violation.rs](../smarthunt/src/detectors/cei_violation.rs) (311 lines)
- [dead_code.rs](../smarthunt/src/detectors/dead_code.rs) (265 lines)
- [uninitialized.rs](../smarthunt/src/detectors/uninitialized.rs) (292 lines)
- [missing_access_control.rs](../smarthunt/src/detectors/missing_access_control.rs) (265 lines)
- [centralization_risk.rs](../smarthunt/src/detectors/centralization_risk.rs) (221 lines)
- [constant_state_var.rs](../smarthunt/src/detectors/constant_state_var.rs) (178 lines)

**Old Tasks** (`smarthunt/src/tasks/`) - **~100 lines**:
- [task.rs](../smarthunt/src/tasks/task.rs) (7 lines) - Old task trait
- Task implementations in subdirectories

---

## Key Architectural Improvements

### Old System → New System

| Aspect | Old System | New System |
|--------|-----------|------------|
| **Context** | Hardcoded fields for each artifact | Dynamic type-erased storage (`HashMap<String, Arc<dyn Any>>`) |
| **PassId** | String-based identifiers | Enum-based with comprehensive categorization |
| **Dependencies** | Manual management | Automatic dependency resolution via scheduler |
| **Detectors** | Separate trait hierarchy | Unified as `BugDetectionPass` extending `Pass` |
| **Storage** | Type-specific fields | Flexible artifact storage with type safety |
| **Architecture** | Monolithic analyzer | LLVM-inspired: Manager → Scheduler → Executor |
| **Extensibility** | Limited | Rich trait hierarchy with `PassLevel`, `PassRepresentation` |

---

## Removal Plan

### Phase 1: Remove Core Legacy Infrastructure (Safe - Fully Superseded)

**Estimated impact**: ~4,410 lines removed
**Risk**: Low - fully replaced by new system
**Prerequisites**: None

#### Step 1.1: Remove Old Engine
- **Delete**: `smarthunt/src/engine/` directory (1,339 lines)
- **Replaced by**:
  - `solidity/src/analysis/manager.rs`
  - `solidity/src/analysis/scheduler.rs`
  - `solidity/src/analysis/executor.rs`
  - `solidity/src/analysis/dependency.rs`

#### Step 1.2: Remove Old Passes
- **Delete**: `smarthunt/src/passes/` directory (1,694 lines)
- **Replaced by**: `solidity/src/analysis/passes/ast/*`
- **Note**: New passes are more comprehensive with better dependency management

#### Step 1.3: Remove Old Graph Structures
- **Delete**: `smarthunt/src/graph/` directory (1,377 lines)
- **Replaced by**: Artifacts stored in new analysis pass context
- **Note**: Graphs are now generated and stored as pass artifacts

#### Step 1.4: Remove Old Tasks
- **Delete**: `smarthunt/src/tasks/` directory (~100 lines)
- **Replaced by**: Pass-based execution model
- **Note**: Task-based parallelism superseded by pass scheduler

#### Step 1.5: Update Module Declarations
- **File**: [smarthunt/src/lib.rs](../smarthunt/src/lib.rs)
- **Action**: Remove module declarations for:
  - `mod engine;`
  - `mod passes;`
  - `mod graph;`
  - `mod tasks;`

#### Step 1.6: Update Main Entry Point
- **File**: [smarthunt/src/main.rs](../smarthunt/src/main.rs)
- **Action**:
  - Remove imports from old engine
  - Update to use new `solidity::analysis::manager::AnalysisManager`
  - Update to use new `detection::manager::DetectionManager`

---

### Phase 2: Remove Migrated Detectors (Safe - Already Reimplemented)

**Estimated impact**: ~1,663 lines removed
**Risk**: Low - new implementations exist and tested
**Prerequisites**: Verify new detectors work correctly

#### Step 2.1: Verify New Detector Implementations
- **Action**: Run tests to confirm new detectors produce equivalent results
- **Test files**: Check `smarthunt/tests/` for detector tests
- **New implementations location**: `smarthunt/src/detection/detectors/ast/`

#### Step 2.2: Remove Migrated Detector Files
- **Delete these 9 files** from `smarthunt/src/detectors/`:
  1. `tx_origin.rs` (150 lines)
  2. `floating_pragma.rs` (121 lines)
  3. `visibility.rs` (150 lines)
  4. `deprecated.rs` (152 lines)
  5. `low_level_call.rs` (151 lines)
  6. `unchecked_call.rs` (154 lines)
  7. `shadowing.rs` (202 lines)
  8. `timestamp_dependence.rs` (152 lines)
  9. `delegatecall.rs` (154 lines)

#### Step 2.3: Update Old Detector Registry
- **File**: [smarthunt/src/detectors/registry.rs](../smarthunt/src/detectors/registry.rs)
- **Action**: Remove registrations for migrated detectors (or mark as deprecated)

---

### Phase 3: Migrate Remaining Detectors (Implementation Required)

**Estimated impact**: 8 detectors to migrate (~2,566 lines)
**Risk**: Medium - requires new implementations
**Prerequisites**: Phase 1 completed

#### Step 3.1: Prioritize Detector Migration

**High Priority** (complex, frequently used):
1. **Reentrancy** ([detectors/reentrancy.rs](../smarthunt/src/detectors/reentrancy.rs), 356 lines)
   - Critical security vulnerability
   - Requires data flow analysis
   - Consider implementing as IR-level pass

2. **CEI Violation** ([detectors/cei_violation.rs](../smarthunt/src/detectors/cei_violation.rs), 311 lines)
   - Related to reentrancy
   - Checks Checks-Effects-Interactions pattern
   - Needs control flow + data flow analysis

3. **Missing Access Control** ([detectors/missing_access_control.rs](../smarthunt/src/detectors/missing_access_control.rs), 265 lines)
   - Critical security issue
   - Can leverage new `modifier_analysis.rs` pass

**Medium Priority** (useful, moderate complexity):
4. **Uninitialized Variables** ([detectors/uninitialized.rs](../smarthunt/src/detectors/uninitialized.rs), 292 lines)
   - Needs data flow analysis
   - Consider IR-level implementation

5. **Dead Code** ([detectors/dead_code.rs](../smarthunt/src/detectors/dead_code.rs), 265 lines)
   - Can leverage new call graph
   - AST-level should suffice

6. **Centralization Risk** ([detectors/centralization_risk.rs](../smarthunt/src/detectors/centralization_risk.rs), 221 lines)
   - Pattern-based detection
   - Can use new modifier analysis

**Low Priority** (simple optimizations):
7. **Constant State Variables** ([detectors/constant_state_var.rs](../smarthunt/src/detectors/constant_state_var.rs), 178 lines)
   - Gas optimization
   - Simple AST-level detector

#### Step 3.2: Implementation Guidelines

For each detector migration:

1. **Create new detector file**: `smarthunt/src/detection/detectors/ast/<name>.rs` or `ir/<name>.rs`
2. **Implement `BugDetectionPass` trait**:
   ```rust
   impl BugDetectionPass for ReentrancyDetector {
       fn detect(&self, context: &AnalysisContext) -> Vec<BugInstance> {
           // Use artifacts from context
           let call_graph = context.get::<CallGraph>(PassId::CallGraphAnalysis)?;
           // Detection logic
       }
   }
   ```
3. **Declare dependencies** in `PassId::dependencies()`:
   - Use new analysis passes (CallGraph, ModifierAnalysis, etc.)
   - Leverage existing artifacts instead of recalculating
4. **Add to registry**: Update [detection/registry.rs](../smarthunt/src/detection/registry.rs)
5. **Add tests**: Create/update test files in `smarthunt/tests/`
6. **Update documentation**: Document new detector capabilities

#### Step 3.3: Remove Old Detector Files (After Migration)

After each detector is migrated and tested:
- Delete old detector file from `smarthunt/src/detectors/`
- Update registry to remove old implementation
- Update tests to use new implementation

---

### Phase 4: Final Cleanup (Remove Old Detector Infrastructure)

**Estimated impact**: ~277 lines removed
**Risk**: Low
**Prerequisites**: All detectors migrated (Phase 3 completed)

#### Step 4.1: Remove Old Detector Infrastructure
- **Delete files**:
  - [smarthunt/src/detectors/detector.rs](../smarthunt/src/detectors/detector.rs) (72 lines) - Old trait
  - [smarthunt/src/detectors/registry.rs](../smarthunt/src/detectors/registry.rs) (184 lines) - Old registry
  - [smarthunt/src/detectors/confidence.rs](../smarthunt/src/detectors/confidence.rs) (21 lines)

#### Step 4.2: Remove Old Detectors Directory
- **Delete**: `smarthunt/src/detectors/` directory entirely
- **Replaced by**: `smarthunt/src/detection/` (new framework)

#### Step 4.3: Update Module Declarations
- **File**: [smarthunt/src/lib.rs](../smarthunt/src/lib.rs)
- **Action**: Remove `mod detectors;` declaration

#### Step 4.4: Final Verification
- Run full test suite: `cargo test`
- Run benchmarks if available
- Test on sample Solidity projects
- Compare detection results between old and new system

---

## Summary

### Total Removal Breakdown

| Phase | Component | Lines | Status |
|-------|-----------|-------|--------|
| **Phase 1** | Old engine, passes, graphs, tasks | ~4,410 | ✅ Safe to remove now |
| **Phase 2** | Migrated detectors (9 files) | ~1,663 | ✅ Safe to remove now |
| **Phase 3** | Pending detector migrations (8 files) | ~2,566 | ⚠️ Migrate first |
| **Phase 4** | Old detector infrastructure | ~277 | ⚠️ After Phase 3 |
| **Total** | | **~8,916** | |

### Immediate Actions (Safe Now)

**Can remove immediately** (~6,073 lines):
- ✅ Phase 1: Old engine, passes, graphs, tasks (4,410 lines)
- ✅ Phase 2: Migrated detectors (1,663 lines)

**Requires migration first** (~2,843 lines):
- ⚠️ Phase 3: 8 remaining detectors (2,566 lines)
- ⚠️ Phase 4: Old detector infrastructure (277 lines)

### Benefits

1. **Code Reduction**: ~35% reduction in codebase size
2. **Improved Architecture**: LLVM-inspired design with proper separation of concerns
3. **Better Maintainability**: Cleaner abstractions, easier to extend
4. **Type Safety**: Type-erased storage with compile-time safety
5. **Automatic Dependencies**: Scheduler handles pass ordering
6. **Unified Framework**: Analysis and detection in one coherent system

### Risks & Mitigation

| Risk | Mitigation |
|------|-----------|
| Breaking existing functionality | Run comprehensive tests after each phase |
| Missing edge cases in new detectors | Compare results with old detectors before removal |
| Incomplete migration | Keep old code temporarily until full verification |
| Test failures | Update tests incrementally with each phase |

---

## Testing Strategy

### Phase 1 Testing
After removing old infrastructure:
1. Verify project compiles: `cargo build`
2. Run unit tests: `cargo test`
3. Test CLI with sample projects
4. Verify new analysis manager works end-to-end

### Phase 2 Testing
For each migrated detector removal:
1. Run detector-specific tests
2. Compare output with new detector
3. Test on real-world Solidity contracts
4. Verify no regressions

### Phase 3 Testing
For each new detector implementation:
1. Implement detector in new framework
2. Run side-by-side comparison with old detector
3. Verify equivalent or better detection
4. Test edge cases
5. Remove old detector only after verification

### Phase 4 Testing
Final cleanup:
1. Full test suite: `cargo test --all`
2. Integration tests with real projects
3. Performance benchmarks
4. Documentation review

---

## Implementation Timeline

### Recommended Sequence

1. **Phase 1** (Immediate): Remove old infrastructure - 1 session
   - Low risk, high impact
   - Immediate code cleanup benefits

2. **Phase 2** (Immediate): Remove migrated detectors - 1 session
   - Low risk, already tested
   - Clean up duplicate implementations

3. **Phase 3** (Multiple sessions): Migrate remaining detectors - 3-5 sessions
   - Prioritize high-security detectors (reentrancy, access control, CEI)
   - Medium priority next (uninitialized, dead code)
   - Low priority last (optimizations)

4. **Phase 4** (Final cleanup): Remove old detector infrastructure - 1 session
   - After all detectors migrated
   - Final cleanup and documentation

**Total estimated effort**: 6-8 implementation sessions

---

## Rollback Strategy

If issues arise:

1. **Phase 1/2 Rollback**:
   - Revert commit(s)
   - Old and new systems were independent

2. **Phase 3 Rollback** (per detector):
   - Keep old detector file temporarily
   - Add feature flag to switch between implementations
   - Remove after full verification

3. **Gradual Migration**:
   - Use feature flags: `--use-new-detectors` vs `--use-old-detectors`
   - Run both in parallel during transition
   - Compare outputs for consistency

---

## Next Steps

1. ✅ Review this plan
2. ✅ Get approval to proceed
3. ⚠️ **Execute Phase 1**: Remove old infrastructure (immediate)
4. ⚠️ **Execute Phase 2**: Remove migrated detectors (immediate)
5. ⚠️ **Plan Phase 3**: Prioritize detector migration order
6. ⚠️ **Implement Phase 3**: Migrate detectors one by one
7. ⚠️ **Execute Phase 4**: Final cleanup

---

## Questions to Resolve

1. Are there any external dependencies on the old `smarthunt/src/engine` or `smarthunt/src/detectors` modules?
2. Should we keep old code in a separate branch for reference?
3. Are there any performance benchmarks to validate the new system?
4. Should we add feature flags for gradual migration?
5. Are there any documentation updates needed?

---

## References

- **New Framework PR/Commit**: `1731977` - "Implement new LLVM-inspired bug framework"
- **Core Analysis Framework**: [solidity/src/analysis/](../solidity/src/analysis/)
- **Detection Framework**: [smarthunt/src/detection/](../smarthunt/src/detection/)
- **Old System**: [smarthunt/src/engine/](../smarthunt/src/engine/), [smarthunt/src/passes/](../smarthunt/src/passes/), [smarthunt/src/detectors/](../smarthunt/src/detectors/)
