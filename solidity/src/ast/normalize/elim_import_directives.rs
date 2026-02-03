//! Module to eliminate `import` directive in Solidity smart contracts.

use crate::{
    ast::{utils::*, *},
    ast::normalize::substitution::NameSubstitutor,
};
use crate::ast::Name;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

/// Data structure for renaming imported names.
/// TODO: rename this data structure.
struct SubstituteImportedExpr {
    /// Mapping [TODO: docs]
    symbol_name_map: HashMap<String, Name>,
}

impl SubstituteImportedExpr {
    /// Constructor.
    pub fn new(symbol_name_map: HashMap<String, Name>) -> Self {
        SubstituteImportedExpr { symbol_name_map }
    }

    /// TODO: docs
    pub fn substitue(&mut self, elems: &[SourceUnitElem]) -> Vec<SourceUnitElem> {
        elems
            .iter()
            .map(|elem| self.map_source_unit_elem(elem))
            .collect()
    }
}

impl Map<'_> for SubstituteImportedExpr {
    fn map_expr(&mut self, expr: &Expr) -> Expr {
        if let Expr::Member(member_expr) = expr {
            let expr_name = format!("{}", member_expr);
            if let Some(symbol_name) = self.symbol_name_map.get(&expr_name) {
                Identifier::new(None, symbol_name.clone(), expr.typ(), expr.loc()).into()
            } else {
                map::default::map_expr(self, expr)
            }
        } else {
            map::default::map_expr(self, expr)
        }
    }
}

/// Construct a hash table mapping the file path of a [`SourceUnit`] to the
/// [`SourceUnit`].
fn construct_source_unit_map<'a>(
    source_units: &'a [SourceUnit],
) -> HashMap<String, &'a SourceUnit> {
    let mut source_unit_map: HashMap<String, &'a SourceUnit> = HashMap::new();
    for source_unit in source_units.iter() {
        source_unit_map.insert(source_unit.path.clone(), source_unit);
    }
    source_unit_map
}

/// Unfold the `import` directive that imports a source unit as an alias.
fn unfold_imported_source_unit(
    imported_elem_names: &mut HashSet<String>,
    imported_source_unit: &SourceUnit,
    source_unit_alias: &str,
    target_elems: &[SourceUnitElem],
) -> (Vec<SourceUnitElem>, Vec<SourceUnitElem>) {
    let mut imported_elems: Vec<SourceUnitElem> = vec![];

    // Mapping unit alias symbols named
    // TODO: rename this variable.
    let mut symbol_aliases: HashMap<String, Name> = HashMap::new();

    // Import only source unit elements having names, since only they are accessible
    // from the source unit alias.
    for elem in imported_source_unit.elems.iter() {
        if let Some(elem_name) = elem.get_name() {
            // Use the base name (without indexing) to construct the
            let elem_aliased_name = format!("{}.{}", source_unit_alias, elem_name.base);
            symbol_aliases.insert(elem_aliased_name, elem_name.clone());

            let imported_elem_name = format!("{}:{}", imported_source_unit.path, &elem_name);
            if !imported_elem_names.contains(&imported_elem_name) {
                imported_elem_names.insert(imported_elem_name);
                // TODO: selectively check and import only the elements that are accessed via
                // the source alias.
                imported_elems.push(elem.clone());
            }
        }
    }

    let mut substitutor = SubstituteImportedExpr::new(symbol_aliases);
    let subsituted_elems = substitutor.substitue(target_elems);

    (imported_elems, subsituted_elems)
}

/// Unfold the `import` directive that imports symbols in a source unit.
fn unfold_imported_symbols(
    imported_elem_names: &mut HashSet<String>,
    imported_source_unit: &SourceUnit,
    import_symbols: &[ImportSymbol],
    target_elems: &[SourceUnitElem],
) -> (Vec<SourceUnitElem>, Vec<SourceUnitElem>) {
    let mut imported_elems: Vec<SourceUnitElem> = vec![];
    let mut subst_elems: Vec<SourceUnitElem> = target_elems.to_vec();

    for symbol in import_symbols.iter() {
        let imported_elem = imported_source_unit
            .elems
            .iter()
            .find(|elem| match elem.get_name() {
                Some(elem_name) => elem_name.base == symbol.symbol_name,
                _ => false,
            });

        if let Some(elem) = imported_elem {
            let orig_name = match &elem.get_name() {
                Some(name) => name.clone(),
                None => panic!("Unfold imported symbol: element name not found: {}", { elem }),
            };
            let imported_elem_name = format!("{}:{}", imported_source_unit.path, orig_name);
            if !imported_elem_names.contains(&imported_elem_name) {
                imported_elem_names.insert(imported_elem_name);
                imported_elems.push(elem.clone());
            }
            if let Some(alias) = &symbol.symbol_alias {
                let alias_name = Name::new(alias.to_string(), None);
                let mut substitutor = NameSubstitutor::new(&[alias_name], &[orig_name]);
                subst_elems = substitutor.substitute_source_unit_elems(subst_elems.as_slice());
            }
        }
    }

    (imported_elems, subst_elems)
}

