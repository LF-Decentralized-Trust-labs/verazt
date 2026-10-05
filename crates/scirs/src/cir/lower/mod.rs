//! Pass 2a: CIR → BIR lowering.
//!
//! This module orchestrates the three-step transformation from
//! CIR (Canonical IR) into BIR.

pub mod cfg;
pub mod icfg;
pub mod ssa;

#[cfg(test)]
pub(crate) mod test_support;

use crate::bir::cfg::{Function, FunctionId};
use crate::bir::module::Module;
use crate::cir::{AttrValue, CanonDecl, CanonFunctionDecl, CanonMemberDecl, CanonModule};
use thiserror::Error;

/// Errors that can occur during CIR → BIR lowering.
#[derive(Debug, Error)]
pub enum LowerError {
    #[error("SSA renaming error: {0}")]
    SsaError(String),

    #[error("CFG construction error: {0}")]
    CfgError(String),

    #[error("ICFG construction error: {0}")]
    IcfgError(String),
}

/// Lower a CIR CanonModule into an BIR Module.
///
/// This runs the three-step Pass 2a transformation:
///   1. CFG Construction in SSA form (CIR already makes chain semantics
///      explicit, so expressions map directly onto BIR ops)
///   2. SSA Numbering
///   3. ICFG + Alias + Taint init
///
/// NOTE: Modifier expansion is handled by the CIR lowering pass.
pub fn lower_module(cir: &CanonModule) -> Result<Module, LowerError> {
    let mut bir_module = Module::new(cir.id.clone());

    for decl in &cir.decls {
        let contract = match decl {
            CanonDecl::Contract(c) => c,
            CanonDecl::Dialect(_) => continue,
        };

        for member in &contract.members {
            match member {
                CanonMemberDecl::Function(func_decl) => {
                    let func_id = FunctionId(format!("{}.{}", contract.name, func_decl.name));

                    // Step 1: CFG construction in SSA form
                    let mut blocks =
                        cfg::build_cfg(&func_decl.body, &func_decl.params, &contract.name);

                    // Step 2: SSA numbering
                    ssa::rename_to_ssa(&mut blocks);

                    let mut bir_func = Function::new(func_id, is_public(func_decl))
                        .with_attrs(func_decl.attrs.clone())
                        .with_span(func_decl.span.clone());
                    bir_func.blocks = blocks;
                    bir_module.functions.push(bir_func);
                }
                CanonMemberDecl::Storage(_)
                | CanonMemberDecl::TypeAlias(_)
                | CanonMemberDecl::GlobalInvariant(_)
                | CanonMemberDecl::Dialect(_) => {}
            }
        }
    }

    // Step 3: ICFG, alias sets, and taint graph initialization
    icfg::build_icfg(&mut bir_module);

    Ok(bir_module)
}

/// Returns `true` if the function is callable from outside the contract.
fn is_public(func: &CanonFunctionDecl) -> bool {
    func.attrs.iter().any(|a| {
        a.namespace == "sir"
            && a.key == "visibility"
            && matches!(&a.value, AttrValue::String(s) if s == "public" || s == "external")
    })
}
