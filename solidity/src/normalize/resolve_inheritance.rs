//! Module to flatten inheritance in Solidity smart contracts.
//!
//! This module will perform naming resolution for all inheritance and
//! interfaces.

use meta::Name;
use crate::{ast::*, util::*};
use std::collections::HashMap;

//-------------------------------------------------
// Finding and linearizing inheritance information
//-------------------------------------------------

/// Data structure for finding and linearizing inheritance information in
/// Solidity source units.
struct InheritanceLinearizer {
    /// Mapping each derived contract to its base contracts with the order as
    /// specified in the source unit.
    inheritance_map: HashMap<Name, Vec<Name>>,
}

impl InheritanceLinearizer {
    fn new() -> Self {
        Self { inheritance_map: HashMap::new() }
    }

    pub fn find_inheritance(&mut self, source_unit: &SourceUnit) -> HashMap<Name, Vec<Name>> {
        // First, run the visitor pattern to update all inheritance information.
        self.visit_source_unit(source_unit);

        // Then, update the non-inherited contracts.
        for source_unit_elem in source_unit.elems.iter() {
            if let SourceUnitElem::Contract(contract) = source_unit_elem {
                self.inheritance_map
                    .entry(contract.name.clone())
                    .or_default();
            }
        }

        // Return result
        self.inheritance_map.clone()
    }

    /// Merge base contract list based on C3 linearization algorithm.
    ///
    /// C3 algorithm: <https://en.wikipedia.org/wiki/C3_linearization>
    fn merge(&self, contracts_lists: Vec<Vec<Name>>) -> Vec<Name> {
        let mut working_contract_lists = contracts_lists.clone();
        let mut merged_contracts = vec![];

        while !working_contract_lists.is_empty() {
            let mut ncontract_lists = vec![];

            let mut contract_to_be_merged = None;

            // Find contract name to be merged.
            for contract_list in working_contract_lists.iter() {
                if let Some(contract) = contract_list.first() {
                    let can_be_merged = working_contract_lists.iter().all(|contracts| {
                        match contracts.iter().position(|c| c.eq(contract)) {
                            Some(idx) => idx == 0,
                            None => true,
                        }
                    });
                    if can_be_merged {
                        contract_to_be_merged = Some(contract);
                        break;
                    }
                }
            }

            // Merge the candidate contract and update the new work list.
            if let Some(contract) = contract_to_be_merged {
                for contract_list in working_contract_lists.iter() {
                    match contract_list.split_first() {
                        Some((c, others)) if c.eq(contract) => {
                            if !merged_contracts.contains(contract) {
                                merged_contracts.push(contract.clone());
                            }
                            if !others.is_empty() {
                                ncontract_lists.push(others.to_vec());
                            }
                        }
                        _ => ncontract_lists.push(contract_list.clone()),
                    }
                }
                working_contract_lists = ncontract_lists
            } else {
                panic!(
                    "Linearizing inheritance: unable to merge contract list: {}",
                    contracts_lists
                        .iter()
                        .map(|contracts| contracts.iter().map(|c| c.to_string()).collect())
                        .collect::<Vec<String>>()
                        .join("; ")
                )
            }
        }

        merged_contracts
    }

    /// Linearize inheritance map.
    ///
    /// Solidity use C3 linearization (a.k.a., MRO: Method Resolution Order)
    /// algorithm, similar to Python.
    ///
    /// Input `base_contracts_map`: mapping each derived contract to its base
    /// contracts.
    pub fn linearize_inheritance(
        &self,
        inheritance_map: &HashMap<Name, Vec<Name>>,
    ) -> HashMap<Name, Vec<Name>> {
        let mut linearization_map: HashMap<Name, Vec<Name>> = HashMap::new();

        // Initialize non-inherited contracts to the linearization map
        for (derived_contract, base_contracts) in inheritance_map {
            if base_contracts.is_empty() {
                linearization_map.insert(derived_contract.clone(), vec![derived_contract.clone()]);
            }
        }

        let mut finished = false;
        while !finished {
            finished = true;
            for (derived_contract, base_contracts) in inheritance_map.iter() {
                if linearization_map.contains_key(derived_contract) {
                    continue;
                }

                // Reverse base contracts to follow the order of C3 algo.
                let mut reversed_base_contracts = base_contracts.clone();
                reversed_base_contracts.reverse();

                let mut linearized_base_contracts: Vec<Vec<Name>> = vec![];
                let mut can_merge_based_contracts = true;
                linearized_base_contracts.push(vec![derived_contract.clone()]);
                for base in reversed_base_contracts.iter() {
                    if let Some(contracts) = linearization_map.get(base) {
                        linearized_base_contracts.push(contracts.clone())
                    } else {
                        can_merge_based_contracts = false;
                    }
                }

                if can_merge_based_contracts {
                    let linearized_bases = self.merge(linearized_base_contracts);
                    linearization_map.insert(derived_contract.clone(), linearized_bases);
                    finished = false;
                }
            }
        }

        linearization_map
    }
}

