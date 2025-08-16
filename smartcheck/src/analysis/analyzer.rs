//-------------------------------------------------------------------------
// Data structures representing Analyzer
//-------------------------------------------------------------------------

use solidity::ast::{ContractDef, FuncDef, SourceUnit, Stmt, source_unit};

pub struct Analyzer {}

//-------------------------------------------------------------------------
// Implementation for Analyzer
//-------------------------------------------------------------------------

impl Analyzer {
    fn analyze_source_unit(&self, source_unit: &SourceUnit) {}

    fn analyze_contract(&self, contract: &ContractDef) {}

    fn analyze_function(&self, func: &FuncDef) {}

    fn analyze_statement(&self, stmt: &Stmt) {}
}
