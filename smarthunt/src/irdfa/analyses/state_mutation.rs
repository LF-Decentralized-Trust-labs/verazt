use crate::irdfa::var::VarId;
use solidity::ast::Loc;

/// State mutation event
#[derive(Clone, Debug)]
pub struct StateMutation {
    pub var: VarId,
    pub kind: MutationKind,
    pub loc: Loc,
    pub in_external_call_context: bool,
}

#[derive(Clone, Debug)]
pub enum MutationKind {
    Write,
    Increment,
    Decrement,
    MapUpdate,
    ArrayPush,
    ArrayPop,
}

/// State access (read or write)
#[derive(Clone, Debug)]
pub struct StateAccess {
    pub var: VarId,
    pub kind: AccessKind,
    pub loc: Loc,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessKind {
    Read,
    Write,
}

/// External call information
#[derive(Clone, Debug)]
pub struct ExternalCallInfo {
    pub loc: Loc,
    pub is_delegate: bool,
}

/// State access ordering for CEI analysis
pub struct StateAccessSequence {
    pub accesses: Vec<StateAccess>,
    pub external_calls: Vec<ExternalCallInfo>,
}

impl StateAccessSequence {
    pub fn new() -> Self {
        Self {
            accesses: Vec::new(),
            external_calls: Vec::new(),
        }
    }
}

impl Default for StateAccessSequence {
    fn default() -> Self {
        Self::new()
    }
}

/// State mutation analysis pass (to be integrated with analysis framework)
pub struct StateMutationPass;
