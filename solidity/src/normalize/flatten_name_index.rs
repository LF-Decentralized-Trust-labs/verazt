//! Module to flatten names.

use crate::{ast::*, util::*};
use meta::Name;

struct NameIdxFlattener {}

impl NameIdxFlattener {
    pub fn new() -> Self {
        Self {}
    }

    pub fn flatten_name(&mut self, source_units: &[SourceUnit]) -> Vec<SourceUnit> {
        self.map_source_units(source_units)
    }
}

impl Map<'_> for NameIdxFlattener {
    fn map_name(&mut self, name: &Name) -> Name {
        Name { base: format!("{}", name), index: None }
    }
}

pub fn flatten_name(source_units: &[SourceUnit]) -> Vec<SourceUnit> {
    println!("Normalize AST: flattening name indices");
    let mut flattener = NameIdxFlattener::new();
    flattener.flatten_name(source_units)
}
