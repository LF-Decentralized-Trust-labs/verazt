//! Module to eliminate function modifier from function definitions.

use crate::{
    ast::{utils::*, *},
    ast::normalize::substitution,
};
use crate::ast::Name;
use std::{collections::HashMap, ops::Deref};

#[derive(Clone)]
struct ModifierDefFinder {}

impl ModifierDefFinder {
    pub fn new() -> Self {
        ModifierDefFinder {}
    }

    pub(super) fn find_modifier_definitions(
        &mut self,
        source_unit: &SourceUnit,
    ) -> HashMap<Name, FuncDef> {
        let mut func_info = HashMap::new();
        self.fold_source_unit(&mut func_info, source_unit).clone()
    }
}

impl Fold<'_, &mut HashMap<Name, FuncDef>> for ModifierDefFinder {
    /// Override `fold_source_unit_elem`.
    fn fold_source_unit_elem<'a>(
        &mut self,
        acc: &'a mut HashMap<Name, FuncDef>,
        elem: &SourceUnitElem,
    ) -> &'a mut HashMap<Name, FuncDef> {
        match elem {
            SourceUnitElem::Contract(contract) => self.fold_contract_def(acc, contract),
            _ => acc,
        }
    }

    /// Override `fold_contract_elem`.
    fn fold_contract_elem<'a>(
        &mut self,
        acc: &'a mut HashMap<Name, FuncDef>,
        elem: &ContractElem,
    ) -> &'a mut HashMap<Name, FuncDef> {
        match elem {
            ContractElem::Func(func) if func.kind == FuncKind::Modifier => {
                acc.insert(func.name.clone(), func.clone());
            }
            _ => {}
        }
        acc
    }
}

fn find_modifier_declarations(source_unit: &SourceUnit) -> HashMap<Name, FuncDef> {
    let mut finder = ModifierDefFinder::new();
    finder.find_modifier_definitions(source_unit)
}

//-------------------------------------------------
// Utilities function
//-------------------------------------------------

fn create_ident_from_var_decl(var: &VarDecl) -> Identifier {
    Identifier::new(var.id, var.name.clone(), var.typ.clone(), var.loc)
}

//----------------------------------------------------------------------
// Replacing modifier invocation with the body of modifier declaration.
//----------------------------------------------------------------------

/// Data structure for replacing a modifier invocation with statements in
/// the body of the modifier declaration.
struct ModifierTransformer {
    modifier_def_map: HashMap<Name, FuncDef>,
}

impl ModifierTransformer {
    pub fn new() -> Self {
        ModifierTransformer { modifier_def_map: HashMap::new() }
    }

    pub fn remove_modifier_invocations(&mut self, source: &SourceUnit) -> SourceUnit {
        self.modifier_def_map = find_modifier_declarations(source);
        map::default::map_source_unit(self, source)
    }
}

impl Map<'_> for ModifierTransformer {
    /// Override `map_contract_def`
    fn map_contract_def(&mut self, contract: &ContractDef) -> ContractDef {
        let nbody = contract
            .body
            .iter()
            .filter(|elem| match elem {
                ContractElem::Func(func) => func.kind != FuncKind::Modifier,
                _ => true,
            })
            .map(|elem| map::default::map_contract_element(self, elem))
            .collect();

        ContractDef { body: nbody, ..contract.clone() }
    }

    /// Override `map_func_def`
    fn map_func_def(&mut self, func: &FuncDef) -> FuncDef {
        let mut func = func.clone();

        let mut base_constructor_modifier_invocs = vec![];

        for minvoc in func.modifier_invocs.clone() {
            if minvoc.kind == CallKind::BaseConstructorCall {
                base_constructor_modifier_invocs.push(minvoc.clone());
                continue;
            }

            let mut statements: Vec<Stmt> = vec![];
            let func_body = match &func.body {
                Some(block) => block.body.to_vec(),
                None => vec![],
            };

            let modifier_name = match minvoc.callee.deref() {
                Expr::Ident(ident) => ident.name.clone(),
                expr => panic!("Invalid modifier name: {}", expr),
            };

            match self.modifier_def_map.get(&modifier_name) {
                Some(modifier) => {
                    let args: Vec<Expr> = match minvoc.args {
                        CallArgs::Unnamed(args) => args,
                        CallArgs::Named(_) => {
                            panic!("Invalid modifier args: {}", minvoc.args)
                        }
                    };
                    let params: Vec<Identifier> = modifier
                        .params
                        .iter()
                        .map(create_ident_from_var_decl)
                        .collect();

                    let modifier_body = match &modifier.body {
                        None => vec![],
                        Some(blk) => blk.body.clone(),
                    };

                    for stmt in modifier_body {
                        match stmt {
                            Stmt::Placeholder(_) => {
                                for stmt in &func_body {
                                    statements.push(stmt.clone());
                                }
                            }
                            _ => {
                                let nstmt = substitution::substitute_stmt(
                                    params.clone(),
                                    args.clone(),
                                    &stmt,
                                );
                                statements.push(nstmt);
                            }
                        }
                    }
                }
                None => panic!("Modifier {} not found", minvoc.callee.deref()),
            }

            func.body = func
                .body
                .clone()
                .map(|blk| Block { body: statements, ..blk });
        }

        // Keep only modifier invocations of base constructor calls.
        func.modifier_invocs = base_constructor_modifier_invocs;

        // Return result
        func
    }
}

