//! Module for renaming variables.

use crate::{ast::*, util::*};

/// Function to rename variables.
pub fn rename_variables(source_unit: &SourceUnit) -> SourceUnit {
    info!("Renaming variables...");
    let mut renamer = Renamer::new(None);
    renamer.rename_source_unit(source_unit)
}

/// Function to rename variables.
pub fn rename_variables_in_block(block: &Block, env: NamingEnv) -> Block {
    debug!("Renaming variables...");
    let mut renamer = Renamer::new(Some(env));
    renamer.rename_block(block)
}

//-------------------------------------------------
// Rename variables.
//-------------------------------------------------

/// Data structure to rename variables.
///
/// This data structure should be kept private.
pub(super) struct Renamer {
    env: NamingEnv,
}

impl Renamer {
    pub fn new(env: Option<NamingEnv>) -> Self {
        let env = match env {
            Some(env) => env,
            None => NamingEnv::new(),
        };
        Renamer { env }
    }

    pub fn create_new_idents(&mut self, idents: &[Identifier]) -> Vec<Identifier> {
        idents
            .iter()
            .map(|ident| {
                let name = &ident.name;
                let (nname, nenv) = self.env.create_new_name(&name.base);
                self.env = nenv;
                Identifier { name: nname, ..ident.clone() }
            })
            .collect()
    }

    pub fn rename_source_unit(&mut self, source_unit: &SourceUnit) -> SourceUnit {
        self.map_source_unit(source_unit)
    }

    pub fn rename_block(&mut self, block: &Block) -> Block {
        self.map_block(block)
    }
}

impl Map for Renamer {
    fn map_func_def(&mut self, func: &FuncDef) -> FuncDef {
        // Save the current renaming environment
        let stored_env = self.env.clone();

        // Transform the function.
        let nfunc = map::default::map_func_def(self, func);

        // Restore the renaming environment
        self.env = stored_env;

        // Return result.
        nfunc
    }

    /// Override `map_var_decl`.
    fn map_var_decl(&mut self, vdecl: &VarDecl) -> VarDecl {
        let nidents = self.create_new_idents(&vdecl.vars);
        let nvalue = vdecl
            .value
            .as_ref()
            .map(|expr| map::default::map_expr(self, expr));
        VarDecl::new(nidents, nvalue)
    }

    /// Override `map_member_access`.
    fn map_member_expr(&mut self, expr: &MemberExpr) -> MemberExpr {
        // Only rename the base of the member access expression, since the member name
        // are Yul keywords
        let nbase = self.map_name(&expr.base);
        MemberExpr { base: nbase, ..expr.clone() }
    }

    /// Override `map_name`.
    fn map_name(&mut self, name: &Name) -> Name {
        let idx = self.env.get_current_index(&name.base);
        let mut nname = name.clone();
        nname.set_index(idx);
        nname
    }
}
