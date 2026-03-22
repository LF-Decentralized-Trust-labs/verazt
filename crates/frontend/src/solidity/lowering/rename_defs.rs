//! Module to rename overloaded function definitions.
//!
//! Only functions that share the same base name (i.e., are overloaded) get
//! numeric suffixes. Non-overloaded functions keep their original names.
//!
//! The renaming is necessary to eliminate the `import` directives later.

use crate::solidity::ast::NamingEnv;
use crate::solidity::{ast::utils::*, ast::*};
use std::collections::HashMap;

//-------------------------------------------------
// Pass 1: Count function name occurrences
//-------------------------------------------------

/// Count how many times each function base name appears across all source units.
fn count_func_names(source_units: &[SourceUnit]) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for su in source_units {
        for elem in &su.elems {
            match elem {
                SourceUnitElem::Contract(c) => {
                    for member in &c.body {
                        if let ContractElem::Func(f) = member {
                            *counts.entry(f.name.base.clone()).or_insert(0) += 1;
                        }
                    }
                }
                SourceUnitElem::Func(f) => {
                    *counts.entry(f.name.base.clone()).or_insert(0) += 1;
                }
                _ => {}
            }
        }
    }
    counts
}

//-------------------------------------------------
// Pass 2: Rename only overloaded definitions
//-------------------------------------------------

#[derive(Debug, Clone)]
struct Renamer {
    env: NamingEnv,
    /// Set of function base names that are overloaded (appear more than once).
    overloaded_names: HashMap<String, usize>,
    /// Name of the contract currently being processed (for naming constructors).
    current_contract_name: Option<String>,
}

impl Renamer {
    pub fn new(env: Option<&NamingEnv>, overloaded_names: HashMap<String, usize>) -> Self {
        let env = match env {
            Some(env) => env.clone(),
            None => NamingEnv::new(),
        };
        Renamer { env, overloaded_names, current_contract_name: None }
    }

    pub fn rename_definitions(&mut self, source_units: &[SourceUnit]) -> Vec<SourceUnit> {
        self.map_source_units(source_units)
    }
}

/// Implement `Map` utility to rename only overloaded functions.
impl Map<'_> for Renamer {
    /// Override `map_source_unit` to capture the current renaming scope.
    fn map_source_unit(&mut self, source_unit: &SourceUnit) -> SourceUnit {
        map::default::map_source_unit(self, source_unit)
    }

    /// Override `map_contract_def` to track the current contract name.
    fn map_contract_def(&mut self, contract: &ContractDef) -> ContractDef {
        let saved = self.current_contract_name.take();
        self.current_contract_name = Some(contract.name.base.clone());
        let result = map::default::map_contract_def(self, contract);
        self.current_contract_name = saved;
        result
    }

    /// Override `map_func_def` to rename function names.
    fn map_func_def(&mut self, func: &FuncDef) -> FuncDef {
        // Give constructors the contract name if they don't have one.
        let is_constructor = func.kind == FuncKind::Constructor && func.name.is_empty();
        if is_constructor {
            let mut nfunc = map::default::map_func_def(self, func);
            if let Some(contract_name) = &self.current_contract_name {
                nfunc.name = Name::new(contract_name.clone(), None);
            }
            return nfunc;
        }

        // Only rename functions whose base name is overloaded.
        let is_overloaded = self
            .overloaded_names
            .get(&func.name.base)
            .map_or(false, |&count| count > 1);

        if !is_overloaded {
            // Non-overloaded: keep original name, still need to register
            // in the env so rename_callees can find it.
            let (_, nenv) = self.env.create_new_name(&func.name.base);
            self.env = nenv;
            return map::default::map_func_def(self, func);
        }

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

/// Rename overloaded function definitions. Non-overloaded functions keep
/// their original names; only functions with duplicate base names get
/// numeric suffixes.
pub fn rename_defs(
    source_units: &[SourceUnit],
    env: Option<&NamingEnv>,
) -> (Vec<SourceUnit>, NamingEnv) {
    // Pass 1: count function name occurrences
    let counts = count_func_names(source_units);

    // Pass 2: rename only overloaded names
    let mut renamer = Renamer::new(env, counts);
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

                function z(uint x) public returns (uint) {
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
