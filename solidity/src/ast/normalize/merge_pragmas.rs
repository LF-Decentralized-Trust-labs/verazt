//! Module to merge `pragma` directives in source unit elements.

use crate::ast::*;
use itertools::Itertools;
use std::cmp::Ordering;

/// Merge pragma directives in source unit elements.
pub fn merge_pragma_elems(source_unit_elems: &[SourceUnitElem]) -> Vec<SourceUnitElem> {
    let mut solidity_pragmas: Vec<PragmaDir> = vec![];
    let mut abicoder_pragmas: Vec<PragmaDir> = vec![];
    let mut experimental_pragmas: Vec<PragmaDir> = vec![];
    let mut other_elems: Vec<SourceUnitElem> = vec![];

    // Extract pragma directives and other source unit elements.
    for elem in source_unit_elems.iter() {
        if let SourceUnitElem::Pragma(p) = elem {
            match p.kind {
                PragmaKind::Version(_) => solidity_pragmas.push(p.clone()),
                PragmaKind::AbiCoder(_) => abicoder_pragmas.push(p.clone()),
                PragmaKind::Experimental(_) => experimental_pragmas.push(p.clone()),
            }
        } else {
            other_elems.push(elem.clone())
        }
    }

    // Merge source unit elements.
    let mut nelems: Vec<SourceUnitElem> = vec![];

    // Merge all solidity constraint into a pragma string.
    if !solidity_pragmas.is_empty() {
        let solidity_vers = solidity_pragmas
            .iter()
            .flat_map(|p| match &p.kind {
                PragmaKind::Version(v) => Some(v.to_string()),
                _ => None,
            })
            .collect::<Vec<String>>()
            .join(", ");
        let loc = match &solidity_pragmas[..] {
            [p] => p.loc,
            _ => None,
        };
        let npragma = PragmaDir::new(None, PragmaKind::Version(solidity_vers), loc);
        nelems.push(npragma.into());
    }

    // Take the newest abicoder pragma, e.g., v2 should have a higher priority than
    // v1.
    if !abicoder_pragmas.is_empty() {
        let sorted_pragmas: Vec<&PragmaDir> = abicoder_pragmas
            .iter()
            .sorted_by(|p1, p2| match (&p1.kind, &p2.kind) {
                (PragmaKind::AbiCoder(v1), PragmaKind::AbiCoder(v2)) => v2.cmp(v1),
                (PragmaKind::AbiCoder(_), _) => Ordering::Less,
                (_, PragmaKind::AbiCoder(_)) => Ordering::Greater,
                _ => Ordering::Equal,
            })
            .collect();
        nelems.push(sorted_pragmas[0].clone().into());
    }

    // Migrate all experimental pragma directives.
    for p in experimental_pragmas.iter() {
        nelems.push(p.clone().into());
    }

    nelems.extend(other_elems);
    nelems
}

/// Merge all `pragma` directives in a list of source units.
pub fn merge_pragmas(source_units: &[SourceUnit]) -> Vec<SourceUnit> {
    source_units
        .iter()
        .map(|sunit| {
            let nelems = merge_pragma_elems(&sunit.elems);
            SourceUnit { elems: nelems, ..sunit.clone() }
        })
        .collect()
}
