//! FIR data structures: Module, Function, Terminator.
//!
//! Each BIR `Function` is lowered into a family of FIR functions —
//! one per basic block. Terminators become tail calls.

use crate::bir::cfg::FunctionId;
use crate::bir::ops::{Op, OpRef, SsaName};
use crate::sir::Type;
use std::fmt::{self, Display};

/// The top-level FIR module — one per BIR Module.
#[derive(Debug, Clone)]
pub struct Module {
    pub source_module_id: String,
    pub functions: Vec<Function>,
}

/// A FIR function — one per BIR basic block.
///
/// For a BIR function `@Foo` with blocks `%bb0`, `%bb1`, `%bb2`:
/// - `%bb0` → `@Foo`     (entry)
/// - `%bb1` → `@Foo$1`
/// - `%bb2` → `@Foo$2`
#[derive(Debug, Clone)]
pub struct Function {
    pub id: FunctionId,
    pub params: Vec<(SsaName, Type)>,
    pub body: Vec<Op>,
    pub term: Terminator,
}

/// Data for a single tail call.
#[derive(Debug, Clone)]
pub struct TailCallData {
    pub callee: FunctionId,
    pub args: Vec<OpRef>,
}

/// The terminator of a FIR function body.
#[derive(Debug, Clone)]
pub enum Terminator {
    /// Unconditional tail call.
    TailCall(TailCallData),
    /// Conditional branch → two tail calls.
    Branch {
        cond: OpRef,
        then_call: TailCallData,
        else_call: TailCallData,
    },
    /// Return from the original function.
    Return(Vec<OpRef>),
    /// Revert (transaction failure).
    Revert,
    /// Unreachable code.
    Unreachable,
}

// ═══════════════════════════════════════════════════════════════════
// Implementations
// ═══════════════════════════════════════════════════════════════════

impl Module {
    pub fn new(source_module_id: String) -> Self {
        Module { source_module_id, functions: Vec::new() }
    }
}

impl Function {
    pub fn new(id: FunctionId, params: Vec<(SsaName, Type)>, body: Vec<Op>, term: Terminator) -> Self {
        Function { id, params, body, term }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Display implementations
// ═══════════════════════════════════════════════════════════════════

impl Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, func) in self.functions.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{func}")?;
        }
        Ok(())
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params: Vec<_> = self.params.iter().map(|(name, ty)| format!("{name}: {ty}")).collect();
        writeln!(f, "function {}({}) {{", self.id, params.join(", "))?;
        for op in &self.body {
            writeln!(f, "    {op}")?;
        }
        writeln!(f, "    {}", self.term)?;
        writeln!(f, "}}")
    }
}

impl Display for TailCallData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args: Vec<_> = self.args.iter().map(|a| a.to_string()).collect();
        write!(f, "tail call {}({})", self.callee, args.join(", "))
    }
}

impl Display for Terminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Terminator::TailCall(tc) => write!(f, "{tc}"),
            Terminator::Branch { cond, then_call, else_call } => {
                write!(f, "if {cond} then {then_call} else {else_call}")
            }
            Terminator::Return(vals) => {
                let parts: Vec<_> = vals.iter().map(|r| r.to_string()).collect();
                write!(f, "return {}", parts.join(", "))
            }
            Terminator::Revert => write!(f, "revert"),
            Terminator::Unreachable => write!(f, "unreachable"),
        }
    }
}
