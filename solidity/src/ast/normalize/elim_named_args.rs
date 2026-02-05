//! Module to eliminate named arguments in Solidity smart contracts.

use crate::ast::{utils::*, *};
use crate::ast::Name;
use std::collections::HashMap;

//-------------------------------------------------
// Finding parameter names
//-------------------------------------------------

/// Data structure for collecting informations of parameters' names in function
/// defintions, error definitions, event definitions, etc.
struct ParamNamesFinder {
    /// Name of the current contract when finding parameter names.
    current_contract: Option<String>,
}

impl ParamNamesFinder {
    /// Constructor
    pub fn new() -> Self {
        ParamNamesFinder { current_contract: None }
    }

    /// Encode name of a callable object to String
    pub fn encode_callable_name(&self, name: &Name) -> String {
        match &self.current_contract {
            Some(contract) => format!("{}.{}", contract, name),
            None => format!("{}", name),
        }
    }

    /// Find parameter names in a list of source units.
    ///
    /// Output is a [`HashMap`] that maps a definition name (funciton, event,
    /// error) to its parameter names.
    pub fn find_parameter_names(
        &mut self,
        source_units: &[SourceUnit],
    ) -> HashMap<String, Vec<String>> {
        let mut param_names_map = HashMap::new();
        self.fold_source_units(&mut param_names_map, source_units);
        param_names_map
    }
}

impl Fold<'_, &mut HashMap<String, Vec<String>>> for ParamNamesFinder {
    /// Override `fold_contract_def` to capture the current contract name.
    fn fold_contract_def<'a>(
        &mut self,
        acc: &'a mut HashMap<String, Vec<String>>,
        contract: &ContractDef,
    ) -> &'a mut HashMap<String, Vec<String>> {
        let saved_contract_scope = self.current_contract.clone();
        self.current_contract = Some(contract.name.to_string());
        let acc = fold::default::fold_contract_def(self, acc, contract);
        self.current_contract = saved_contract_scope;
        acc
    }

    /// Override `fold_func_def` to collect parameter names in
    /// function definition.
    fn fold_func_def<'a>(
        &mut self,
        acc: &'a mut HashMap<String, Vec<String>>,
        func: &FuncDef,
    ) -> &'a mut HashMap<String, Vec<String>> {
        let param_names: Vec<String> = func
            .params
            .iter()
            .map(|vdecl| vdecl.name.to_string())
            .collect();
        let callable_name = self.encode_callable_name(&func.name);
        acc.insert(callable_name, param_names);
        acc
    }

    /// Override `fold_error_def` to collect parameter names in error
    /// definition.
    fn fold_error_def<'a>(
        &mut self,
        acc: &'a mut HashMap<String, Vec<String>>,
        error: &ErrorDef,
    ) -> &'a mut HashMap<String, Vec<String>> {
        let param_names: Vec<String> = error
            .params
            .iter()
            .map(|vdecl| vdecl.name.to_string())
            .collect();
        let callable_name = self.encode_callable_name(&error.name);
        acc.insert(callable_name, param_names);
        acc
    }

    /// Override `fold_event_def` to collect parameter names in event
    /// definition.
    fn fold_event_def<'a>(
        &mut self,
        acc: &'a mut HashMap<String, Vec<String>>,
        event: &EventDef,
    ) -> &'a mut HashMap<String, Vec<String>> {
        let param_names: Vec<String> = event
            .params
            .iter()
            .map(|vdecl| vdecl.name.to_string())
            .collect();
        let callable_name = self.encode_callable_name(&event.name);
        acc.insert(callable_name, param_names);
        acc
    }
}

//-------------------------------------------------
// Eliminating argument names
//-------------------------------------------------

/// Data structure for eliminating named arguments in [`CallExpr`].
struct NamedArgsEliminator {
    /// Mapping a definition name (function, event, error, etc) to its parameter
    /// names.
    param_names_map: HashMap<String, Vec<String>>,

    /// Current contract.
    current_contract_scope: Option<String>,
}

impl NamedArgsEliminator {
    /// Constructor.
    fn new(source_units: &[SourceUnit]) -> Self {
        let mut param_names_finder = ParamNamesFinder::new();
        let param_names_map = param_names_finder.find_parameter_names(source_units);

        // debug!("** Parameter names:");
        // for name in param_names_map.keys() {
        //     if let Some(params) = param_names_map.get(name) {
        //         debug!("{} -> {}", name, params.join(", "));
        //     }
        // }
        NamedArgsEliminator { param_names_map, current_contract_scope: None }
    }
}

