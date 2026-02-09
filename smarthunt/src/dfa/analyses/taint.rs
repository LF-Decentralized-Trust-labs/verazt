use crate::dfa::var::VarId;
use solidity::ast::Loc;
use std::collections::HashSet;

/// Taint sources specific to Solidity
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum TaintSource {
    MsgSender,           // msg.sender
    MsgValue,            // msg.value
    MsgData,             // msg.data
    TxOrigin,            // tx.origin
    BlockTimestamp,      // block.timestamp
    BlockNumber,         // block.number
    ExternalCallResult,  // Result of external call
    CallDataLoad,        // Direct calldata access
    FunctionParameter,   // Function input parameter
    StorageRead,         // Value read from storage
}

/// Taint sinks (sensitive operations)
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum TaintSink {
    EtherTransfer,       // .transfer(), .send(), .call{value:}()
    Delegatecall,        // .delegatecall()
    Selfdestruct,        // selfdestruct()
    StorageWrite,        // State variable assignment
    ExternalCallAddress, // Address in external call
    ArrayIndex,          // Array/mapping index (for DoS)
    LoopBound,           // Loop iteration bound
}

/// Taint state for a variable
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TaintState {
    Untainted,
    Tainted {
        sources: HashSet<TaintSource>,
        propagation_path: Vec<Loc>,
    },
}

/// Taint flow from source to sink
#[derive(Clone, Debug)]
pub struct TaintFlow {
    pub source: TaintSource,
    pub sink: TaintSink,
    pub var: VarId,
    pub path: Vec<Loc>,
}

/// Taint analysis pass (to be integrated with analysis framework)
pub struct TaintAnalysisPass;
