//! Module removing the `using` directives in Solidity smart contracts.
//!
//! Prerequisite: Assuming that all contract and library names are already
//! renamed to resolve all ambiguities.

use crate::solidity::ast::Name;
use crate::solidity::ast::{utils::*, *};
use std::{collections::HashMap, ops::Deref};

/// Data structure modelling a function implementation that is used for target
/// data type.
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
struct UsedFunc {
    /// Name of the implementation function.
    pub func_name: Name,

    /// The target data type, `None` means all types (corresponding to `using
    /// for *`).
    pub target_typ: Option<Type>,
}

impl UsedFunc {
    pub fn new(func_name: Name, target_typ: Option<Type>) -> Self {
        UsedFunc { func_name, target_typ }
    }
}

/// Data structure modelling a function and its associated library.
#[derive(PartialEq, Eq, Debug, Clone)]
struct LibFunc {
    /// Name of the library.
    pub lib_name: Option<Name>,

    /// Definition of the function.
    pub func_def: FuncDef,
}

/// Data structure for removing using directives.
struct UsingEliminator {
    /// The input source_unit
    _source_unit: SourceUnit,

    /// of Mapping implementation functions to the libraries where they are
    /// implemented.
    impl_func_lib_map: HashMap<UsedFunc, LibFunc>,
}

impl UsingEliminator {
    /// Constructor.
    pub fn new(source_unit: &SourceUnit) -> UsingEliminator {
        UsingEliminator { _source_unit: source_unit.clone(), impl_func_lib_map: HashMap::new() }
    }

    /// Eliminate the `using` directives from source unit.
    pub fn eliminate_using(&mut self, source_unit: &SourceUnit) -> SourceUnit {
        self.map_source_unit(source_unit)
    }
}

impl Map<'_> for UsingEliminator {
    /// Override `map_source_unit` to remove `using` directives.
    fn map_source_unit<'a>(&mut self, source: &SourceUnit) -> SourceUnit {
        let nelems = source
            .elems
            .iter()
            .filter_map(|elem| match elem {
                // Remove `using` directives and transform other elements
                SourceUnitElem::Using(_) => None,
                _ => Some(self.map_source_unit_elem(elem)),
            })
            .collect();
        SourceUnit { elems: nelems, ..source.clone() }
    }

    /// Override `map_contract_def` to remove `using` directives.
    fn map_contract_def<'a>(&mut self, contract: &ContractDef) -> ContractDef {
        let nelems = contract
            .body
            .iter()
            .filter_map(|elem| match elem {
                // Remove `using`  directives and transform other elements
                ContractElem::Using(_) => None,
                _ => Some(self.map_contract_elem(elem)),
            })
            .collect();
        ContractDef { body: nelems, ..contract.clone() }
    }

    /// Override `map_call_expr` to transform calls to library functions.
    fn map_call_expr(&mut self, expr: &CallExpr) -> CallExpr {
        let nexpr = map::default::map_call_expr(self, expr);
        if let Expr::Member(m) = nexpr.callee.deref() {
            let base = &m.base;
            // Check if the base_expr is in the used for list.

            let func_name = &m.member;
            let impl_func2 = UsedFunc::new(func_name.clone(), Some(base.typ()));
            let impl_func1 = UsedFunc::new(func_name.clone(), None);

            // Find function in library first
            if let Some(lib_func) = self.impl_func_lib_map.get(&impl_func1) {
                let _nargs: CallArgs = match &nexpr.args {
                    CallArgs::Unnamed(_) => nexpr.args.clone(),
                    CallArgs::Named(named_args) => {
                        let func = &lib_func.func_def;
                        let first_arg_name = match func.params[..] {
                            [] => panic!("Lib function has no parameter: {}", func.name),
                            _ => func.params[0].name.clone(),
                        };
                        let first_arg =
                            NamedArg::new(first_arg_name.to_string(), base.as_ref().clone(), None);
                        let mut nnamed_args = named_args.clone();
                        nnamed_args.insert(0, first_arg);
                        CallArgs::Named(nnamed_args)
                    }
                };
                // let nargs = vec![base_expr.clone()];
                // FIXME: update new expr
                return nexpr;
            }
            // Then find free functions
            else if let Some(_lib_func) = self.impl_func_lib_map.get(&impl_func2) {
                // let lib_name = lib_func.clone();
                // let args: Vec<Expr> = match nexpr.args.clone() {
                //     CallArgs::Unnamed(args) => args.to_vec(),
                //     CallArgs::Named(args) => args.iter().map(|x| x.value.clone()).collect(),
                // };
                // let nargs = vec![base_expr.clone()];
                // FIXME: update new expr
                return nexpr;
            }

            // let lib_name = find_function_type(source_unit, lib_name,
            // func_name)
        }
        nexpr

        // if let Some(expr) = find_member_access_callee(expr.callee.deref()) {
        //     if let Some(pair) = self.functions.get(&expr.member) {
        //         let typ = pair.typ.clone();
        //         let base = &*expr.base;
        //         // Check the target type
        //         if self.check_target_type(typ, base) {
        //             let mut args: Vec<Expr> = match expr.args.clone() {
        //                 CallArgs::Unnamed(args) => args.to_vec(),
        //                 CallArgs::Named(args) => args.iter().map(|x|
        // x.value.clone()).collect(),             };
        //             let mut nargs = vec![base.clone()];
        //             nargs.append(&mut args);
        //             return self.create_func_call_for_member_access_callee(
        //                 &pair.func_name,
        //                 &expr,
        //                 nargs,
        //                 expr,
        //             );
        //         }
        //     }
        // }
        // expr.clone()
    }
}

