//! Module removing the `using` directives in Solidity smart contracts.
//!
//! Prerequisite: Assuming that all contract and library names are already
//! renamed to resolve all ambiguities.

use crate::{ast::*, util::*};
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

#[derive(Clone)]
struct UsedFuncFinder<'a> {
    current_contract: Option<&'a ContractDef>,
    source_unit: &'a SourceUnit,
}

impl<'a> UsedFuncFinder<'a> {
    pub fn new(source_unit: &'a SourceUnit) -> Self {
        UsedFuncFinder { current_contract: None, source_unit }
    }

    fn find_used_funcs_in_using_library_directive(
        &self,
        _uldir: &UsingLib,
        _target_typ: &Type,
    ) -> Vec<UsedFunc> {
        // TODO: Find the library definition first
        vec![]
    }

    fn find_used_funcs_in_using_function_directive(
        &self,
        ufdir: &UsingFunc,
        target_typ: &Type,
    ) -> Vec<UsedFunc> {
        debug!("Using Func Directive: {ufdir}");
        for elem in self.source_unit.elems.iter() {
            if let SourceUnitElem::Func(func) = elem {
                // FIXME: need to find function by scopes
                if func.name == ufdir.func_name {
                    return vec![UsedFunc::new(func.name.clone(), Some(target_typ.clone()))];
                }
            }
        }
        vec![]
    }

    fn find_used_funcs_in_using(&self, udir: &UsingDir) -> Vec<UsedFunc> {
        debug!("Finding used functions");
        let target_typ = match &udir.target_type {
            Some(typ) => typ.clone(),
            None => todo!(),
        };
        match &udir.kind {
            UsingKind::UsingLib(ulib) => {
                self.find_used_funcs_in_using_library_directive(ulib, &target_typ)
            }
            UsingKind::UsingFunc(ufuncs) => ufuncs
                .iter()
                .flat_map(|ufunc| {
                    self.find_used_funcs_in_using_function_directive(ufunc, &target_typ)
                })
                .collect(),
        }
    }

    fn _find_used_funcs(&self) -> Vec<UsedFunc> {
        self.source_unit
            .elems
            .iter()
            .flat_map(|elem| match elem {
                SourceUnitElem::Using(udir) => self.find_used_funcs_in_using(udir),
                _ => vec![],
            })
            .collect()
    }
}

impl<'a> Fold<'a, Vec<UsedFunc>> for UsedFuncFinder<'a> {
    /// Override `fold_source_unit_elem` to find data types' implementation
    /// functions in `using` directives.
    fn fold_source_unit_elem(
        &mut self,
        acc: Vec<UsedFunc>,
        elem: &'a SourceUnitElem,
    ) -> Vec<UsedFunc> {
        if let SourceUnitElem::Using(udir) = elem {
            return self.find_used_funcs_in_using(udir);
        }
        fold::default::fold_source_unit_elem(self, acc, elem)
    }

    /// Override `fold_contract_def` to capture the current contract definition.
    fn fold_contract_def(
        &mut self,
        acc: Vec<UsedFunc>,
        contract: &'a ContractDef,
    ) -> Vec<UsedFunc> {
        // Save the current contract scope
        let saved_contract_scope = self.current_contract;

        // Update the new contract scope
        self.current_contract = Some(contract);
        let acc = fold::default::fold_contract_def(self, acc, contract);

        // Restore the contract scope
        self.current_contract = saved_contract_scope;
        acc
    }

    /// Override `fold_contract_elem` to find data types' implementation
    /// functions in `using` directives.
    fn fold_contract_elem(&mut self, acc: Vec<UsedFunc>, elem: &'a ContractElem) -> Vec<UsedFunc> {
        if let ContractElem::Using(udir) = elem {
            return self.find_used_funcs_in_using(udir);
        }
        fold::default::fold_contract_elem(self, acc, elem)
    }
}

/// Data structure modelling a function and its associated library.
#[derive(PartialEq, Eq, Debug, Clone)]
struct LibFunc {
    /// Name of the library.
    pub lib_name: Option<Name>,

    /// Definition of the function.
    pub func_def: FunctionDef,
}

impl LibFunc {
    pub fn _new(lib_name: Option<Name>, func_def: FunctionDef) -> Self {
        LibFunc { lib_name, func_def }
    }
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
    pub fn eliminate_using(&self, source_unit: &SourceUnit) -> SourceUnit {
        let mut _finder = UsedFuncFinder::new(source_unit);
        source_unit.clone()
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
    println!("Normalize AST: eliminate using directives");
    let mut nsource_units = vec![];
    for sunit in source_units.iter() {
        let eliminator = UsingEliminator::new(sunit);
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
    use crate::{
        compile::compile_solidity_source_code,
        normalize::{eliminate_using_directives, util::configure_unit_test_env},
        util::syntactic_comparer::compare_source_units,
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

        let input_sunit = match compile_solidity_source_code(input_contract, "0.8.15") {
            Ok(source_units) => source_units[0].clone(),
            Err(err) => panic!("Failed to parse input source unit: {err}"),
        };
        let output_sunits = eliminate_using_directives(&[input_sunit]);
        let expected_sunits = match compile_solidity_source_code(expected_contract, "0.8.15") {
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

        let input_sunits = match compile_solidity_source_code(input_contract, "0.8.15") {
            Ok(source_units) => source_units,
            Err(err) => panic!("Failed to parse input source code: {err}"),
        };
        let output_sunits = eliminate_using_directives(&input_sunits);

        let expected_sunits = match compile_solidity_source_code(expected_contract, "0.8.15") {
            Ok(source_units) => source_units,
            Err(err) => panic!("Failed to parse expected source code: {err}"),
        };

        if let Err(err) = compare_source_units(&output_sunits, &expected_sunits) {
            panic!("Failed to eliminate using directive: {err}")
        }
    }
}