/// Extract an import directive from a list of source unit elements.
fn extract_import(elems: &[SourceUnitElem]) -> Option<(ImportDir, Vec<SourceUnitElem>)> {
    let mut other_elems = vec![];
    let mut iter = elems.iter();
    while let Some(elem) = iter.next() {
        if let SourceUnitElem::Import(import) = elem {
            for other_elem in iter {
                other_elems.push(other_elem.clone());
            }
            return Some((import.clone(), other_elems));
        } else {
            other_elems.push(elem.clone());
        }
    }
    None
}

/// Eliminate all import directives in a list of source units.
pub fn eliminate_import(source_units: &[SourceUnit]) -> Vec<SourceUnit> {
    let mut finished = false;
    let mut nsource_units = source_units.to_vec();
    let mut imported_elem_names: HashSet<String> = HashSet::new();
    while !finished {
        // Initialize data for each elimination iteration
        let all_source_units: Vec<SourceUnit> = nsource_units;
        let source_unit_map = construct_source_unit_map(&all_source_units);
        nsource_units = vec![];
        finished = true;
        for sunit in all_source_units.iter() {
            if let Some((import, other_elems)) = extract_import(&sunit.elems) {
                finished = false;
                let source_unit_path = Path::new(&sunit.path);
                let imported_path = import.get_import_path();

                // Compute the full path of the imported source unit
                let imported_full_path = match source_unit_path.parent() {
                    Some(parent_path) => parent_path
                        .join(&imported_path)
                        .to_str()
                        .unwrap_or(&imported_path)
                        .to_string(),
                    None => imported_path.clone(),
                };

                let imported_sunit = match source_unit_map.get(&imported_full_path) {
                    Some(sunit) => sunit,
                    None => return vec![],
                };

                let nelems = match &import.kind {
                    ImportKind::ImportSourceUnit(import_unit) => match &import_unit.alias {
                        Some(unit_alias) => {
                            let (mut output_elems, subsituted_elems) = unfold_imported_source_unit(
                                &mut imported_elem_names,
                                imported_sunit,
                                unit_alias,
                                &other_elems,
                            );
                            output_elems.extend(subsituted_elems);
                            output_elems
                        }
                        None => {
                            let mut output_elems = imported_sunit.elems.clone();
                            output_elems.extend(other_elems);
                            output_elems
                        }
                    },
                    ImportKind::ImportSymbols(import_symbols) => {
                        let (mut output_elems, substituted_elems) = unfold_imported_symbols(
                            &mut imported_elem_names,
                            imported_sunit,
                            &import_symbols.imported_symbols,
                            &other_elems,
                        );
                        output_elems.extend(substituted_elems);
                        output_elems
                    }
                };

                let nsunit = SourceUnit { elems: nelems, ..sunit.clone() };
                nsource_units.push(nsunit);
            } else {
                nsource_units.push(sunit.clone());
            }
        }
    }

    nsource_units
}

//-------------------------------------------------
// Unit tests
//-------------------------------------------------

/// Unit tests
#[cfg(test)]
mod tests {
    use super::eliminate_import;
    use crate::{
        ast::utils::syntactic_comparer::compare_source_units,
        compile::compile_solidity_source_code_list,
        ast::normalize::{
            rename_callees, rename_defs, rename_vars, utils::configure_unit_test_env,
        },
    };
    use indoc::indoc;

