//! Module to rename variables. This module performs the following
//! transformations:
//! - Rename shadowed variables.
//! - Rename variables in different source units to have unique names.
//! - Indexing numbers are globally accross all source units.

use crate::ast::NamingEnv;
use crate::ast::utils::{self, Map};
use crate::ast::yul::utils::{YulMap, yul_map_default};
use crate::ast::yul::{
    YulBlock, YulFuncDef, YulIdentifier, YulMemberExpr, YulSourceUnit, YulVarDecl,
};
use crate::ast::*;

/// Function to rename variables in a YulSourceUnit.
pub fn rename_yul_variables(source_unit: &YulSourceUnit) -> YulSourceUnit {
    let mut renamer = Renamer::new(None);
    renamer.map_yul_source_unit(source_unit)
}

/// Function to rename variables in a YulBlock.
pub fn rename_yul_variables_in_block(block: &YulBlock, env: NamingEnv) -> YulBlock {
    let mut renamer = Renamer::new(Some(&env));
    renamer.map_yul_block(block)
}

//-------------------------------------------------
// Rename variables.
//-------------------------------------------------

/// Data structure for renaming variables, function definitions, modifier
/// definitions, error definitions, event definitions, struct definitions, and
/// enum definitions.
///
/// This data structure should be kept private.
struct Renamer {
    env: NamingEnv,
}

impl Renamer {
    pub fn new(env: Option<&NamingEnv>) -> Self {
        let env = match env {
            Some(env) => env.clone(),
            None => NamingEnv::new(),
        };
        Renamer { env }
    }

    fn rename_variables(&mut self, source_units: &[SourceUnit]) -> Vec<SourceUnit> {
        // Reset environment
        self.env = NamingEnv::new();
        self.map_source_units(source_units)
    }

    fn create_new_idents(&mut self, idents: &[YulIdentifier]) -> Vec<YulIdentifier> {
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
}

impl Map<'_> for Renamer {
    fn map_block(&mut self, block: &Block) -> Block {
        // Save the current naming index before transforming the block,
        // which may introduce a new naming scope.
        let current_naming_index = self.env.current_naming_index_map.clone();

        // Transform block..
        let nblock = utils::map::default::map_block(self, block);

        // Restore the current naming index and return result
        self.env.current_naming_index_map = current_naming_index;

        // Return result
        nblock
    }

    /// Override `map_asm_stmt` to transfer the naming environment to
    /// Yul AST.
    fn map_asm_stmt(&mut self, stmt: &AsmStmt) -> AsmStmt {
        let blk = YulBlock::new(stmt.body.clone());
        let nblk = rename_yul_variables_in_block(&blk, self.env.clone());
        AsmStmt { body: nblk.body, ..stmt.clone() }
    }

    /// Override `map_var_decl`.
    fn map_var_decl(&mut self, vdecl: &VarDecl) -> VarDecl {
        // First, transform the variable declaration normally.
        let nvdecl = utils::map::default::map_var_decl(self, vdecl);

        // Then, create a new index for the variable name.
        let base_name = &nvdecl.name.base;
        if base_name.is_empty() {
            return nvdecl;
        }
        let (nname, nenv) = self.env.create_new_name(base_name);
        self.env = nenv;

        VarDecl { name: nname, ..nvdecl.clone() }
    }

    /// Override `map_identifier`.
    fn map_ident(&mut self, ident: &Identifier) -> Identifier {
        // Get the index of the variable declaration in the current scope.
        let index = self.env.get_current_index(&ident.name.base);

        // Update and return the new identifier.
        let mut nident = ident.clone();
        nident.set_naming_index(index);
        nident
    }
}

/// Function to rename ariables.
pub fn rename_vars(
    source_units: &[SourceUnit],
    env: Option<&NamingEnv>,
) -> (Vec<SourceUnit>, NamingEnv) {
    println!("Normalize AST: renaming variables");
    let mut renamer = Renamer::new(env);
    let nsource_units = renamer.rename_variables(source_units);
    (nsource_units, renamer.env)
}

impl YulMap for Renamer {
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

//-------------------------------------------------
// Unit tests
//-------------------------------------------------

/// Unit tests
#[cfg(test)]
mod tests {
    use super::rename_vars;
    use crate::{
        ast::normalize::utils::configure_unit_test_env,
        ast::utils::syntactic_comparer::compare_source_units, parser::parse_solidity_source_code,
    };
    use indoc::indoc;

    // Test normalization in a single contract.
    #[test]
    fn test_renaming_variable_in_contract() {
        let _ = configure_unit_test_env();

        // Input contract
        let input_contract = indoc! {r###"
            contract C {
                uint256 storedData; // State variable
                uint256 storedValue = 2; // State variable
                function getResult() public view returns (uint256) {
                    uint256 a = 1; // local variable
                    uint256 b = 2;
                    uint256 result = a + b;
                    {
                        uint256 a = 2;
                        uint256 b = 3;
                        uint256 result = a + b;
                        result = result + a + b;
                    }
                    if (a > b) {
                        result++;
                        --result;
                    } else {
                        uint256 result = result + b;
                        result += 2;
                    }
                    result = result + 2;
                    return storedData; //access the state variable
                }
            }"###};

        // Expected output contract
        let expected_contract = indoc! {r###"
            contract C {
                uint256 storedData_0;
                uint256 storedValue_0 = 2;
                function getResult() public view returns (uint256) {
                    uint256 a_0 = 1;
                    uint256 b_0 = 2;
                    uint256 result_0 = a_0 + b_0;
                    {
                        uint256 a_1 = 2;
                        uint256 b_1 = 3;
                        uint256 result_1 = a_1 + b_1;
                        result_1 = result_1 + a_1 + b_1;
                    }
                    if (a_0 > b_0) {
                        result_0++;
                        --result_0;
                    } else {
                        uint256 result_2 = result_0 + b_0;
                        result_2 += 2;
                    }
                    result_0 = result_0 + 2;
                    return storedData_0;
                }
            }"###};

        let input_sunits = match parse_solidity_source_code(input_contract, "0.8.15") {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse input source unit: {}", err),
        };
        let (output_sunits, _) = rename_vars(&input_sunits, None);

        let expected_sunits = match parse_solidity_source_code(expected_contract, "0.8.15") {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse expected source unit: {}", err),
        };

        if let Err(err) = compare_source_units(&output_sunits, &expected_sunits) {
            panic!("Failed to rename variables: {}", err)
        }
    }
}
