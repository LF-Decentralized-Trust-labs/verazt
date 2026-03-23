//! Resolve inheritance — inline parent members into derived contracts.
//!
//! For each contract with non-empty `parents`, this pass:
//! 1. Performs C3 linearization of the inheritance chain.
//! 2. Copies storage / function / dialect members from parent contracts into
//!    the derived contract (parents first, then own members — most-derived
//!    wins).
//! 3. Clears the `parents` field so CIR sees a flat contract.
//!
//! `super.method()` calls are left as-is; they are resolved by `elim_modifiers`
//! or future passes that have full call-graph information.

use crate::sir;
use crate::sir::lower::CirLowerError;
use std::collections::HashMap;

/// Resolve inheritance and flatten contracts.
pub fn run(module: &sir::Module) -> Result<sir::Module, CirLowerError> {
    // Build a name → ContractDecl index.
    let contract_map: HashMap<String, &sir::ContractDecl> = module
        .decls
        .iter()
        .filter_map(|d| {
            if let sir::Decl::Contract(c) = d {
                Some((c.name.clone(), c))
            } else {
                None
            }
        })
        .collect();

    let decls = module
        .decls
        .iter()
        .map(|d| match d {
            sir::Decl::Contract(c) => Ok(sir::Decl::Contract(flatten_contract(c, &contract_map)?)),
            sir::Decl::Dialect(d) => Ok(sir::Decl::Dialect(d.clone())),
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(sir::Module { id: module.id.clone(), attrs: module.attrs.clone(), decls })
}

fn flatten_contract(
    contract: &sir::ContractDecl,
    map: &HashMap<String, &sir::ContractDecl>,
) -> Result<sir::ContractDecl, CirLowerError> {
    if contract.parents.is_empty() {
        return Ok(contract.clone());
    }

    // Compute C3 linearisation (ancestors, most-base first).
    let linear = linearize(&contract.name, map)?;

    // Merge members: walk from the most-base ancestor toward the derived
    // contract.  Later (more-derived) declarations shadow older ones for
    // functions (same name).  Storage and dialect members are accumulated.
    let mut merged: Vec<sir::MemberDecl> = Vec::new();

    for ancestor_name in &linear {
        if ancestor_name == &contract.name {
            continue; // own members added last
        }
        if let Some(ancestor) = map.get(ancestor_name) {
            for m in &ancestor.members {
                add_member(&mut merged, m);
            }
        }
    }
    // Own members override anything inherited.
    for m in &contract.members {
        add_member(&mut merged, m);
    }

    Ok(sir::ContractDecl {
        name: contract.name.clone(),
        parents: vec![], // flattened — no longer needed
        attrs: contract.attrs.clone(),
        members: merged,
        span: contract.span,
    })
}

/// Add `member` to `members`, replacing any existing function with the same
/// name (most-derived wins).
fn add_member(members: &mut Vec<sir::MemberDecl>, member: &sir::MemberDecl) {
    match member {
        sir::MemberDecl::Function(f) => {
            // Replace existing function with same name (most-derived wins).
            if let Some(pos) = members
                .iter()
                .position(|m| matches!(m, sir::MemberDecl::Function(g) if g.name == f.name))
            {
                members[pos] = member.clone();
            } else {
                members.push(member.clone());
            }
        }
        sir::MemberDecl::Storage(s) => {
            // Don't duplicate storage slots with the same name.
            if !members
                .iter()
                .any(|m| matches!(m, sir::MemberDecl::Storage(t) if t.name == s.name))
            {
                members.push(member.clone());
            }
        }
        sir::MemberDecl::UsingFor(_) => {
            // UsingFor directives are accumulated (elim_using strips them later).
            members.push(member.clone());
        }
        _ => {
            // TypeAlias, GlobalInvariant, Dialect — always accumulate.
            members.push(member.clone());
        }
    }
}

// ─── C3 Linearisation ────────────────────────────────────────────────────────

fn linearize(
    name: &str,
    map: &HashMap<String, &sir::ContractDecl>,
) -> Result<Vec<String>, CirLowerError> {
    let Some(contract) = map.get(name) else {
        return Ok(vec![name.to_string()]);
    };

    if contract.parents.is_empty() {
        return Ok(vec![name.to_string()]);
    }

    // Build the lists required by C3: L(contract) = contract + merge(L(p1), ...,
    // L(pn), [p1,...,pn])
    let mut lists: Vec<Vec<String>> = Vec::new();
    for parent in &contract.parents {
        lists.push(linearize(parent, map)?);
    }
    lists.push(contract.parents.clone());

    let mut result = vec![name.to_string()];
    result.extend(c3_merge(lists)?);
    Ok(result)
}

fn c3_merge(mut lists: Vec<Vec<String>>) -> Result<Vec<String>, CirLowerError> {
    let mut result = Vec::new();

    loop {
        // Remove empty lists.
        lists.retain(|l| !l.is_empty());
        if lists.is_empty() {
            return Ok(result);
        }

        // Find a good head: a head that does not appear in the tail of any list.
        let mut chosen: Option<String> = None;
        'outer: for candidate_list in &lists {
            let head = candidate_list[0].clone();
            for other_list in &lists {
                if other_list.len() > 1 && other_list[1..].contains(&head) {
                    continue 'outer;
                }
            }
            chosen = Some(head);
            break;
        }

        let head = chosen.ok_or_else(|| {
            CirLowerError::General("Cannot linearize inheritance: cycle detected".into())
        })?;

        result.push(head.clone());

        // Remove `head` from the front of every list that starts with it.
        for list in &mut lists {
            if list.first().map(|s| s.as_str()) == Some(&head) {
                list.remove(0);
            }
        }
    }
}