    /// Test removing circular imports in contracts.
    #[test]
    fn remove_circular_imports() {
        // use color_eyre::{Report, Result};
        let _ = configure_unit_test_env();

        // Input contract 1
        let input_1 = (
            "circular_import_s1.sol",
            indoc! {r###"
            import { f as g } from "circular_import_s2.sol";

            function f() pure returns (uint256) {
                return 1;
            }"###},
        );

        // Input contract 2
        let input_2 = (
            "circular_import_s2.sol",
            indoc! {r###"
            import { f as g } from "circular_import_s1.sol";

            function f() pure returns (uint256) {
                return 2;
            }"###},
        );

        // Expected output contract 1
        let expected_1 = (
            "circular_import_s1.sol",
            indoc! {r###"
            function f_1() pure returns (uint256) {
                return 2;
            }

            function f_0() pure returns (uint256) {
                return 1;
            }"###},
        );

        // Expected output contract 2
        let expected_2 = (
            "circular_import_s2.sol",
            indoc! {r###"
            function f_0() pure returns (uint256) {
                return 1;
            }

            function f_1() pure returns (uint256) {
                return 2;
            }"###},
        );

        let input_sunits = match compile_solidity_source_code_list(&[input_1, input_2], "0.8.15") {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse input source unit: {}", err),
        };

        let expected_sunits =
            match compile_solidity_source_code_list(&[expected_1, expected_2], "0.8.15") {
                Ok(sunits) => sunits,
                Err(err) => panic!("Failed to parse expected source unit: {}", err),
            };

        // Need to rename variables, definitions, etc before eliminating import
        // directives.
        let (output_sunits, env) = rename_vars(&input_sunits, None);
        let (output_sunits, env) = rename_defs(&output_sunits, Some(&env));
        let (output_sunits, _) = rename_callees(&output_sunits, Some(&env));

        // Elimiante import directives
        let output_sunits = eliminate_import(&output_sunits);

        if let Err(err) = compare_source_units(&output_sunits, &expected_sunits) {
            panic!("Failed to eliminiate import: {}", err)
        }
    }

    /// Test removing multiple level imports in contracts.
    #[test]
    fn remove_multiple_level_imports() {
        // use color_eyre::{Report, Result};
        let _ = configure_unit_test_env();

        // Input contract 1
        let input_1 = (
            "import_as_s1.sol",
            indoc! {r###"
            uint256 constant a = 89;

            function fre() pure returns (uint256) {
                return a;
            }

            function bar() returns (uint256) {
                return a;
            }"###},
        );

        // Input contract 2
        let input_2 = (
            "import_as_s2.sol",
            indoc! {r###"
            import {a as b, fre, fre as foo} from "import_as_s1.sol";
            import "import_as_s1.sol" as S1;

            uint256 constant a = 13;

            contract C {
                function f() public returns (uint256, uint256, uint256, uint256) {
                    uint256 n = foo();
                    return (a, fre(), S1.fre(), b);
                }
            }"###},
        );

        // Input contract 3
        let input_3 = (
            "import_as_s3.sol",
            indoc! {r###"
            import "import_as_s2.sol" as S2;

            uint256 constant a = 13;

            contract C {
                function f() public returns (uint256, uint256, uint256, uint256) {
                    uint256 n = S2.foo();
                    return (a, S2.fre(), S2.S1.bar(), S2.b);
                }
            }"###},
        );

        // Expected output contract 1
        let expected_1 = (
            "import_as_s1.sol",
            indoc! {r###"
            uint256 constant a_0 = 89;

            function fre_0() pure returns (uint256) {
                return a_0;
            }

            function bar_0() returns (uint256) {
                return a_0;
            }"###},
        );

        // Expected output contract 2
        let expected_2 = (
            "import_as_s2.sol",
            indoc! {r###"
            uint256 constant a_0 = 89;

            function fre_0() pure returns (uint256) {
                return a_0;
            }

            function bar_0() returns (uint256) {
                return a_0;
            }

            uint256 constant a_1 = 13;

            contract C_0 {
                function f_0() public returns (uint256, uint256, uint256, uint256) {
                    uint256 n_0 = fre_0();
                    return (a_1, fre_0(), fre_0(), a_0);
                }
            }"###},
        );

        // Expected output contract 3
        let expected_3 = (
            "import_as_s3.sol",
            indoc! {r###"
            uint256 constant a_0 = 89;

            function fre_0() pure returns (uint256) {
                return a_0;
            }

            function bar_0() returns (uint256) {
                return a_0;
            }

            uint256 constant a_1 = 13;

            contract C_0 {
                function f_0() public returns (uint256, uint256, uint256, uint256) {
                    uint256 n_0 = fre_0();
                    return (a_1, fre_0(), fre_0(), a_0);
                }
            }

            uint256 constant a_2 = 13;

            contract C_2 {
                function f_1() public returns (uint256, uint256, uint256, uint256) {
                    uint256 n = fre_0();
                    return (a_2, fre_0(), bar_0(), a_0);
                }
            }"###},
        );

        let input_sunits =
            match compile_solidity_source_code_list(&[input_1, input_2, input_3], "0.8.15") {
                Ok(sunits) => sunits,
                Err(err) => panic!("Failed to parse input source unit: {}", err),
            };

        let expected_sunits = match compile_solidity_source_code_list(
            &[expected_1, expected_2, expected_3],
            "0.8.15",
        ) {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse expected source unit: {}", err),
        };

        // Need to rename variables, definitions, etc before eliminating import
        // directives.
        let (output_sunits, env) = rename_vars(&input_sunits, None);
        let (output_sunits, env) = rename_defs(&output_sunits, Some(&env));
        let (output_sunits, _) = rename_callees(&output_sunits, Some(&env));

        // Elimiante import directives
        let output_sunits = eliminate_import(&output_sunits);

        if let Err(err) = compare_source_units(&output_sunits, &expected_sunits) {
            panic!("Failed to eliminiate import: {}", err)
        }
    }
}
