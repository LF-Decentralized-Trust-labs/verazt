//! Module for parsing Solidity version from pragma directives.

use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

/// Data structure representing a parse tree of a source code unit.
///
/// This data structure is automatically derived by [`Pest`] parser.
#[derive(Parser)]
#[grammar = "src/compile/version_parser/version_grammar.pest"]
struct PragmaParser;

impl PragmaParser {
    /// Parse a Solidity source unit.
    ///
    /// Returns a list of version pragma found in the source unit.
    fn parse_source_unit(pair: Pair<Rule>) -> Vec<String> {
        let pair_inners = pair.clone().into_inner();
        let mut versions = vec![];
        for p in pair_inners {
            if matches!(p.as_rule(), Rule::pragma_solidity_version) {
                let version = Self::parse_pragma_solidity_version(p);
                versions.push(version);
            }
        }
        versions
    }

    /// Parse a `pragma` solidity version.
    fn parse_pragma_solidity_version(pair: Pair<Rule>) -> String {
        pair.into_inner().as_str().to_string()
    }
}

/// Parse `pragma` version strings in a Solidity source code string.
///
/// Return a list of version strings found in the source unit.
pub fn parse_pragma_solidity_version(source_code: &str) -> Vec<String> {
    let mut pairs = match PragmaParser::parse(Rule::source_unit, source_code) {
        Ok(pairs) => pairs,
        Err(e) => {
            debug!("Unable to parse pragma solidity version: {e}");
            return vec![];
        }
    };

    // Skip `SOI` token of Pest
    match pairs.next() {
        Some(next_pairs) => PragmaParser::parse_source_unit(next_pairs),
        None => {
            debug!("Error while parsing pragma solidity version: {pairs}");
            vec![]
        }
    }
}
