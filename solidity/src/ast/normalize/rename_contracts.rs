//! Module to rename contracts.
//!
//! The renaming will be done on both contract definitions and base contract
//! information of inheritance.

use crate::{ast::utils::*, ast::*};
use crate::ast::NamingEnv;

#[derive(Debug, Clone)]
struct Renamer<'a> {
    pub current_source_unit: Option<&'a SourceUnit>,
    source_units: &'a [SourceUnit],
    env: NamingEnv,
    phase: Option<RenamingPhase>,
}

impl<'a> Renamer<'a> {
    pub fn new(env: Option<&NamingEnv>) -> Self {
        let env = match env {
            Some(env) => env.clone(),
            None => NamingEnv::new(),
        };
        Renamer { current_source_unit: None, source_units: &[], env, phase: None }
    }
}

/// Implement `Map` utility to rename callees.
impl<'a> Map<'_> for Renamer<'a> {
    /// Override `map_source_unit` to capture the current renaming scope.
    fn map_source_unit(&mut self, source_unit: &SourceUnit) -> SourceUnit {
        self.current_source_unit = self
            .source_units
            .iter()
            .find(|unit| unit.path == source_unit.path);

        // Reset contract scope
        let nsource_unit = map::default::map_source_unit(self, source_unit);

        // Clear the source unit scope and return result.
        self.current_source_unit = None;
        nsource_unit
    }

    /// Override `map_contract_def` to rename contract names.
    fn map_contract_def(&mut self, contract: &ContractDef) -> ContractDef {
        if matches!(&self.phase, Some(RenamingPhase::RenameDefinitions)) {
            // First phase: rename the contract name with a new indexing number.
            let (nname, nenv) = self.env.create_new_name(&contract.name.base);
            self.env = nenv;
            if let Some(idx) = nname.index {
                self.env
                    .current_naming_index_map
                    .insert(nname.base.clone(), idx);
            }

            // Transform the contract definition and update its new name.
            let mut ncontract = map::default::map_contract_def(self, contract);
            ncontract.name = nname;

            // Return result
            ncontract
        } else if matches!(&self.phase, Some(RenamingPhase::RenameUsedContractNames)) {
            // Second phase: rename the contract where it is used.
            map::default::map_contract_def(self, contract)
        } else {
            // Unknonw phase: do nothing
            contract.clone()
        }
    }

    /// Override `map_base_contract` to update indexing of the base name.
    fn map_base_contract(&mut self, base: &BaseContract) -> BaseContract {
        let nbase = map::default::map_base_contract(self, base);

        // Skip renaming base contracts if this is not the second phase.
        if !matches!(self.phase, Some(RenamingPhase::RenameUsedContractNames)) {
            return nbase;
        }

        // Rename base contracts in the second phase.
        if let Some(source_unit) = self.current_source_unit
            && let Some(contract) = source_unit.find_contract_def_by_base_name(&base.name)
        {
            return BaseContract { name: contract.name.clone(), ..nbase };
        }
        nbase
    }

    /// Override `map_func_def` to rename base contracts that will be overriden.
    fn map_func_def(&mut self, func: &FuncDef) -> FuncDef {
        let nfunc = map::default::map_func_def(self, func);

        // Skip renaming function if this is not the second phase.
        if !matches!(self.phase, Some(RenamingPhase::RenameUsedContractNames)) {
            return nfunc;
        }

        // Rename overrides
        let overriding = match (&self.current_source_unit, &func.overriding) {
            (Some(source_unit), Overriding::Some(names)) => {
                let mut nnames = vec![];
                for name in names.iter() {
                    match source_unit.find_contract_def_by_base_name(name) {
                        Some(contract) => nnames.push(contract.name.clone()),
                        None => nnames.push(name.clone()),
                    }
                }
                Overriding::Some(nnames)
            }
            _ => func.overriding.clone(),
        };

        FuncDef { overriding, ..nfunc }
    }

    /// Override `map_var_decl` to rename base contracts that will be overriden.
    fn map_var_decl(&mut self, vdecl: &VarDecl) -> VarDecl {
        let nvdecl = map::default::map_var_decl(self, vdecl);

        // Skip renaming variable declaration if this is not the second phase.
        if !matches!(self.phase, Some(RenamingPhase::RenameUsedContractNames)) {
            return nvdecl;
        }

        // Rename overrides
        let overriding = match (&self.current_source_unit, &vdecl.overriding) {
            (Some(source_unit), Overriding::Some(names)) => {
                let mut nnames = vec![];
                for name in names.iter() {
                    match source_unit.find_contract_def_by_base_name(name) {
                        Some(contract) => nnames.push(contract.name.clone()),
                        None => nnames.push(name.clone()),
                    }
                }
                Overriding::Some(nnames)
            }
            _ => vdecl.overriding.clone(),
        };

        VarDecl { overriding, ..nvdecl }
    }
}

