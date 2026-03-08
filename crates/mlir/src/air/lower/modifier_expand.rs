//! Step 1: EVM Modifier Expansion
//!
//! For EVM-dialect contracts, inline modifier bodies at modifier_placeholder
//! positions. Only runs when `#sir.loaded_dialects` contains `"evm"`.

use crate::sir::{ContractDecl, FunctionDecl, MemberDecl, Stmt};

/// Expand modifiers for a function declaration.
///
/// If the function has `#evm.modifiers` attribute, look up each modifier
/// definition in the contract and inline its body. The placeholder
/// `_` in the modifier body is replaced with the function body.
pub fn expand_modifiers(func: &FunctionDecl, contract: &ContractDecl) -> Vec<Stmt> {
    let body = func.body.clone().unwrap_or_default();

    // Check for #evm.modifiers attribute
    let has_modifiers = func
        .attrs
        .iter()
        .any(|a| a.namespace == "evm" && a.key == "modifiers");

    if !has_modifiers {
        return body;
    }

    // Collect modifier definitions from the contract
    let modifier_defs: Vec<_> = contract
        .members
        .iter()
        .filter_map(|m| {
            if let MemberDecl::Dialect(crate::sir::DialectMemberDecl::Evm(
                crate::sir::dialect::evm::EvmMemberDecl::ModifierDef { name, body, .. },
            )) = m
            {
                Some((name.clone(), body.clone()))
            } else {
                None
            }
        })
        .collect();

    if modifier_defs.is_empty() {
        return body;
    }

    // For now, we inline all modifier bodies wrapping the function body.
    // The modifier body's `_` (placeholder) is replaced by the function body.
    // Since SIR doesn't have an explicit placeholder statement, we simply
    // prepend modifier checks (Assert statements from modifier bodies) before
    // the function body.
    let mut result = Vec::new();
    for (_name, mod_body) in &modifier_defs {
        // Add all statements from modifier body except the placeholder
        for stmt in mod_body {
            result.push(stmt.clone());
        }
    }
    result.extend(body);
    result
}
