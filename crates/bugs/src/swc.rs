//-------------------------------------------------------------------------
// Data structures representing SWC (Smart Contract Weakness Classification)
//-------------------------------------------------------------------------

use crate::bug::BugCategory;
use serde::{Deserialize, Serialize};

/// A Smart Contract Weakness Classification (SWC) entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SWC {
    pub id: usize,
    pub title: String,
    pub description: String,
    pub category: BugCategory,
    pub cwe_ids: Vec<usize>,
}

/// Get the BugCategory for a given SWC ID.
pub fn category_from_swc(swc_id: usize) -> Option<BugCategory> {
    known_swc_entries()
        .into_iter()
        .find(|e| e.id == swc_id)
        .map(|e| e.category)
}

/// SWC registry with known entries.
pub fn known_swc_entries() -> Vec<SWC> {
    vec![
        SWC {
            id: 100,
            title: "Function Default Visibility".to_string(),
            description: "Functions that do not have a function visibility type specified are public by default.".to_string(),
            category: BugCategory::AccessControl,
            cwe_ids: vec![710],
        },
        SWC {
            id: 101,
            title: "Integer Overflow and Underflow".to_string(),
            description: "An overflow/underflow happens when an arithmetic operation reaches the maximum or minimum size of a type.".to_string(),
            category: BugCategory::Arithmetic,
            cwe_ids: vec![682],
        },
        SWC {
            id: 102,
            title: "Outdated Compiler Version".to_string(),
            description: "Using an outdated compiler version can be problematic.".to_string(),
            category: BugCategory::CodeQuality,
            cwe_ids: vec![937],
        },
        SWC {
            id: 103,
            title: "Floating Pragma".to_string(),
            description: "Contracts should be deployed with the same compiler version and flags that they have been tested with thoroughly.".to_string(),
            category: BugCategory::CodeQuality,
            cwe_ids: vec![664],
        },
        SWC {
            id: 104,
            title: "Unchecked Call Return Value".to_string(),
            description: "The return value of a message call is not checked.".to_string(),
            category: BugCategory::UncheckedLowLevelCalls,
            cwe_ids: vec![252],
        },
        SWC {
            id: 105,
            title: "Unprotected Ether Withdrawal".to_string(),
            description: "Due to missing or insufficient access controls, malicious parties can withdraw some or all Ether from the contract account.".to_string(),
            category: BugCategory::AccessControl,
            cwe_ids: vec![284],
        },
        SWC {
            id: 106,
            title: "Unprotected SELFDESTRUCT Instruction".to_string(),
            description: "Due to missing or insufficient access controls, malicious parties can self-destruct the contract.".to_string(),
            category: BugCategory::AccessControl,
            cwe_ids: vec![284],
        },
        SWC {
            id: 107,
            title: "Reentrancy".to_string(),
            description: "One of the major dangers of calling external contracts is that they can take over the control flow.".to_string(),
            category: BugCategory::Reentrancy,
            cwe_ids: vec![841],
        },
        SWC {
            id: 108,
            title: "State Variable Default Visibility".to_string(),
            description: "Labeling the visibility explicitly makes it easier to catch incorrect assumptions about who can access the variable.".to_string(),
            category: BugCategory::AccessControl,
            cwe_ids: vec![710],
        },
        SWC {
            id: 109,
            title: "Uninitialized Storage Pointer".to_string(),
            description: "Uninitialized local storage variables can point to unexpected storage locations in the contract.".to_string(),
            category: BugCategory::Other,
            cwe_ids: vec![824],
        },
        SWC {
            id: 110,
            title: "Assert Violation".to_string(),
            description: "The assert() function is meant to assert invariants; properly functioning code should never fail an assert.".to_string(),
            category: BugCategory::Other,
            cwe_ids: vec![670],
        },
        SWC {
            id: 111,
            title: "Use of Deprecated Solidity Functions".to_string(),
            description: "Several functions and operators in Solidity are deprecated.".to_string(),
            category: BugCategory::CodeQuality,
            cwe_ids: vec![477],
        },
        SWC {
            id: 112,
            title: "Delegatecall to Untrusted Callee".to_string(),
            description: "There exists a special variant of a message call, named delegatecall, which is identical to a message call apart from the fact that the code at the target address is executed in the context of the calling contract.".to_string(),
            category: BugCategory::AccessControl,
            cwe_ids: vec![829],
        },
        SWC {
            id: 113,
            title: "DoS with Failed Call".to_string(),
            description: "External calls can fail accidentally or deliberately, which can cause a DoS condition in the contract.".to_string(),
            category: BugCategory::DenialOfService,
            cwe_ids: vec![400],
        },
        SWC {
            id: 114,
            title: "Transaction Order Dependence".to_string(),
            description: "The miner of the block can choose the order of transactions, which can be exploited.".to_string(),
            category: BugCategory::FrontRunning,
            cwe_ids: vec![362],
        },
        SWC {
            id: 115,
            title: "Authorization through tx.origin".to_string(),
            description: "tx.origin is a global variable in Solidity which stores the address of the account that sent the transaction.".to_string(),
            category: BugCategory::AccessControl,
            cwe_ids: vec![345],
        },
        SWC {
            id: 116,
            title: "Block values as a proxy for time".to_string(),
            description: "Contracts often need access to time values to perform certain types of functionality. Values such as block.timestamp and block.number can be used.".to_string(),
            category: BugCategory::TimeManipulation,
            cwe_ids: vec![829],
        },
        SWC {
            id: 119,
            title: "Shadowing State Variables".to_string(),
            description: "Solidity allows for ambiguous naming of state variables when inheritance is used.".to_string(),
            category: BugCategory::CodeQuality,
            cwe_ids: vec![710],
        },
        SWC {
            id: 120,
            title: "Weak Sources of Randomness from Chain Attributes".to_string(),
            description: "Using chain attributes as a source of randomness is unreliable.".to_string(),
            category: BugCategory::BadRandomness,
            cwe_ids: vec![330],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_from_swc() {
        assert_eq!(category_from_swc(107), Some(BugCategory::Reentrancy));
        assert_eq!(category_from_swc(101), Some(BugCategory::Arithmetic));
        assert_eq!(category_from_swc(115), Some(BugCategory::AccessControl));
        assert_eq!(category_from_swc(104), Some(BugCategory::UncheckedLowLevelCalls));
        assert_eq!(category_from_swc(113), Some(BugCategory::DenialOfService));
        assert_eq!(category_from_swc(120), Some(BugCategory::BadRandomness));
        assert_eq!(category_from_swc(114), Some(BugCategory::FrontRunning));
        assert_eq!(category_from_swc(116), Some(BugCategory::TimeManipulation));
        assert_eq!(category_from_swc(9999), None);
    }

    #[test]
    fn test_known_swc_entries() {
        let entries = known_swc_entries();
        assert!(!entries.is_empty());
        // Check a few known entries
        let reentrancy = entries.iter().find(|e| e.id == 107).unwrap();
        assert_eq!(reentrancy.title, "Reentrancy");
        assert_eq!(reentrancy.category, BugCategory::Reentrancy);
    }

    #[test]
    fn test_swc_serde() {
        let entries = known_swc_entries();
        let json = serde_json::to_string(&entries).unwrap();
        let parsed: Vec<SWC> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), entries.len());
    }
}
