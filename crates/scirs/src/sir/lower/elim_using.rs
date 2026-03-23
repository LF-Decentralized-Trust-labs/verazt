//! Eliminate `using-for` directives from contracts.
//!
//! Strips all `MemberDecl::UsingFor` declarations.  Method-style calls on
//! extended types (e.g. `balance.add(x)`) reach CIR as
//! `FunctionCall(FieldAccess(balance, "add"), [x])` and are left to the
//! later pipeline stages.

use crate::sir::lower::CirLowerError;
use crate::sir;

/// Strip all `UsingFor` member declarations.
pub fn run(module: &sir::Module) -> Result<sir::Module, CirLowerError> {
    let decls = module.decls.iter().map(strip_decl).collect();
    Ok(sir::Module { id: module.id.clone(), attrs: module.attrs.clone(), decls })
}

fn strip_decl(decl: &sir::Decl) -> sir::Decl {
    match decl {
        sir::Decl::Contract(c) => sir::Decl::Contract(strip_contract(c)),
        sir::Decl::Dialect(d) => sir::Decl::Dialect(d.clone()),
    }
}

fn strip_contract(contract: &sir::ContractDecl) -> sir::ContractDecl {
    let members = contract
        .members
        .iter()
        .filter(|m| !matches!(m, sir::MemberDecl::UsingFor(_)))
        .cloned()
        .collect();
    sir::ContractDecl { members, ..contract.clone() }
}
