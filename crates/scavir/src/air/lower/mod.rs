//! Pass 2a: SCIR → ANIR lowering.
//!
//! This module orchestrates the five-step transformation from
//! ContractIR (SCIR) into ANIR.

pub mod cfg;
pub mod dialect_lower;
pub mod icfg;
pub mod modifier_expand;
pub mod ssa;

use crate::air::module::AnirModule;
use thiserror::Error;

/// Errors that can occur during SCIR → ANIR lowering.
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

/// Lower a SCIR Module into an AnirModule.
///
/// This runs the five-step Pass 2a transformation:
///   1. EVM Modifier Expansion (EVM only)
///   2. CFG Construction
///   3. SSA Renaming
///   4. Dialect Lowering
///   5. ICFG + Alias + Taint init
pub fn lower_module(cir: &crate::sir::Module) -> Result<AnirModule, LowerError> {
    use crate::air::cfg::{AnirFunction, FunctionId};

    let mut anir_module = AnirModule::new(cir.id.clone());

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
            .any(|a| a.namespace == "scir" && a.key == "loaded_dialects");

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
                        a.namespace == "scir"
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

                    let mut anir_func = AnirFunction::new(func_id, is_public);
                    anir_func.blocks = blocks;
                    anir_module.functions.push(anir_func);
                }
                _ => { /* StorageDecl, TypeAlias, etc. — not lowered to ANIR functions */ }
            }
        }
    }

    // Step 5: ICFG, alias sets, and taint graph initialization
    icfg::build_icfg(&mut anir_module);

    Ok(anir_module)
}
