//! Pass 2a: SIR → AIR lowering.
//!
//! This module orchestrates the five-step transformation from
//! SIR (SIR) into AIR.

pub mod cfg;
pub mod dialect_lower;
pub mod icfg;
pub mod modifier_expand;
pub mod ssa;

use crate::air::module::Module;
use thiserror::Error;

/// Errors that can occur during SIR → AIR lowering.
#[derive(Debug, Error)]
pub enum LowerError {
    #[error("Untagged dialect op after Step 4: {0}")]
    UntaggedDialectOp(String),

    #[error("Modifier not found: {0}")]
    ModifierNotFound(String),

    #[error("SSA renaming error: {0}")]
    SsaError(String),

    #[error("CFG construction error: {0}")]
    CfgError(String),

    #[error("ICFG construction error: {0}")]
    IcfgError(String),
}

/// Lower a SIR Module into an Module.
///
/// This runs the five-step Pass 2a transformation:
///   1. EVM Modifier Expansion (EVM only)
///   2. CFG Construction
///   3. SSA Renaming
///   4. Dialect Lowering
///   5. ICFG + Alias + Taint init
pub fn lower_module(cir: &crate::sir::Module) -> Result<Module, LowerError> {
    use crate::air::cfg::{Function, FunctionId};

    let mut air_module = Module::new(cir.id.clone());

    // Iterate over each contract declaration
    for decl in &cir.decls {
        let contract = match decl {
            crate::sir::Decl::Contract(c) => c,
            crate::sir::Decl::Dialect(_) => continue,
        };

        // Check if this is an EVM dialect module
        let has_evm_dialect = cir
            .attrs
            .iter()
            .any(|a| a.namespace == "sir" && a.key == "loaded_dialects");

        // Process each member declaration
        for member in &contract.members {
            match member {
                crate::sir::MemberDecl::Function(func_decl) => {
                    // Step 1: Modifier expansion (EVM only)
                    let expanded_body = if has_evm_dialect {
                        modifier_expand::expand_modifiers(func_decl, contract)
                    } else {
                        func_decl.body.clone().unwrap_or_default()
                    };

                    // Determine visibility
                    let is_public = func_decl.attrs.iter().any(|a| {
                        a.namespace == "sir"
                            && a.key == "visibility"
                            && matches!(&a.value, crate::sir::AttrValue::String(s) if s == "public" || s == "external")
                    });

                    let func_id = FunctionId(format!("{}.{}", contract.name, func_decl.name));

                    // Step 2: CFG construction
                    let mut blocks = cfg::build_cfg(&expanded_body, &func_decl.params);

                    // Step 3: SSA renaming
                    ssa::rename_to_ssa(&mut blocks);

                    // Step 4: Dialect lowering
                    dialect_lower::lower_dialect_ops(&mut blocks, &cir.attrs)?;

                    let mut air_func = Function::new(func_id, is_public);
                    air_func.blocks = blocks;
                    air_module.functions.push(air_func);
                }
                _ => { /* StorageDecl, TypeAlias, etc. — not lowered to AIR functions */ }
            }
        }
    }

    // Step 5: ICFG, alias sets, and taint graph initialization
    icfg::build_icfg(&mut air_module);

    Ok(air_module)
}
