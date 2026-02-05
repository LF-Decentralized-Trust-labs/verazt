//-------------------------------------------------------------------------
// Data structures representing analysis context
//-------------------------------------------------------------------------

use crate::engine::config::Config;
use crate::graph::{
    SymbolTable, TypeIndex, CfgCollection, CallGraph, InheritanceGraph,
    FunctionId,
};
use solidity::ast::{FuncDef, SourceUnit, VarDecl, Name, ContractDef};
use solidity::ir;
use std::collections::HashSet;

/// The main analysis context holding all analysis artifacts.
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    /// Original source units (includes type info from solc)
    pub source_units: Vec<SourceUnit>,
    
    /// Optional IR units
    pub ir_units: Option<Vec<ir::SourceUnit>>,
    
    /// Symbol table for fast lookups
    pub symbols: Option<SymbolTable>,
    
    /// Type index for type queries
    pub type_index: Option<TypeIndex>,
    
    /// Inheritance graph
    pub inheritance: Option<InheritanceGraph>,
    
    /// Control flow graphs for all functions
    pub cfgs: Option<CfgCollection>,
    
    /// Call graph
    pub call_graph: Option<CallGraph>,
    
    /// Taint graph (for data flow analysis)
    pub taint_graph: Option<TaintGraph>,
    
    /// Definition-use chains
    pub def_use_chains: Option<DefUseChains>,
    
    /// State mutation map
    pub state_mutations: Option<StateMutationMap>,
    
    /// Access control information
    pub access_control: Option<AccessControlInfo>,
    
    /// Configuration
    pub config: Config,
    
    /// Set of completed passes
    pub completed_passes: HashSet<String>,
}

impl AnalysisContext {
    /// Create a new analysis context.
    pub fn new(source_units: Vec<SourceUnit>, config: Config) -> Self {
        Self {
            source_units,
            ir_units: None,
            symbols: None,
            type_index: None,
            inheritance: None,
            cfgs: None,
            call_graph: None,
            taint_graph: None,
            def_use_chains: None,
            state_mutations: None,
            access_control: None,
            config,
            completed_passes: HashSet::new(),
        }
    }

    /// Create context with IR.
    pub fn with_ir(mut self, ir_units: Vec<ir::SourceUnit>) -> Self {
        self.ir_units = Some(ir_units);
        self
    }

    /// Mark a pass as completed.
    pub fn mark_pass_completed(&mut self, pass_id: &str) {
        self.completed_passes.insert(pass_id.to_string());
    }

    /// Check if a pass has been completed.
    pub fn is_pass_completed(&self, pass_id: &str) -> bool {
        self.completed_passes.contains(pass_id)
    }

    /// Get the symbol table (panics if not built yet).
    pub fn symbol_table(&self) -> &SymbolTable {
        self.symbols.as_ref().expect("Symbol table not built yet")
    }

    /// Get the type index (panics if not built yet).
    pub fn type_index(&self) -> &TypeIndex {
        self.type_index.as_ref().expect("Type index not built yet")
    }

    /// Get the inheritance graph (panics if not built yet).
    pub fn inheritance_graph(&self) -> &InheritanceGraph {
        self.inheritance.as_ref().expect("Inheritance graph not built yet")
    }

    /// Get the CFG collection (panics if not built yet).
    pub fn cfg_collection(&self) -> &CfgCollection {
        self.cfgs.as_ref().expect("CFGs not built yet")
    }

    /// Get the call graph (panics if not built yet).
    pub fn call_graph(&self) -> &CallGraph {
        self.call_graph.as_ref().expect("Call graph not built yet")
    }

    /// Get CFG for a specific function.
    pub fn get_cfg(&self, func_id: &FunctionId) -> Option<&crate::graph::ControlFlowGraph> {
        self.cfgs.as_ref()?.get(func_id)
    }