/// Data structure to represent the renaming phase.
#[derive(Debug, Clone)]
enum RenamingPhase {
    /// Rename contract definitions.
    RenameDefinitions,

    /// Rename base contracts.
    RenameUsedContractNames,
}

//-------------------------------------------------
// Public functions
//-------------------------------------------------

/// Rename contracts in a list of source units.
pub fn rename_contracts(
    source_units: &[SourceUnit],
    env: Option<&NamingEnv>,
) -> (Vec<SourceUnit>, NamingEnv) {
    println!("Normalize AST: renaming base contracts");

    // First, rename names of contract definitions.
    let mut renamer = Renamer::new(env);
    renamer.phase = Some(RenamingPhase::RenameDefinitions);
    renamer.source_units = source_units;
    let output_source_units = renamer.map_source_units(source_units);

    // Then, rename the used names
    renamer.phase = Some(RenamingPhase::RenameUsedContractNames);
    renamer.source_units = &output_source_units;
    let nsource_units = renamer.map_source_units(&output_source_units);

    // Return result
    (nsource_units, renamer.env)
}

//-------------------------------------------------
// Unit tests
//-------------------------------------------------

/// Unit tests
#[cfg(test)]
mod tests {
    use super::rename_contracts;
    use crate::{
        ast::utils::syntactic_comparer::compare_source_units,
        parser::parse_solidity_source_code,
        ast::normalize::{flatten_name, utils::configure_unit_test_env},
    };
    use indoc::indoc;

    /// Test renaming contracts.
    #[test]
    fn test_rename_contracts() {
        let _ = configure_unit_test_env();

        // Input contract
        let input_contract = indoc! {r###"
            contract A {
                function foo() public pure virtual returns (string memory) {
                    return "A";
                }
            }

            contract B is A {
                function foo() public pure virtual override returns (string memory) {
                    return "B";
                }
            }

            contract C is A {
                function foo() public pure virtual override returns (string memory) {
                    return "C";
                }
            }

            contract D is B, C {
                function foo() public pure override(B, C) returns (string memory) {
                    return super.foo();
                }
            }

            contract E is C, B {
                function foo() public pure override(C, B) returns (string memory) {
                    return super.foo();
                }
            }

            contract F is A, B {
                function foo() public pure override(A, B) returns (string memory) {
                    return super.foo();
                }
            }"###};

        // Expected output contract
        let expected_contract = indoc! {r###"
            contract A_0 {
                function foo() public pure virtual returns (string memory) {
                    return "A";
                }
            }

            contract B_0 is A_0 {
                function foo() public pure virtual override returns (string memory) {
                    return "B";
                }
            }

            contract C_0 is A_0 {
                function foo() public pure virtual override returns (string memory) {
                    return "C";
                }
            }

            contract D_0 is B_0, C_0 {
                function foo() public pure override(B_0, C_0) returns (string memory) {
                    return super.foo();
                }
            }

            contract E_0 is C_0, B_0 {
                function foo() public pure override(C_0, B_0) returns (string memory) {
                    return super.foo();
                }
            }

            contract F_0 is A_0, B_0 {
                function foo() public pure override(A_0, B_0) returns (string memory) {
                    return super.foo();
                }
            }"###};

        let input_sunits = match parse_solidity_source_code(input_contract, "0.8.17") {
            Ok(input_sunits) => input_sunits,
            Err(err) => panic!("Failed to parse input source unit: {err}"),
        };
        let (output_sunits, _) = rename_contracts(&input_sunits, None);
        let output_sunits = flatten_name(&output_sunits);

        let expected_sunits = match parse_solidity_source_code(expected_contract, "0.8.17") {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse expected source unit: {err}"),
        };

        if let Err(err) = compare_source_units(&output_sunits, &expected_sunits) {
            panic!("Failed to rename base contracts: {err}")
        }
    }
}
