//! Module for renaming Yul variables.

use crate::ast::yul::{*, utils::{YulMap, yul_map_default}};
use crate::ast::{Name, NamingEnv};

/// Function to rename variables in a YulSourceUnit.
pub fn rename_yul_variables(source_unit: &YulSourceUnit) -> YulSourceUnit {
    let mut renamer = YulRenamer::new(None);
    renamer.rename_source_unit(source_unit)
}

/// Function to rename variables in a YulBlock.
pub fn rename_yul_variables_in_block(block: &YulBlock, env: NamingEnv) -> YulBlock {
    let mut renamer = YulRenamer::new(Some(env));
    renamer.rename_block(block)
}

//-------------------------------------------------
// Rename Yul variables.
//-------------------------------------------------

/// Data structure to rename Yul variables.
///
/// This data structure should be kept private.
pub(super) struct YulRenamer {
    env: NamingEnv,
}

impl YulRenamer {
    pub fn new(env: Option<NamingEnv>) -> Self {
        let env = match env {
            Some(env) => env,
            None => NamingEnv::new(),
        };
        YulRenamer { env }
    }

    pub fn create_new_idents(&mut self, idents: &[YulIdentifier]) -> Vec<YulIdentifier> {
        idents
            .iter()
            .map(|ident| {
                let name = &ident.name;
                let (nname, nenv) = self.env.create_new_name(&name.base);
                self.env = nenv;
                YulIdentifier { name: nname, ..ident.clone() }
            })
            .collect()
    }

    pub fn rename_source_unit(&mut self, source_unit: &YulSourceUnit) -> YulSourceUnit {
        self.map_yul_source_unit(source_unit)
    }

    pub fn rename_block(&mut self, block: &YulBlock) -> YulBlock {
        self.map_yul_block(block)
    }
}

impl YulMap for YulRenamer {
    fn map_yul_func_def(&mut self, func: &YulFuncDef) -> YulFuncDef {
        // Save the current renaming environment
        let stored_env = self.env.clone();

        // Transform the function.
        let nfunc = yul_map_default::map_yul_func_def(self, func);

        // Restore the renaming environment
        self.env = stored_env;

        // Return result.
        nfunc
    }

    /// Override `map_yul_var_decl`.
    fn map_yul_var_decl(&mut self, vdecl: &YulVarDecl) -> YulVarDecl {
        let nidents = self.create_new_idents(&vdecl.vars);
        let nvalue = vdecl
            .value
            .as_ref()
            .map(|expr| yul_map_default::map_yul_expr(self, expr));
        YulVarDecl::new(nidents, nvalue)
    }

    /// Override `map_yul_member_expr`.
    fn map_yul_member_expr(&mut self, expr: &YulMemberExpr) -> YulMemberExpr {
        // Only rename the base of the member access expression, since the member name
        // are Yul keywords
        let nbase = self.map_yul_name(&expr.base);
        YulMemberExpr { base: nbase, ..expr.clone() }
    }

    /// Override `map_yul_name`.
    fn map_yul_name(&mut self, name: &Name) -> Name {
        let idx = self.env.get_current_index(&name.base);
        let mut nname = name.clone();
        nname.set_index(idx);
        nname
    }
}