    /// Get all contracts from source units.
    pub fn get_all_contracts(&self) -> Vec<&ContractDef> {
        self.source_units
            .iter()
            .flat_map(|su| su.elems.iter())
            .filter_map(|elem| {
                if let solidity::ast::SourceUnitElem::Contract(c) = elem {
                    Some(c)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all functions from source units.
    pub fn get_all_functions(&self) -> Vec<(&Option<ContractDef>, &FuncDef)> {
        let mut result = Vec::new();
        
        for su in &self.source_units {
            for elem in &su.elems {
                match elem {
                    solidity::ast::SourceUnitElem::Func(f) => {
                        result.push((&None, f));
                    }
                    solidity::ast::SourceUnitElem::Contract(c) => {
                        for body_elem in &c.body {
                            if let solidity::ast::ContractElem::Func(f) = body_elem {
                                // We can't easily return a reference to an owned value
                                // so we'll use a different approach
                                result.push((&None, f));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        
        result
    }

    /// Check if a function modifies state.
    pub fn modifies_state(&self, func_id: &FunctionId) -> bool {
        self.state_mutations
            .as_ref()
            .map(|sm| sm.modifies_state(func_id))
            .unwrap_or(false)
    }

    /// Get state variables written by a function.
    pub fn get_state_writes(&self, func_id: &FunctionId) -> Vec<Name> {
        self.state_mutations
            .as_ref()
            .map(|sm| sm.get_writes(func_id))
            .unwrap_or_default()
    }

    /// Get state variables read by a function.
    pub fn get_state_reads(&self, func_id: &FunctionId) -> Vec<Name> {
        self.state_mutations
            .as_ref()
            .map(|sm| sm.get_reads(func_id))
            .unwrap_or_default()
    }

    /// Check if a function has access control modifiers.
    pub fn has_access_control(&self, func_id: &FunctionId) -> bool {
        self.access_control
            .as_ref()
            .map(|ac| ac.has_access_control(func_id))
            .unwrap_or(false)
    }
}

//-------------------------------------------------------------------------
// Additional data structures for analysis artifacts
//-------------------------------------------------------------------------

/// Taint source types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaintSource {
    /// User input from function parameter
    FunctionParameter(Name),
    /// External call return value
    ExternalCallReturn,
    /// msg.sender
    MsgSender,
    /// msg.value
    MsgValue,
    /// block.timestamp
    BlockTimestamp,
    /// tx.origin
    TxOrigin,
}

/// Taint sink types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaintSink {
    /// External call target
    ExternalCallTarget,
    /// External call value
    ExternalCallValue,
    /// State variable write
    StateWrite(Name),
    /// Array index
    ArrayIndex,
    /// Assertion condition
    AssertCondition,
}

/// Taint graph for tracking data flow.
#[derive(Debug, Clone, Default)]
pub struct TaintGraph {
    /// Tainted variables in each function
    pub tainted_vars: std::collections::HashMap<FunctionId, HashSet<Name>>,
    /// Sources for each tainted variable
    pub sources: std::collections::HashMap<(FunctionId, Name), Vec<TaintSource>>,
    /// Sinks reached by tainted data
    pub sinks: Vec<(FunctionId, TaintSink, Vec<TaintSource>)>,
}

impl TaintGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_tainted(&self, func: &FunctionId, var: &Name) -> bool {
        self.tainted_vars
            .get(func)
            .map(|vars| vars.contains(var))
            .unwrap_or(false)
    }

    pub fn get_sources(&self, func: &FunctionId, var: &Name) -> Vec<TaintSource> {
        self.sources
            .get(&(func.clone(), var.clone()))
            .cloned()
            .unwrap_or_default()
    }
}

/// Definition-use chains for variables.
#[derive(Debug, Clone, Default)]
pub struct DefUseChains {
    /// Definitions: variable -> locations where it's defined
    pub definitions: std::collections::HashMap<Name, Vec<solidity::ast::Loc>>,
    /// Uses: variable -> locations where it's used
    pub uses: std::collections::HashMap<Name, Vec<solidity::ast::Loc>>,
}

impl DefUseChains {
    pub fn new() -> Self {
        Self::default()
    }
}

/// State mutation map.
#[derive(Debug, Clone, Default)]
pub struct StateMutationMap {
    /// Functions that read each state variable
    pub reads: std::collections::HashMap<Name, Vec<FunctionId>>,
    /// Functions that write each state variable
    pub writes: std::collections::HashMap<Name, Vec<FunctionId>>,
    /// State variables read by each function
    pub function_reads: std::collections::HashMap<FunctionId, Vec<Name>>,
    /// State variables written by each function
    pub function_writes: std::collections::HashMap<FunctionId, Vec<Name>>,
}

impl StateMutationMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn modifies_state(&self, func: &FunctionId) -> bool {
        self.function_writes
            .get(func)
            .map(|writes| !writes.is_empty())
            .unwrap_or(false)
    }

    pub fn get_writes(&self, func: &FunctionId) -> Vec<Name> {
        self.function_writes.get(func).cloned().unwrap_or_default()
    }

    pub fn get_reads(&self, func: &FunctionId) -> Vec<Name> {
        self.function_reads.get(func).cloned().unwrap_or_default()
    }

    pub fn add_read(&mut self, func: FunctionId, var: Name) {
        self.reads.entry(var.clone()).or_default().push(func.clone());
        self.function_reads.entry(func).or_default().push(var);
    }

    pub fn add_write(&mut self, func: FunctionId, var: Name) {
        self.writes.entry(var.clone()).or_default().push(func.clone());
        self.function_writes.entry(func).or_default().push(var);
    }
}

/// Access control information.
#[derive(Debug, Clone, Default)]
pub struct AccessControlInfo {
    /// Functions with access control modifiers
    pub protected_functions: HashSet<FunctionId>,
    /// Modifier names that are access control related
    pub access_control_modifiers: HashSet<String>,
    /// Functions that check msg.sender
    pub sender_checked_functions: HashSet<FunctionId>,
}

impl AccessControlInfo {
    pub fn new() -> Self {
        let mut info = Self::default();
        // Common access control modifier names
        info.access_control_modifiers.insert("onlyOwner".to_string());
        info.access_control_modifiers.insert("onlyAdmin".to_string());
        info.access_control_modifiers.insert("onlyRole".to_string());
        info.access_control_modifiers.insert("onlyMinter".to_string());
        info.access_control_modifiers.insert("onlyPauser".to_string());
        info.access_control_modifiers.insert("onlyGovernance".to_string());
        info.access_control_modifiers.insert("onlyController".to_string());
        info.access_control_modifiers.insert("onlyAuthorized".to_string());
        info.access_control_modifiers.insert("whenNotPaused".to_string());
        info.access_control_modifiers.insert("nonReentrant".to_string());
        info
    }

    pub fn has_access_control(&self, func: &FunctionId) -> bool {
        self.protected_functions.contains(func) || self.sender_checked_functions.contains(func)
    }

    pub fn mark_protected(&mut self, func: FunctionId) {
        self.protected_functions.insert(func);
    }

    pub fn mark_sender_checked(&mut self, func: FunctionId) {
        self.sender_checked_functions.insert(func);
    }
}

