//! Module to compare 2 ASTs.

use super::Compare;
use crate::ast::*;
use extlib::{error::Result, fail};
use itertools::izip;

/// Compare 2 source units using a syntactic comparer.
///
/// This function will compare the structure of 2 AST and ignore the source code
/// location of all program constructs.
pub fn compare_source_unit(source_unit1: &SourceUnit, source_unit2: &SourceUnit) -> Result<()> {
    let mut comparer = SyntacticComparer::new();
    comparer.compare_source_unit(source_unit1, source_unit2)
}

/// Compare 2 lists of source units.
pub fn compare_source_units(
    source_units1: &[SourceUnit],
    source_units2: &[SourceUnit],
) -> Result<()> {
    let mut comparer = SyntacticComparer::new();
    if source_units1.len() != source_units2.len() {
        fail!("Different number of source units!");
    }

    let mut sorted_sunits1 = source_units1.to_vec();
    sorted_sunits1.sort_by(|sunit1, sunit2| sunit1.path.cmp(&sunit2.path));

    let mut sorted_sunits2 = source_units2.to_vec();
    sorted_sunits2.sort_by(|sunit1, sunit2| sunit1.path.cmp(&sunit2.path));

    for (source_unit1, source_unit2) in izip!(sorted_sunits1, sorted_sunits2) {
        comparer.compare_source_unit(&source_unit1, &source_unit2)?;
    }

    Ok(())
}

/// Compare 2 contracts using a syntactic comparer.
pub fn compare_contract(contract1: &ContractDef, contract2: &ContractDef) -> Result<()> {
    let mut comparer = SyntacticComparer::new();
    comparer.compare_contract_def(contract1, contract2)
}

/// Data structure to compare 2 ASTs syntactically.
struct SyntacticComparer {}

impl SyntacticComparer {
    /// Constructor.
    pub fn new() -> Self {
        SyntacticComparer {}
    }
}

/// Implement the syntactic comparer using default behaviors in the [`Compare`]
/// trait.
impl Compare<'_> for SyntacticComparer {}
