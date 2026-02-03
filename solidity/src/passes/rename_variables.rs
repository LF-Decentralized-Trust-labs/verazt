//! Module to rename variables. This module performs the following
//! transformations:
//! - Rename shadowed variables.
//! - Rename variables in different source units to have unique names.
//! - Indexing numbers are globally accross all source units.

use crate::{ast::utils::*, ast::*, yul::normalize as yul_normalize};
use crate::ast::yul::YulBlock;
use meta::NamingEnv;

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
}

impl Map<'_> for Renamer {
    fn map_block(&mut self, block: &Block) -> Block {
        // Save the current naming index before transforming the block,
        // which may introduce a new naming scope.
        let current_naming_index = self.env.current_naming_index_map.clone();

        // Transform block..
        let nblock = map::default::map_block(self, block);

        // Restore the current naming index and return result
        self.env.current_naming_index_map = current_naming_index;

        // Return result
        nblock
    }

    /// Override `map_asm_stmt` to transfer the naming environment to
    /// Yul AST.
    fn map_asm_stmt(&mut self, stmt: &AsmStmt) -> AsmStmt {
        let blk = YulBlock::new(stmt.body.clone());
        let nblk = yul_normalize::rename_variables::rename_yul_variables_in_block(&blk, self.env.clone());
        AsmStmt { body: nblk.body, ..stmt.clone() }
    }

    /// Override `map_var_decl`.
    fn map_var_decl(&mut self, vdecl: &VarDecl) -> VarDecl {
        // First, transform the variable declaration normally.
        let nvdecl = map::default::map_var_decl(self, vdecl);

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
pub fn rename_variables(
    source_units: &[SourceUnit],
    env: Option<&NamingEnv>,
) -> (Vec<SourceUnit>, NamingEnv) {
    println!("Normalize AST: renaming variables");
    let mut renamer = Renamer::new(env);
    let nsource_units = renamer.rename_variables(source_units);
    (nsource_units, renamer.env)
}

//-------------------------------------------------
// Unit tests
//-------------------------------------------------

/// Unit tests
#[cfg(test)]
mod tests {
    use super::rename_variables;
    use crate::{
        ast::utils::syntactic_comparer::compare_source_units,
        compile::compile_solidity_source_code, passes::utils::configure_unit_test_env,
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

        let input_sunits = match compile_solidity_source_code(input_contract, "0.8.15") {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse input source unit: {}", err),
        };
        let (output_sunits, _) = rename_variables(&input_sunits, None);

        let expected_sunits = match compile_solidity_source_code(expected_contract, "0.8.15") {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse expected source unit: {}", err),
        };

        if let Err(err) = compare_source_units(&output_sunits, &expected_sunits) {
            panic!("Failed to rename variables: {}", err)
        }
    }
}
