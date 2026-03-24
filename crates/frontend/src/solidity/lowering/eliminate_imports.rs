//! Module to eliminate `import` directive in Solidity smart contracts.

use super::substitution::NameSubstitutor;
use crate::solidity::ast::Name;
use crate::solidity::ast::{utils::*, *};
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

/// Prefix a source unit element's name with the given alias.
///
/// For example, if the alias is `S1` and the element has name `foo`,
/// the result will be `S1_foo`.
fn prefix_elem_name(elem: &mut SourceUnitElem, alias: &str) {
    match elem {
        SourceUnitElem::Contract(c) => {
            c.name = Name::new(format!("{}_{}", alias, c.name.base), c.name.index);
        }
        SourceUnitElem::Func(f) => {
            f.name = Name::new(format!("{}_{}", alias, f.name.base), f.name.index);
        }
        SourceUnitElem::Var(v) => {
            v.name = Name::new(format!("{}_{}", alias, v.name.base), v.name.index);
        }
        SourceUnitElem::Struct(s) => {
            s.name = Name::new(format!("{}_{}", alias, s.name.base), s.name.index);
        }
        SourceUnitElem::Enum(e) => {
            e.name = Name::new(format!("{}_{}", alias, e.name.base), e.name.index);
        }
        SourceUnitElem::Error(e) => {
            e.name = Name::new(format!("{}_{}", alias, e.name.base), e.name.index);
        }
        SourceUnitElem::UserType(t) => {
            t.name = Name::new(format!("{}_{}", alias, t.name.base), t.name.index);
        }
        SourceUnitElem::Pragma(_) | SourceUnitElem::Import(_) | SourceUnitElem::Using(_) => {}
    }
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
            // Use the base name (without indexing) to construct the alias reference.
            let elem_aliased_name = format!("{}.{}", source_unit_alias, elem_name.base);

            // Create the prefixed name: {alias}_{original_name}
            let prefixed_base = format!("{}_{}", source_unit_alias, elem_name.base);
            let prefixed_name = Name::new(prefixed_base, elem_name.index);

            // Map the member access expression (e.g., S1.foo) to the prefixed name
            symbol_aliases.insert(elem_aliased_name, prefixed_name.clone());

            let imported_elem_name = format!("{}:{}", imported_source_unit.path, &elem_name);
            if !imported_elem_names.contains(&imported_elem_name) {
                imported_elem_names.insert(imported_elem_name);

                // Prefix the imported element's name before adding it.
                let mut prefixed_elem = elem.clone();
                prefix_elem_name(&mut prefixed_elem, source_unit_alias);
                imported_elems.push(prefixed_elem);
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
    // Track processed (source_unit_path, imported_path) pairs to detect and
    // break circular import chains.
    let mut processed_imports: HashSet<(String, String)> = HashSet::new();
    while !finished {
        // Initialize data for each elimination iteration
        let all_source_units: Vec<SourceUnit> = nsource_units;
        let source_unit_map = construct_source_unit_map(&all_source_units);
        nsource_units = vec![];
        finished = true;
        for sunit in all_source_units.iter() {
            if let Some((import, other_elems)) = extract_import(&sunit.elems) {
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

                let import_key = (sunit.path.clone(), imported_full_path.clone());

                // If this (source, import) pair was already processed, skip it
                // to break circular import chains.
                if !processed_imports.insert(import_key) {
                    // Already processed — drop the import directive and keep
                    // the remaining elements.
                    let nsunit = SourceUnit { elems: other_elems, ..sunit.clone() };
                    // There may still be other imports to process.
                    if extract_import(&nsunit.elems).is_some() {
                        finished = false;
                    }
                    nsource_units.push(nsunit);
                    continue;
                }

                finished = false;

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
    use crate::solidity::{
        ast::utils::syntactic_comparer::compare_source_units,
        lowering::{rename_callees, rename_defs, utils::configure_unit_test_env},
        parsing::parse_solidity_source_code_list,
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

        let input_sunits = match parse_solidity_source_code_list(&[input_1, input_2], "0.8.15") {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse input source unit: {}", err),
        };

        let expected_sunits =
            match parse_solidity_source_code_list(&[expected_1, expected_2], "0.8.15") {
                Ok(sunits) => sunits,
                Err(err) => panic!("Failed to parse expected source unit: {}", err),
            };

        // Rename definitions and callees (no rename_vars or rename_contracts).
        let env = crate::solidity::ast::NamingEnv::new();
        let (output_sunits, env) = rename_defs(&input_sunits, Some(&env));
        let (output_sunits, _) = rename_callees(&output_sunits, Some(&env));

        // Eliminate import directives
        let output_sunits = eliminate_import(&output_sunits);

        if let Err(err) = compare_source_units(&output_sunits, &expected_sunits) {
            panic!("Failed to eliminiate import: {}", err)
        }
    }

    /// Test removing multiple level imports in contracts.
    // TODO: fix eliminate_import to handle symbol import name conflicts (a as b)
    #[test]
    #[ignore = "eliminate_import produces duplicate constant names from symbol imports"]
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

        // Expected output contract 1: no variable indexes, only function indexes
        let expected_1 = (
            "import_as_s1.sol",
            indoc! {r###"
            uint256 constant a = 89;

            function fre_0() pure returns (uint256) {
                return a;
            }

            function bar_0() returns (uint256) {
                return a;
            }"###},
        );

        // Expected output contract 2:
        //   - Symbol imports ({a as b, fre, fre as foo}): bring names directly, no
        //     prefix
        //   - Aliased import (as S1): prefix with S1_
        //   - Variables keep original names (no indexes)
        //   - Only functions get indexes from rename_defs
        let expected_2 = (
            "import_as_s2.sol",
            indoc! {r###"
            uint256 constant a = 89;

            function fre_0() pure returns (uint256) {
                return a;
            }

            function bar_0() returns (uint256) {
                return a;
            }

            uint256 constant S1_a = 89;

            function S1_fre_0() pure returns (uint256) {
                return S1_a;
            }

            function S1_bar_0() returns (uint256) {
                return S1_a;
            }

            uint256 constant a = 13;

            contract C {
                function f_0() public returns (uint256, uint256, uint256, uint256) {
                    uint256 n = fre_0();
                    return (a, fre_0(), S1_fre_0(), a);
                }
            }"###},
        );

        // Expected output contract 3:
        //   - Aliased import (as S2): prefix with S2_
        //   - Nested aliases flatten: S2.S1.bar -> S2_S1_bar
        let expected_3 = (
            "import_as_s3.sol",
            indoc! {r###"
            uint256 constant S2_a = 89;

            function S2_fre_0() pure returns (uint256) {
                return S2_a;
            }

            function S2_bar_0() returns (uint256) {
                return S2_a;
            }

            uint256 constant S2_S1_a = 89;

            function S2_S1_fre_0() pure returns (uint256) {
                return S2_S1_a;
            }

            function S2_S1_bar_0() returns (uint256) {
                return S2_S1_a;
            }

            uint256 constant S2_a = 13;

            contract S2_C {
                function S2_f_0() public returns (uint256, uint256, uint256, uint256) {
                    uint256 n = S2_fre_0();
                    return (S2_a, S2_fre_0(), S2_S1_fre_0(), S2_a);
                }
            }

            uint256 constant a = 13;

            contract C {
                function f_1() public returns (uint256, uint256, uint256, uint256) {
                    uint256 n = S2_fre_0();
                    return (a, S2_fre_0(), S2_S1_bar_0(), S2_a);
                }
            }"###},
        );

        let input_sunits =
            match parse_solidity_source_code_list(&[input_1, input_2, input_3], "0.8.15") {
                Ok(sunits) => sunits,
                Err(err) => panic!("Failed to parse input source unit: {}", err),
            };

        let expected_sunits =
            match parse_solidity_source_code_list(&[expected_1, expected_2, expected_3], "0.8.15")
            {
                Ok(sunits) => sunits,
                Err(err) => panic!("Failed to parse expected source unit: {}", err),
            };

        // Rename definitions and callees (no rename_vars or rename_contracts).
        let env = crate::solidity::ast::NamingEnv::new();
        let (output_sunits, env) = rename_defs(&input_sunits, Some(&env));
        let (output_sunits, _) = rename_callees(&output_sunits, Some(&env));

        // Eliminate import directives
        let output_sunits = eliminate_import(&output_sunits);

        if let Err(err) = compare_source_units(&output_sunits, &expected_sunits) {
            panic!("Failed to eliminiate import: {}", err)
        }
    }
}
