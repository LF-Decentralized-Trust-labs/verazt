//! Module to all definitions, such as contract definitions, function
//! definitions, modifier definitions, event definitions, error definitions,
//! enum definitions, struct definitions, etc.
//!
//! The renaming is necessary to eliminate the `import` directives later.

use crate::solidity::ast::NamingEnv;

use crate::solidity::{ast::utils::*, ast::*};

//-------------------------------------------------
// Rename definitions
//-------------------------------------------------

#[derive(Debug, Clone)]
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

    pub fn rename_definitions(&mut self, source_units: &[SourceUnit]) -> Vec<SourceUnit> {
        // Rename all function definitions and return results.
        self.map_source_units(source_units)
    }
}

/// Implement `Map` utility to rename overloaded functions.
impl Map<'_> for Renamer {
    /// Override `map_source_unit` to capture the current renaming scope.
    fn map_source_unit(&mut self, source_unit: &SourceUnit) -> SourceUnit {
        map::default::map_source_unit(self, source_unit)
    }

    /// Override `map_func_def` to rename function names.
    fn map_func_def(&mut self, func: &FuncDef) -> FuncDef {
        // Save the current naming index before transforming the function definition.
        let current_naming_index = self.env.current_naming_index_map.clone();

        // Rename the function name with a new indexing number.
        let (nname, nenv) = self.env.create_new_name(&func.name.base);
        self.env = nenv;

        // Transform the function definition and update its new name.
        let mut nfunc = map::default::map_func_def(self, func);
        nfunc.name = nname;

        // Restore the current naming index
        self.env.current_naming_index_map = current_naming_index;

        // Return result
        nfunc
    }
}

//-------------------------------------------------
// Public functions
//-------------------------------------------------

/// Rename definitions of contracts, functions, enums, structs, events, errors.
pub fn rename_defs(
    source_units: &[SourceUnit],
    env: Option<&NamingEnv>,
) -> (Vec<SourceUnit>, NamingEnv) {
    let mut renamer = Renamer::new(env);
    let nsource_units = renamer.rename_definitions(source_units);
    (nsource_units, renamer.env)
}

//-------------------------------------------------
// Unit tests
//-------------------------------------------------

/// Unit tests
#[cfg(test)]
mod tests {
    use super::rename_defs;
    use crate::solidity::{
        ast::utils::syntactic_comparer::compare_source_units,
        lowering::utils::configure_unit_test_env, parsing::parse_solidity_source_code,
    };
    use indoc::indoc;

    /// Test renaming definitions.
    ///
    /// Note that this unit test doesn't rename callee functions.
    #[test]
    fn test_rename_definitions() {
        let _ = configure_unit_test_env();

        // Input contract
        let input_contract = indoc! {r###"
            contract C {
                function g() public returns (uint, uint) {
                    return (2, 3);
                }

                function g(uint x) public returns (uint) {
                    return x;
                }

                function z(uint x) public returns (uint) {
                    uint a = 2;
                    return a;
                }
            }

            function f(uint) returns (uint) {
                return 2;
            }

            function f(string memory) returns (uint) {
                return 3;
            }

            function g(bool) returns (uint) {
                return 1;
            }"###};

        // Expected output contract
        let expected_contract = indoc! {r###"
            contract C {
                function g_0() public returns (uint, uint) {
                    return (2, 3);
                }

                function g_1(uint x) public returns (uint) {
                    return x;
                }

                function z_0(uint x) public returns (uint) {
                    uint a = 2;
                    return a;
                }
            }

            function f_0(uint) returns (uint) {
                return 2;
            }

            function f_1(string memory) returns (uint) {
                return 3;
            }

            function g_2(bool) returns (uint) {
                return 1;
            }"###};

        let input_sunits = match parse_solidity_source_code(input_contract, "0.8.15") {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse input source unit: {}", err),
        };
        let (output_sunits, _) = rename_defs(&input_sunits, None);

        let expected_sunits = match parse_solidity_source_code(expected_contract, "0.8.15") {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse expected source unit: {}", err),
        };

        if let Err(err) = compare_source_units(&output_sunits, &expected_sunits) {
            panic!("Failed to flatten definitions: {}", err)
        }
    }
}
