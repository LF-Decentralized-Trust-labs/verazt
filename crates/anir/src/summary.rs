//! Function summary for interprocedural analysis.

use crate::cfg::FunctionId;
use crate::interfaces::{StorageRef, TaintLabel};
use crate::ops::{ParamIndex, ReturnIndex};
use std::collections::HashMap;
use std::fmt::{self, Display};

/// Summary of a function's effects for interprocedural analysis.
#[derive(Debug, Clone)]
pub struct FunctionSummary {
    pub func_id: FunctionId,
    /// Taint labels flowing into each parameter.
    pub taint_in: HashMap<ParamIndex, TaintLabel>,
    /// Taint labels flowing out of each return value.
    pub taint_out: HashMap<ReturnIndex, TaintLabel>,
    /// Storage locations modified by this function.
    pub modifies: Vec<StorageRef>,
    /// Whether this function may revert.
    pub may_revert: bool,
    /// Whether this function is safe from reentrancy.
    pub reentrancy_safe: bool,
    /// Whether this function transfers value.
    pub value_transfer: bool,
}

impl FunctionSummary {
    pub fn new(func_id: FunctionId) -> Self {
        FunctionSummary {
            func_id,
            taint_in: HashMap::new(),
            taint_out: HashMap::new(),
            modifies: Vec::new(),
            may_revert: false,
            reentrancy_safe: true,
            value_transfer: false,
        }
    }
}

impl Display for FunctionSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Summary({}): ", self.func_id)?;
        if self.may_revert {
            write!(f, "may_revert ")?;
        }
        if !self.reentrancy_safe {
            write!(f, "reentrant ")?;
        }
        if self.value_transfer {
            write!(f, "value_transfer ")?;
        }
        write!(f, "modifies={}", self.modifies.len())
    }
}
