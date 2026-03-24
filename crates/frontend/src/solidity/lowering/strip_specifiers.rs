//! Pass to strip `override` and `virtual` specifiers from **renamed**
//! function and variable definitions.  After renaming, these specifiers
//! reference stale names and can produce invalid Solidity when re-printed.
//!
//! Non-renamed functions (those without a disambiguation index) keep their
//! original specifiers because Solidity still needs them for proper
//! override resolution.

use crate::solidity::ast::utils::map::Map;
use crate::solidity::ast::*;

struct StripSpecifiers;

impl<'a> Map<'a> for StripSpecifiers {
    fn map_contract_def(&mut self, contract: &'a ContractDef) -> ContractDef {
        let contract =
            crate::solidity::ast::utils::map::default::map_contract_def(self, contract);
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
        let is_special = matches!(
            func.kind,
            FuncKind::Fallback | FuncKind::Receive | FuncKind::Constructor
        );
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
        // For state variables, sanitize `Overriding::Some(names)` to
        // `Overriding::All` to avoid stale contract name references, but
        // keep the override marker since state variables may override
        // virtual functions in base contracts.
        if matches!(vdecl.overriding, Overriding::Some(_)) {
            vdecl.overriding = Overriding::All;
        }
        vdecl
    }
}

/// Strip `override` and `virtual` specifiers from renamed definitions in
/// the given source units.
pub fn strip_specifiers(source_units: &[SourceUnit]) -> Vec<SourceUnit> {
    let mut mapper = StripSpecifiers;
    mapper.map_source_units(source_units)
}
