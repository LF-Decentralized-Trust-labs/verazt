//! Pass to strip `override` and `virtual` specifiers from **renamed**
//! function and variable definitions.  After renaming, these specifiers
//! reference stale names and can produce invalid Solidity when re-printed.
//!
//! Non-renamed functions (those without a disambiguation index) keep their
//! original specifiers because Solidity still needs them for proper
//! override resolution.

use crate::solidity::ast::utils::map::Map;
use crate::solidity::ast::*;
use std::collections::HashSet;

struct StripSpecifiers {
    /// Base contract names of the contract currently being processed.
    current_base_names: HashSet<String>,
}

impl<'a> Map<'a> for StripSpecifiers {
    fn map_contract_def(&mut self, contract: &'a ContractDef) -> ContractDef {
        // Record the base contracts for the contract currently being processed
        // so that map_var_decl can check if an override reference is valid.
        self.current_base_names = contract
            .base_contracts
            .iter()
            .map(|b| b.name.base.clone())
            .collect();
        let contract = crate::solidity::ast::utils::map::default::map_contract_def(self, contract);
        self.current_base_names.clear();
        // Preserve `is_abstract` — contracts may need to remain abstract for
        // valid Solidity output (e.g. contracts with internal constructors, or
        // contracts that don't call parent constructors).
        // Do NOT clear `is_abstract` here.
        contract
    }

    fn map_func_def(&mut self, func: &'a FuncDef) -> FuncDef {
        let mut func = crate::solidity::ast::utils::map::default::map_func_def(self, func);
        // Special functions (fallback, receive, constructor) cannot be renamed
        // and may still legitimately override each other, so preserve their
        // virtual/override specifiers.
        let is_special =
            matches!(func.kind, FuncKind::Fallback | FuncKind::Receive | FuncKind::Constructor);
        if is_special {
            return func;
        }

        // Only strip specifiers from renamed functions (those with a
        // disambiguation index).  Non-renamed functions still participate
        // in Solidity's override mechanism and need these specifiers.
        let is_renamed = func.name.index.is_some();
        if !is_renamed {
            // Non-renamed functions keep their original virtual/override
            // specifiers unchanged.
            return func;
        }

        // Renamed functions have unique names — strip virtual/override.
        if func.body.is_some() {
            func.is_virtual = false;
        }
        func.overriding = Overriding::None;
        func
    }

    fn map_var_decl(&mut self, vdecl: &'a VarDecl) -> VarDecl {
        let mut vdecl = crate::solidity::ast::utils::map::default::map_var_decl(self, vdecl);
        // For state variables with override(A, B), keep the explicit contract
        // list when those base contracts still exist in the inheritance chain.
        // If some referenced contracts were removed by inheritance resolution,
        // filter them out. If none remain, clear the override entirely.
        if let Overriding::Some(ref names) = vdecl.overriding {
            let valid_names: Vec<Name> = names
                .iter()
                .filter(|n| self.current_base_names.contains(&n.base))
                .cloned()
                .collect();
            vdecl.overriding = if valid_names.is_empty() {
                Overriding::None
            } else {
                Overriding::Some(valid_names)
            };
        }
        vdecl
    }
}

/// Strip `override` and `virtual` specifiers from renamed definitions in
/// the given source units.
pub fn strip_specifiers(source_units: &[SourceUnit]) -> Vec<SourceUnit> {
    let mut mapper = StripSpecifiers { current_base_names: HashSet::new() };
    mapper.map_source_units(source_units)
}