impl Map<'_> for NamedArgsEliminator {
    /// Override `map_contract_def` to capture the current contract
    /// scope.
    fn map_contract_def(&mut self, contract: &ContractDef) -> ContractDef {
        let saved_contract_scope = self.current_contract_scope.clone();
        self.current_contract_scope = Some(contract.name.to_string());
        let contract = map::default::map_contract_def(self, contract);
        self.current_contract_scope = saved_contract_scope;
        contract
    }

    /// Override `map_call_expr` to eliminate named arguments.
    fn map_call_expr(&mut self, expr: &CallExpr) -> CallExpr {
        match &expr.args {
            CallArgs::Unnamed(_) => expr.clone(),
            CallArgs::Named(args) => {
                let encoded_callee_name = match *expr.callee {
                    Expr::Member(_) => format!("{}", expr.callee),
                    _ => match &self.current_contract_scope {
                        Some(contract) => format!("{}.{}", contract, expr.callee),
                        None => format!("{}", expr.callee),
                    },
                };

                let param_names = match self.param_names_map.get(&encoded_callee_name) {
                    Some(names) => names,
                    None => return expr.clone(),
                };

                let mut unnamed_args = vec![];
                for param_name in param_names.iter() {
                    if let Some(arg) = args.iter().find(|arg| arg.name.eq(param_name)) {
                        unnamed_args.push(arg.value.clone());
                    }
                }

                if unnamed_args.len() != args.len() {
                    panic!("Named arguments were eliminated incorrectly!");
                }

                CallExpr { args: CallArgs::Unnamed(unnamed_args), ..expr.clone() }
            }
        }
    }
}

/// Function to remove named arguments.
pub fn eliminate_named_args(source_units: &[SourceUnit]) -> Vec<SourceUnit> {
    let mut eliminator = NamedArgsEliminator::new(source_units);
    eliminator.map_source_units(source_units)
}

//-------------------------------------------------
// Unit tests
//-------------------------------------------------

/// Unit tests
#[cfg(test)]
mod tests {
    use super::eliminate_named_args;
    use crate::{
        ast::utils::syntactic_comparer::compare_source_units,
        parser::parse_solidity_source_code, ast::normalize::utils::configure_unit_test_env,
    };
    use indoc::indoc;

    // Test normalization in a single contract.
    #[test]
    fn test_eliminate_named_arguments() {
        let _ = configure_unit_test_env();

        // Input contract
        let input_contract = indoc! {r###"
            contract XYZ {
                function someFuncWithManyInputs(
                    uint256 x,
                    uint256 y,
                    uint256 z,
                    address a,
                    bool b,
                    string memory c
                ) public pure returns (uint256) {}

                function callFunc() external pure returns (uint256) {
                    return someFuncWithManyInputs(1, 2, 3, address(0), true, "c");
                }

                function callFuncWithKeyValue() external pure returns (uint256) {
                    return
                        someFuncWithManyInputs({
                            a: address(0),
                            b: true,
                            c: "c",
                            x: 1,
                            y: 2,
                            z: 3
                        });
                }
            }"###};

        // Expected output contract
        let expected_contract = indoc! {r###"
            contract XYZ {
                function someFuncWithManyInputs(
                    uint256 x,
                    uint256 y,
                    uint256 z,
                    address a,
                    bool b,
                    string memory c
                ) public pure returns (uint256) {}

                function callFunc() external pure returns (uint256) {
                    return someFuncWithManyInputs(1, 2, 3, address(0), true, "c");
                }

                function callFuncWithKeyValue() external pure returns (uint256) {
                    return someFuncWithManyInputs(1, 2, 3, address(0), true, "c");
                }
            }"###};

        let input_sunits = match parse_solidity_source_code(input_contract, "0.8.19") {
            Ok(source_units) => source_units,
            Err(err) => panic!("Failed to parse input source unit: {}", err),
        };
        let output_sunits = eliminate_named_args(&input_sunits);

        let expected_sunits = match parse_solidity_source_code(expected_contract, "0.8.19") {
            Ok(source_units) => source_units,
            Err(err) => panic!("Failed to parse expected source unit: {}", err),
        };

        if let Err(err) = compare_source_units(&output_sunits, &expected_sunits) {
            panic!("Failed to eliminiate named args: {}", err)
        }
    }
}
