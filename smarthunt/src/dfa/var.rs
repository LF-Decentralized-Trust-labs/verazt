use std::fmt;
use std::hash::{Hash, Hasher};

/// Variable scope in Solidity
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum VarScope {
    /// Function local variable
    Local,
    /// Contract state variable
    State { contract: String },
    /// Memory location
    Memory,
    /// Storage slot
    Storage,
}

/// Variable identifier for tracking in data flow analysis
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VarId {
    pub name: String,
    pub scope: VarScope,
}

impl VarId {
    pub fn new(name: impl Into<String>, scope: VarScope) -> Self {
        Self { name: name.into(), scope }
    }

    pub fn local(name: impl Into<String>) -> Self {
        Self::new(name, VarScope::Local)
    }

    pub fn state(name: impl Into<String>, contract: impl Into<String>) -> Self {
        Self::new(name, VarScope::State { contract: contract.into() })
    }

    pub fn is_state_var(&self) -> bool {
        matches!(self.scope, VarScope::State { .. })
    }

    pub fn is_local(&self) -> bool {
        matches!(self.scope, VarScope::Local)
    }
}

impl Hash for VarId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.scope.hash(state);
    }
}

impl fmt::Display for VarId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.scope {
            VarScope::Local => write!(f, "{}", self.name),
            VarScope::State { contract } => write!(f, "{}::{}", contract, self.name),
            VarScope::Memory => write!(f, "memory::{}", self.name),
            VarScope::Storage => write!(f, "storage::{}", self.name),
        }
    }
}
