//! Module handling Soldity versions.

use crate::solidity::parsing::version_parser::version_parser;
use common::{error::Result, fail};
use node_semver::{Range, Version};
use std::fs;

/// Enumerate all avaiable Solidity versions.
fn init_solc_version_groups() -> Vec<Vec<Version>> {
    let mut solc_groups = vec![];

    // Solc 0.4.0 -> 0.4.26, sorted by newer patch version
    let mut solc_v4 = vec![];
    for i in (0..27).rev() {
        if let Ok(ver) = Version::parse(format!("0.4.{i}")) {
            solc_v4.push(ver)
        }
    }
    solc_groups.push(solc_v4);

    // Solc 0.5.0 -> 0.5.17, sorted by newer patch version
    let mut solc_v5 = vec![];
    for i in (0..18).rev() {
        if let Ok(ver) = Version::parse(format!("0.5.{i}")) {
            solc_v5.push(ver)
        }
    }
    solc_groups.push(solc_v5);

    // Solc 0.6.0 -> 0.6.12, sorted by newer patch version
    let mut solc_v6 = vec![];
    for i in (0..13).rev() {
        if let Ok(ver) = Version::parse(format!("0.6.{i}")) {
            solc_v6.push(ver)
        }
    }
    solc_groups.push(solc_v6);

    // Solc 0.7.0 -> 0.7.6, sorted by newer patch version
    let mut solc_v7 = vec![];
    for i in (0..7).rev() {
        if let Ok(ver) = Version::parse(format!("0.7.{i}")) {
            solc_v7.push(ver)
        }
    }
    solc_groups.push(solc_v7);

    // Solc 0.8.0 -> 0.8.20, sorted by newer patch version
    let mut solc_v8 = vec![];
    for i in (0..21).rev() {
        if let Ok(ver) = Version::parse(format!("0.8.{i}")) {
            solc_v8.push(ver)
        }
    }
    solc_groups.push(solc_v8);

    // Return all groups, sorted by older minor version
    solc_groups
}

/// Query the official Solidity binary index and return all known versions.
/// Falls back to the hardcoded list if the network is unavailable.
fn fetch_available_solc_versions() -> Vec<Version> {
    let platform = if cfg!(target_os = "macos") {
        "macosx-amd64"
    } else if cfg!(target_os = "windows") {
        "windows-amd64"
    } else {
        "linux-amd64"
    };
    let url = format!("https://binaries.soliditylang.org/{platform}/list.json");
    if let Ok(resp) = ureq::get(&url).call() {
        if let Ok(json) = resp.into_json::<serde_json::Value>() {
            let mut versions: Vec<Version> = json["releases"]
                .as_object()
                .unwrap_or(&serde_json::Map::new())
                .keys()
                .filter_map(|k| Version::parse(k).ok())
                .collect();
            versions.sort_by(|a, b| b.cmp(a));
            return versions;
        }
    }
    // Fallback: hardcoded list
    init_solc_version_groups().into_iter().flatten().collect()
}

/// Normalize a pragma version constraint string.
///
/// Inserts a space before comparison operators (`<`, `<=`, `>`, `>=`, `^`, `~`)
/// when they are directly preceded by a digit, e.g. `>=0.4.0<0.9.0` becomes
/// `>=0.4.0 <0.9.0`.
pub fn normalize_version_constraint(constraint: &str) -> String {
    let mut result = String::with_capacity(constraint.len() + 4);
    let chars: Vec<char> = constraint.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        if (ch == '<' || ch == '>' || ch == '^' || ch == '~')
            && i > 0
            && chars[i - 1].is_ascii_digit()
        {
            result.push(' ');
        }
        result.push(ch);
    }
    result
}

/// Check if a version satisfies a semantic version constraints.
pub fn check_version_constraint(version: &Version, constraint: &str) -> bool {
    let constraint = normalize_version_constraint(constraint);
    match Range::parse(&constraint) {
        Err(_) => false,
        Ok(constraint) => constraint.satisfies(version),
    }
}

/// Check if a version range satisfies a semantic version constraints.
pub fn check_range_constraint(range: &Range, constraint: &str) -> bool {
    match Range::parse(constraint) {
        Err(_) => false,
        Ok(constraint) => constraint.allows_any(range),
    }
}

/// Find all Solidity versions specified in an input Solidity smart contract.
pub fn find_pragma_solidity_versions(input_file: &str) -> Result<Vec<String>> {
    let content = match fs::read_to_string(input_file) {
        Ok(content) => content,
        Err(err) => fail!(err),
    };
    let pragma_versions = version_parser::parse_pragma_solidity_version(&content);
    // debug!("PRAGMA VERSIONS: {pragma_versions:?}");
    Ok(pragma_versions)
}

/// Find the best Solc version for an input Solidity smart contract.
///
/// If no Solidity pragma versions are specified in the smart contract, return
/// the latest Solc.
pub fn find_compatible_solc_versions(solc_ver: &Option<String>) -> Result<Vec<Version>> {
    // Enumerate all available Solc version groups (hardcoded)
    let mut solc_groups = init_solc_version_groups();

    // Sort Solc versions by interleaving over their groups
    let mut all_solc_versions = vec![];
    let mut i = 0;
    let mut stop = false;
    while !stop {
        stop = true;
        for solc_group in &solc_groups {
            if i < solc_group.len() {
                all_solc_versions.push(&solc_group[i]);
                stop = false;
            }
        }
        i += 1;
    }

    match solc_ver {
        None => {
            debug!("Pragma Solidity version is empty!");
            solc_groups.reverse();
            let latest_solc_vers = solc_groups.into_iter().flatten().collect();
            return Ok(latest_solc_vers);
        }
        Some(constraint) => {
            let constraint = normalize_version_constraint(constraint);
            let range = match Range::parse(&constraint) {
                Ok(range) => range,
                Err(_) => fail!("Invalid Solidity version constraint: {}!", constraint),
            };

            let solc_versions: Vec<Version> = all_solc_versions
                .into_iter()
                .filter_map(|v| match v.satisfies(&range) {
                    true => Some(v.clone()),
                    false => None,
                })
                .collect();

            if !solc_versions.is_empty() {
                return Ok(solc_versions);
            }

            // Hardcoded list failed — try fetching from the official index
            let fetched = fetch_available_solc_versions();
            let fetched_versions: Vec<Version> =
                fetched.into_iter().filter(|v| range.satisfies(v)).collect();

            if !fetched_versions.is_empty() {
                return Ok(fetched_versions);
            }

            fail!("No Solidity version satisfying constraint: {}!", constraint)
        }
    }
}

/// Return versions from the official Solidity index satisfying `pragma`.
/// Versions are sorted newest-first.
pub fn find_installable_solc_versions(pragma: &str) -> Result<Vec<Version>> {
    let range = Range::parse(pragma)
        .map_err(|e| common::error::create_error(format!("Invalid pragma '{pragma}': {e}")))?;
    let all = fetch_available_solc_versions();
    let filtered: Vec<Version> = all.into_iter().filter(|v| range.satisfies(v)).collect();
    Ok(filtered)
}