impl Visit<'_> for InheritanceLinearizer {
    fn visit_contract_def(&mut self, contract: &crate::ast::ContractDef) {
        let base_contract_names = contract
            .base_contracts
            .iter()
            .map(|c| c.name.clone())
            .collect();
        self.inheritance_map
            .insert(contract.name.clone(), base_contract_names);
    }
}

//-------------------------------------------------
// Resolve inheritance keywords, such as `super`.
//-------------------------------------------------

struct InheritanceResolver<'a> {
    source_unit: &'a SourceUnit,
    contract_map: HashMap<Name, &'a ContractDef>,
    _inheritance_map: &'a HashMap<Name, Vec<Name>>,
    linearization_map: &'a HashMap<Name, Vec<Name>>,
    current_contract: Option<ContractDef>,
    current_function: Option<FuncDef>,
}

impl<'a> InheritanceResolver<'a> {
    pub fn new(
        source_unit: &'a SourceUnit,
        inheritance_map: &'a HashMap<Name, Vec<Name>>,
        linearization_map: &'a HashMap<Name, Vec<Name>>,
    ) -> Self {
        Self {
            source_unit,
            contract_map: source_unit.construct_contract_map(),
            _inheritance_map: inheritance_map,
            linearization_map,
            current_contract: None,
            current_function: None,
        }
    }

    pub fn elminate_contract_calls(&mut self) -> SourceUnit {
        self.map_source_unit(self.source_unit)
    }
}

impl<'a> Map<'_> for InheritanceResolver<'a> {
    /// Override `map_contract_def` to capture the current contract
    /// definition being processed.
    fn map_contract_def(&mut self, contract: &ContractDef) -> ContractDef {
        self.current_contract = Some(contract.clone());
        let ncontract = map::default::map_contract_def(self, contract);

        let nelems = ncontract.body.iter().filter_map(|elem| match elem {
            ContractElem::Func(func) => {
                // Remove all virtual functions which don't have a body
                if func.is_virtual && func.body.is_none() {
                    None
                } else {
                    // Reset the `virtual` and `override` properties of the function, since all
                    // functions now have unique names
                    let nfunc = FuncDef {
                        id: None,
                        is_virtual: false,
                        overriding: Overriding::None,
                        ..func.clone()
                    };
                    Some(ContractElem::Func(nfunc))
                }
            }
            _ => Some(elem.clone()),
        });

        // Unset the `abstract` property of the contract.
        let ncontract = ContractDef { body: nelems.collect(), ..ncontract };
        self.current_contract = None;
        ncontract
    }

    /// Override `map_func_def` to capture the current function
    /// definition being processed.
    fn map_func_def(&mut self, func: &FuncDef) -> FuncDef {
        // Keep track of the current function definition being processed.
        self.current_function = Some(func.clone());
        let nfunc = map::default::map_func_def(self, func);
        self.current_function = None;
        nfunc
    }

    /// Override `map_member_expr` to resolve `super` keywords in
    /// `MemberAccessExpr`.
    fn map_member_expr(&mut self, expr: &MemberExpr) -> MemberExpr {
        let base = &expr.base;

        // Skip resolve if the base expression is not `super`.
        if !base.to_string().eq(keywords::SUPER) {
            return map::default::map_member_expr(self, expr);
        }

        // Find current contract.
        let current_contract_name = match &self.current_contract {
            None => return map::default::map_member_expr(self, expr),
            Some(contract) => contract.name.clone(),
        };

        // Find if the current function specify an order of overriding base contracts.
        let mut specified_base_contracts = self
            .current_function
            .as_ref()
            .map(|func| match &func.overriding {
                Overriding::Some(overrides) => overrides.to_vec(),
                Overriding::None => vec![],
                Overriding::All => panic!("TODO: implement!"),
            })
            .unwrap_or_default();

        // Find base contracts in linearization order.
        let base_contract_names = if specified_base_contracts.is_empty() {
            match self.linearization_map.get(&current_contract_name) {
                None => return map::default::map_member_expr(self, expr),
                Some(base_contract_names) => base_contract_names,
            }
        } else {
            // Sort the specified base contracts in linearization order
            specified_base_contracts.reverse();
            &specified_base_contracts
        };

        // Start to resolve the keyword `super` referring to a base contracts.
        let member = expr.member.clone();
        for base_contract_name in base_contract_names {
            if let Some(base_contract) = self.contract_map.get(base_contract_name) {
                if base_contract.name == current_contract_name {
                    continue;
                }

                for base_contract_elem in base_contract.body.iter() {
                    let nmember = match base_contract_elem {
                        ContractElem::Var(v) if v.name.base == member.base => Some(&v.name),
                        ContractElem::Func(f) if f.name.base == member.base => Some(&f.name),
                        _ => None,
                    };

                    if let Some(nmember) = nmember {
                        let nbase: Expr = Identifier::new(
                            None,
                            base_contract.name.clone(),
                            base.typ(),
                            base.loc(),
                        )
                        .into();

                        return MemberExpr {
                            base: Box::new(nbase),
                            member: nmember.clone(),
                            ..expr.clone()
                        };
                    }
                }
            }
        }

        // Return default result
        map::default::map_member_expr(self, expr)
    }
}

