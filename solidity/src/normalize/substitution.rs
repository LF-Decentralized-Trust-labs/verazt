//! Substituting `identifier` by an `expression`

use crate::{ast::*, util::*};
use std::collections::HashMap;

//-------------------------------------------------
// Substitute identifier by expression.
//-------------------------------------------------

/// Data structure for substituting identifier by expression.
#[derive(Clone)]
struct IdentExprSubstitutor {
    /// Substitution pairs
    substitution_map: HashMap<Identifier, Expr>,
}

impl IdentExprSubstitutor {
    pub fn new(old_idents: Vec<Identifier>, new_exprs: Vec<Expr>) -> Self {
        if old_idents.len() != new_exprs.len() {
            panic!("Unable to construct subsitution map: \n{old_idents:?}\n -- \n{new_exprs:?}")
        }

        let mut substitution_map = HashMap::new();
        for pair in old_idents.iter().zip(new_exprs.iter()) {
            substitution_map.insert(pair.0.clone(), pair.1.clone());
        }
        IdentExprSubstitutor { substitution_map }
    }
}

impl Map<'_> for IdentExprSubstitutor {
    /// Override `map_expr` to substitute identifier expression.
    fn map_expr(&mut self, expr: &Expr) -> Expr {
        if let Expr::Ident(ident) = expr {
            if let Some(subst_expr) = self.substitution_map.get(ident) {
                return subst_expr.clone();
            }
        }
        map::default::map_expr(self, expr)
    }
}

pub fn substitute_stmt(params: Vec<Identifier>, args: Vec<Expr>, stmt: &Stmt) -> Stmt {
    let mut substitutor = IdentExprSubstitutor::new(params, args);
    substitutor.map_stmt(stmt)
}

//-------------------------------------------------
// Substitute name
//-------------------------------------------------

/// Data structure for substituting names of contract, error, variables,
/// expressions, etc.
pub struct NameSubstitutor {
    substitution_map: HashMap<Name, Name>,
}

impl NameSubstitutor {
    pub fn new(old_names: &[Name], new_names: &[Name]) -> Self {
        if old_names.len() != new_names.len() {
            panic!("Unable to construct name subsitution map:\n{old_names:?}\n -- \n{new_names:?}")
        }

        let mut substitution_map = HashMap::new();
        for pair in old_names.iter().zip(new_names.iter()) {
            substitution_map.insert(pair.0.clone(), pair.1.clone());
        }
        NameSubstitutor { substitution_map }
    }

    pub fn substitute_source_unit_elems(
        &mut self,
        elems: &[SourceUnitElem],
    ) -> Vec<SourceUnitElem> {
        elems
            .iter()
            .map(|elem| self.map_source_unit_elem(elem))
            .collect()
    }
}

impl Map<'_> for NameSubstitutor {
    /// Override `map_source_unit_elem` to make a new scope of renaming
    /// variables.
    fn map_source_unit_elem(&mut self, elem: &SourceUnitElem) -> SourceUnitElem {
        let saved_substitution_map = self.substitution_map.clone();
        let nelem: SourceUnitElem = match elem {
            SourceUnitElem::Var(vdecl) => map::default::map_var_decl(self, vdecl).into(),
            _ => map::default::map_source_unit_elem(self, elem),
        };
        self.substitution_map = saved_substitution_map;
        nelem
    }

    /// Override `map_block` to make a new scope of renaming variables.
    fn map_block(&mut self, block: &Block) -> Block {
        let saved_substitution_map = self.substitution_map.clone();
        let nblock = map::default::map_block(self, block);
        self.substitution_map = saved_substitution_map;
        nblock
    }

    /// Override `map_var_decl` to avoid renaming local variables
    /// that are declared with the same names of the names to be
    /// substituted.
    fn map_var_decl(&mut self, vdecl: &VarDecl) -> VarDecl {
        if self.substitution_map.contains_key(&vdecl.name) {
            self.substitution_map.remove(&vdecl.name);
        }
        map::default::map_var_decl(self, vdecl)
    }

    /// Override `map_func_def` to avoid renaming functions that are
    /// defined with the same names of the names to be substituted.
    fn map_func_def(&mut self, func: &FuncDef) -> FuncDef {
        // if self.substitution_map.contains_key(&func.name) {
        //     self.substitution_map.remove(&func.name);
        // }
        map::default::map_func_def(self, func)
    }

    /// Override `map_error_def` to avoid renaming error that are defined
    /// with the same names of the names to be substituted.
    fn map_error_def(&mut self, error: &ErrorDef) -> ErrorDef {
        if self.substitution_map.contains_key(&error.name) {
            self.substitution_map.remove(&error.name);
        }
        map::default::map_error_def(self, error)
    }

    /// Override `map_event_def` to avoid renaming event that are defined
    /// with the same names of the names to be substituted.
    fn map_event_def(&mut self, event: &EventDef) -> EventDef {
        if self.substitution_map.contains_key(&event.name) {
            self.substitution_map.remove(&event.name);
        }
        map::default::map_event_def(self, event)
    }

    /// Override `map_struct_def` to avoid renaming struct that are
    /// defined with the same names of the names to be substituted.
    fn map_struct_def(&mut self, struct_: &StructDef) -> StructDef {
        if self.substitution_map.contains_key(&struct_.name) {
            self.substitution_map.remove(&struct_.name);
        }
        map::default::map_struct_def(self, struct_)
    }

    /// Override `map_enum_def` to avoid renaming enum that are defined
    /// with the same names of the names to be substituted.
    fn map_enum_def(&mut self, enum_: &EnumDef) -> EnumDef {
        if self.substitution_map.contains_key(&enum_.name) {
            self.substitution_map.remove(&enum_.name);
        }
        map::default::map_enum_def(self, enum_)
    }

    /// Override `map_name` to substitute identifier.
    fn map_name(&mut self, name: &Name) -> Name {
        match self.substitution_map.get(name) {
            Some(nname) => nname.clone(),
            None => name.clone(),
        }
    }
}
