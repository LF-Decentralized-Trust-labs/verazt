use crate::irdfa::analyses::reaching_defs::Definition;
use crate::irdfa::cfg::BasicBlockId;
use crate::irdfa::var::VarId;
use solidity::ast::Loc;
use std::collections::{HashMap, HashSet};

/// A use point
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Use {
    pub var: VarId,
    pub block: BasicBlockId,
    pub stmt_index: usize,
    pub loc: Option<Loc>,
}

/// Def-use chain: maps definitions to their uses
pub struct DefUseChains {
    /// Definition -> Uses
    pub def_to_uses: HashMap<Definition, HashSet<Use>>,
    /// Use -> Reaching definitions
    pub use_to_defs: HashMap<Use, HashSet<Definition>>,
}

impl DefUseChains {
    pub fn new() -> Self {
        Self {
            def_to_uses: HashMap::new(),
            use_to_defs: HashMap::new(),
        }
    }

    pub fn add_def_use(&mut self, def: Definition, use_point: Use) {
        self.def_to_uses
            .entry(def.clone())
            .or_insert_with(HashSet::new)
            .insert(use_point.clone());

        self.use_to_defs
            .entry(use_point)
            .or_insert_with(HashSet::new)
            .insert(def);
    }
}

impl Default for DefUseChains {
    fn default() -> Self {
        Self::new()
    }
}

/// Def-use chains analysis pass (to be integrated with analysis framework)
pub struct DefUseChainsPass;