/// Function to remove modifier invocations from function definitions.
pub fn eliminate_using_directives(source_units: &[SourceUnit]) -> Vec<SourceUnit> {
    let mut nsource_units = vec![];
    for sunit in source_units.iter() {
        let mut eliminator = UsingEliminator::new(sunit);
        nsource_units.push(eliminator.eliminate_using(sunit));
    }
    nsource_units
}

//-------------------------------------------------
// Unit tests
//-------------------------------------------------

/// Unit tests
#[cfg(test)]
mod tests {
    use crate::solidity::{
        ast::utils::syntactic_comparer::compare_source_units,
        lowering::{eliminate_using_directives, utils::configure_unit_test_env},
        parsing::parse_solidity_source_code,
    };
    use indoc::indoc;

    // Test removing the `using` directive.
    #[test]
    fn remove_using_1() {
        let _ = configure_unit_test_env();

        // Input contract
        let input_contract = indoc! {r###"
            library SomeLibrary  {
                function add(uint self, uint b) public returns (uint) {
                    return self+b;
                }
            }
            contract SomeContract {
                using SomeLibrary for uint;
                function add3(uint number) public returns (uint) {
                    return number.add(3);
                }
            }"###};

        // Expected output contract
        let expected_contract = indoc! {r###"
            library SomeLibrary  {
                function add(uint self, uint b) public returns (uint) {
                    return self+b;
                }
            }
            contract SomeContract {
                function add3(uint number) public returns (uint) {
                    return SomeLibrary.add(number, 3);
                }
            }"###};

        let input_sunit = match parse_solidity_source_code(input_contract, "0.8.15") {
            Ok(source_units) => source_units[0].clone(),
            Err(err) => panic!("Failed to parse input source unit: {err}"),
        };
        let output_sunits = eliminate_using_directives(&[input_sunit]);
        let expected_sunits = match parse_solidity_source_code(expected_contract, "0.8.15") {
            Ok(source_units) => source_units,
            Err(err) => panic!("Failed to parse expected source unit: {err}"),
        };
        if let Err(err) = compare_source_units(&output_sunits, &expected_sunits) {
            panic!("Failed to eliminate using directive: {err}")
        }
    }

    /// Test removing the `using` directive.
    #[test]
    fn remove_using_2() {
        let _ = configure_unit_test_env();

        // Input contract
        let input_contract = indoc! {r###"
            function f(uint x) pure returns (uint) {
                return x + 2;
            }
            function g(uint x) pure returns (uint) {
                return x + 8;
            }
            using {g, f} for uint;
            contract C {
                function test(uint x, uint y) public pure returns (uint, uint) {
                    return (x.f(), y.g());
                }
            }"###};

        // Expected output contract
        let expected_contract = indoc! {r###"
            function f(uint256 x) pure returns (uint256) {
                return x + 2;
            }
            function g(uint256 x) pure returns (uint256) {
                return x + 8;
            }
            contract C {
                function test(uint256 x, uint256 y) public pure returns (uint256, uint256) {
                    return (f(x), g(y));
                }
            }"###};

        let input_sunits = match parse_solidity_source_code(input_contract, "0.8.15") {
            Ok(source_units) => source_units,
            Err(err) => panic!("Failed to parse input source code: {err}"),
        };
        let output_sunits = eliminate_using_directives(&input_sunits);

        let expected_sunits = match parse_solidity_source_code(expected_contract, "0.8.15") {
            Ok(source_units) => source_units,
            Err(err) => panic!("Failed to parse expected source code: {err}"),
        };

        if let Err(err) = compare_source_units(&output_sunits, &expected_sunits) {
            panic!("Failed to eliminate using directive: {err}")
        }
    }
}
