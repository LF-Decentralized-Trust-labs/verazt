//! Unit tests for analysis passes.

use smarthunt::passes::{PassId, AnalysisPass};

/// Test PassId enum variants.
#[test]
fn test_pass_id_variants() {
    // Test all pass IDs can be created
    let _symbol_table = PassId::SymbolTable;
    let _type_index = PassId::TypeIndex;
    let _cfg = PassId::Cfg;
    let _call_graph = PassId::CallGraph;
    let _data_flow = PassId::DataFlow;
    let _state_mutation = PassId::StateMutation;
    let _access_control = PassId::AccessControl;
}

/// Test pass dependencies are well-formed.
#[test]
fn test_pass_dependencies_valid() {
    use smarthunt::passes::{
        symbol_table::SymbolTablePass,
        type_index::TypeIndexPass,
        cfg::CfgPass,
        call_graph::CallGraphPass,
        data_flow::DataFlowPass,
        state_mutation::StateMutationPass,
        access_control::AccessControlPass,
    };
    
    // SymbolTable has no dependencies
    let symbol_table_pass = SymbolTablePass::new();
    assert!(symbol_table_pass.dependencies().is_empty());
    
    // TypeIndex depends on SymbolTable
    let type_index_pass = TypeIndexPass::new();
    assert!(type_index_pass.dependencies().contains(&PassId::SymbolTable));
    
    // CFG depends on SymbolTable
    let cfg_pass = CfgPass::new();
    assert!(cfg_pass.dependencies().contains(&PassId::SymbolTable));
    
    // CallGraph depends on SymbolTable and CFG
    let call_graph_pass = CallGraphPass::new();
    assert!(call_graph_pass.dependencies().contains(&PassId::SymbolTable));
    assert!(call_graph_pass.dependencies().contains(&PassId::Cfg));
    
    // DataFlow depends on CFG
    let data_flow_pass = DataFlowPass::new();
    assert!(data_flow_pass.dependencies().contains(&PassId::Cfg));
    
    // StateMutation depends on SymbolTable and CallGraph
    let state_mutation_pass = StateMutationPass::new();
    assert!(state_mutation_pass.dependencies().contains(&PassId::SymbolTable));
    assert!(state_mutation_pass.dependencies().contains(&PassId::CallGraph));
    
    // AccessControl depends on SymbolTable
    let access_control_pass = AccessControlPass::new();
    assert!(access_control_pass.dependencies().contains(&PassId::SymbolTable));
}