//-------------------------------------------------
// Flatten inheritance
//-------------------------------------------------

/// Data structure for eliminating base contracts.
struct InheritanceFlattener<'a> {
    /// Source unit.
    source_unit: &'a SourceUnit,

    /// Mapping containing the original inheritance information of contracts, as
    /// written in the source code.
    inheritance_map: HashMap<Name, Vec<Name>>,

    /// Mapping containing the linearized inheritance information of contracts,
    /// which is obtained after a linearization transformation.
    linearization_map: HashMap<Name, Vec<Name>>,
}

impl<'a> InheritanceFlattener<'a> {
    fn new(source_unit: &'a SourceUnit) -> Self {
        let mut inheritance_finder = InheritanceLinearizer::new();
        let inheritance_map = inheritance_finder.find_inheritance(source_unit);
        let linearization_map = inheritance_finder.linearize_inheritance(&inheritance_map);
        Self { source_unit, inheritance_map, linearization_map }
    }

    pub fn flatten_inheritance(&self) -> SourceUnit {
        let mut nsource_unit = self.source_unit.clone();

        let mut contract_resolver = InheritanceResolver::new(
            &nsource_unit,
            &self.inheritance_map,
            &self.linearization_map,
        );
        nsource_unit = contract_resolver.elminate_contract_calls();

        nsource_unit
    }
}

pub fn resolve_inheritance(source_units: &[SourceUnit]) -> Vec<SourceUnit> {
    println!("Normalize AST: resolve inheritance");
    let mut nsource_units = vec![];
    for sunit in source_units.iter() {
        let flattener = InheritanceFlattener::new(sunit);
        let nsource_unit = flattener.flatten_inheritance();
        nsource_units.push(nsource_unit)
    }
    nsource_units
}

//-------------------------------------------------
// Unit tests
//-------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        compile::compile_solidity_source_code,
        normalize::{
            flatten_name, rename_callees, rename_contracts, rename_definitions,
            util::configure_unit_test_env,
        },
        util::syntactic_comparer::compare_source_units,
    };
    use indoc::indoc;

    #[test]
    fn test_merge() {
        let finder = InheritanceLinearizer::new();
        let result = finder.merge(vec![
            vec!["C".into(), "O".into()],
            vec!["B".into(), "O".into()],
            vec!["A".into(), "O".into()],
            vec!["C".into(), "A".into(), "B".into()],
        ]);
        assert_eq!(result, vec!["C".into(), "A".into(), "B".into(), "O".into()]);
    }

    #[test]
    fn test_resolve_inheritance() {
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
                function foo_0() public pure returns (string memory) {
                    return "A";
                }
            }

            contract B_0 is A_0 {
                function foo_1() public pure returns (string memory) {
                    return "B";
                }
            }

            contract C_0 is A_0 {
                function foo_2() public pure returns (string memory) {
                    return "C";
                }
            }

            contract D_0 is B_0, C_0 {
                function foo_3() public pure returns (string memory) {
                    return C_0.foo_2();
                }
            }

            contract E_0 is C_0, B_0 {
                function foo_4() public pure returns (string memory) {
                    return B_0.foo_1();
                }
            }

            contract F_0 is A_0, B_0 {
                function foo_5() public pure returns (string memory) {
                    return B_0.foo_1();
                }
            }"###};

        let input_sunits = match compile_solidity_source_code(input_contract, "0.8.17") {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse input source unit: {err}"),
        };
        let (output_sunits, env) = rename_contracts(&input_sunits, None);
        let (output_sunits, env) = rename_definitions(&output_sunits, Some(&env));
        let (output_sunits, _) = rename_callees(&output_sunits, Some(&env));
        let output_sunits = resolve_inheritance(&output_sunits);
        let output_sunits = flatten_name(&output_sunits);

        let expected_sunits = match compile_solidity_source_code(expected_contract, "0.8.17") {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse expected source unit: {err}"),
        };

        if let Err(err) = compare_source_units(&output_sunits, &expected_sunits) {
            panic!("Failed to resolve inheritance: {err}")
        }
    }
}
