//! Rename definitions: qualify function names with the contract (file) scope.

use crate::vyper::ast::defs::*;
use crate::vyper::ast::source_unit::*;

/// Qualify function and definition names with their contract scope.
pub fn rename_defs(su: &SourceUnit) -> SourceUnit {
    let contract_name = std::path::Path::new(&su.path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "vyper".to_string());

    let body = su
        .body
        .iter()
        .map(|elem| rename_elem(elem, &contract_name))
        .collect();

    SourceUnit { path: su.path.clone(), body, loc: su.loc.clone() }
}

fn rename_elem(elem: &SourceUnitElem, contract_name: &str) -> SourceUnitElem {
    match elem {
        SourceUnitElem::Func(f) => {
            let new_name = format!("{contract_name}__{}", f.name);
            SourceUnitElem::Func(FuncDef {
                name: new_name,
                params: f.params.clone(),
                return_type: f.return_type.clone(),
                decorators: f.decorators.clone(),
                doc_string: f.doc_string.clone(),
                body: f.body.clone(),
                loc: f.loc.clone(),
            })
        }
        other => other.clone(),
    }
}
