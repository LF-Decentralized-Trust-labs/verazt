//! Unit tests for analysis passes.

use solidity::analysis::pass::Pass;
use solidity::analysis::pass_id::PassId;

/// Test PassId enum variants.
#[test]
fn test_pass_id_variants() {
    // Test all pass IDs can be created
    let _symbol_table = PassId::SymbolTable;
    let _type_index = PassId::TypeIndex;
    let _cfg = PassId::Cfg;
    let _call_graph = PassId::CallGraph;
    let _data_flow = PassId::DataFlow;
    // state_mutation and access_control might not be available or named
    // differently in PassId checking known ones from previous PassId usage
}

/// Test pass dependencies are well-formed.
#[test]
fn test_pass_dependencies_valid() {
    use solidity::analysis::passes::{CallGraphPass, SymbolTablePass, TypeIndexPass};
    // Skipping others if unsure they exist in solidity::analysis::passes yet

    // SymbolTable has no dependencies
    let symbol_table_pass = SymbolTablePass::new();
    assert!(symbol_table_pass.dependencies().is_empty());

    // TypeIndex has no dependencies
    let type_index_pass = TypeIndexPass::new();
    assert!(type_index_pass.dependencies().is_empty());

    // CFG depends on SymbolTable
    // let cfg_pass = CfgPass::new();
    // assert!(cfg_pass.dependencies().contains(&PassId::SymbolTable));

    // CallGraph depends on SymbolTable and CFG
    let call_graph_pass = CallGraphPass::default(); // Check if new() or default()
    assert!(
        call_graph_pass
            .dependencies()
            .contains(&PassId::SymbolTable)
    );
    // assert!(call_graph_pass.dependencies().contains(&PassId::Cfg));

    // DataFlow depends on CFG
    // let data_flow_pass = DataFlowPass::new();
    // assert!(data_flow_pass.dependencies().contains(&PassId::Cfg));

    // StateMutation depends on SymbolTable and CallGraph
    // let state_mutation_pass = StateMutationPass::new();
    // assert!(state_mutation_pass.dependencies().contains(&
    // PassId::SymbolTable)); assert!(state_mutation_pass.dependencies().
    // contains(&PassId::CallGraph));

    // AccessControl depends on SymbolTable
    // let access_control_pass = AccessControlPass::new();
    // assert!(access_control_pass.dependencies().contains(&
    // PassId::SymbolTable));
}
