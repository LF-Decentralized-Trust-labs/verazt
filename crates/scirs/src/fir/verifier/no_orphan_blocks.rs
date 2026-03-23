//! FIR verifier pass: no_orphan_blocks
//!
//! Every function in the family is reachable from the entry function
//! via tail calls.

use crate::fir::ops::{Module, TailCallData, Terminator};
use crate::verify::VerifyError;
use std::collections::{HashSet, VecDeque};

const PASS: &str = "fir::no_orphan_blocks";

pub fn check(module: &Module) -> Vec<VerifyError> {
    let mut errors = Vec::new();

    if module.functions.is_empty() {
        return errors;
    }

    // Build reachability from all entry functions.
    // Entry functions are those without `$` in their name (original BIR functions).
    let all_ids: HashSet<&str> = module.functions.iter().map(|f| f.id.0.as_str()).collect();

    // Build adjacency: func_name -> set of tail call targets
    let mut adj: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    for func in &module.functions {
        let targets = collect_targets(&func.term);
        adj.insert(func.id.0.as_str(), targets);
    }

    // BFS from entry functions
    let mut reachable: HashSet<&str> = HashSet::new();
    let mut queue: VecDeque<&str> = VecDeque::new();

    for func in &module.functions {
        if !func.id.0.contains('$') {
            reachable.insert(func.id.0.as_str());
            queue.push_back(func.id.0.as_str());
        }
    }

    while let Some(current) = queue.pop_front() {
        if let Some(targets) = adj.get(current) {
            for &target in targets {
                if reachable.insert(target) {
                    queue.push_back(target);
                }
            }
        }
    }

    // Report unreachable functions
    for id in &all_ids {
        if !reachable.contains(id) {
            errors.push(VerifyError::new(
                PASS,
                format!("function @{id} is not reachable from any entry function"),
            ));
        }
    }

    errors
}

fn collect_targets<'a>(term: &'a Terminator) -> Vec<&'a str> {
    fn add_tc<'a>(tc: &'a TailCallData, out: &mut Vec<&'a str>) {
        out.push(tc.callee.0.as_str());
    }

    let mut targets = Vec::new();
    match term {
        Terminator::TailCall(tc) => add_tc(tc, &mut targets),
        Terminator::Branch { then_call, else_call, .. } => {
            add_tc(then_call, &mut targets);
            add_tc(else_call, &mut targets);
        }
        Terminator::Return(_) | Terminator::Revert | Terminator::Unreachable => {}
    }
    targets
}