/// Function to remove modifier invocations from function definitions in a list
/// of source units.
pub fn eliminate_modifier_invocs(source_units: &[SourceUnit]) -> Vec<SourceUnit> {
    let mut nsource_units = vec![];
    for sunit in source_units.iter() {
        let mut transformer = ModifierTransformer::new();
        nsource_units.push(transformer.remove_modifier_invocations(sunit));
    }
    nsource_units
}

//-------------------------------------------------
// Unit tests
//-------------------------------------------------

#[cfg(test)]
mod tests {
    use super::eliminate_modifier_invocs;
    use crate::{
        ast::utils::syntactic_comparer::compare_source_units,
        compile::compile_solidity_source_code, ast::normalize::utils::configure_unit_test_env,
    };
    use indoc::indoc;

    #[test]
    fn remove_modifiers() {
        let _ = configure_unit_test_env();

        // Input contract
        let input_contract = indoc! {r###"
            contract FunctionModifier {
                address public owner;
                uint public x = 10;
                bool public locked;

                constructor() {
                    owner = msg.sender;
                }

                modifier onlyOwner() {
                    require(msg.sender == owner, "Not owner");
                    _;
                }

                modifier validAddress(address _addr) {
                    require(_addr != address(0), "Not valid address");
                    _;
                }

                function changeOwner(address _newOwner) public onlyOwner validAddress(_newOwner) {
                    owner = _newOwner;
                }

                modifier noReentrancy() {
                    require(!locked, "No reentrancy");

                    locked = true;
                    _;
                    locked = false;
                }

                function decrement(uint i) public noReentrancy {
                    x -= i;

                    if (i > 1) {
                        decrement(i - 1);
                    }
                }
            }"###};

        // Expected output contract
        let expected_contract = indoc! {r###"
            contract FunctionModifier {
                address public owner;
                uint256 public x = 10;
                bool public locked;

                constructor() public {
                    owner = msg.sender;
                }

                function changeOwner(address _newOwner) public {
                require(_newOwner != address(0), "Not valid address");
                require(msg.sender == owner, "Not owner");
                owner = _newOwner;
                }

                function decrement(uint256 i) public {
                    require(!locked, "No reentrancy");
                    locked = true;
                    x -= i;
                    if (i > 1) {
                        decrement(i - 1);
                    }
                    locked = false;
                }
            }"###};

        let input_sunits = match compile_solidity_source_code(input_contract, "0.8.19") {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse input source unit: {}", err),
        };
        let output_sunits = eliminate_modifier_invocs(&input_sunits);

        let expected_sunits = match compile_solidity_source_code(expected_contract, "0.8.19") {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse expected source unit: {}", err),
        };

        if let Err(err) = compare_source_units(&output_sunits, &expected_sunits) {
            panic!("Failed to eliminiate modifiers: {}", err)
        }
    }
}
