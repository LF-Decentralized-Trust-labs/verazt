//! Alias analysis data structures.

use crate::interfaces::AliasGroupId;
use crate::ops::{OpId, OpRef};
use std::collections::HashMap;
use std::fmt::{self, Display};

/// Top-level alias map: groups all storage ops by alias group.
#[derive(Debug, Clone, Default)]
pub struct AliasMap {
    pub groups: HashMap<AliasGroupId, AliasGroup>,
}

/// A group of storage operations that may alias each other.
#[derive(Debug, Clone, Default)]
pub struct AliasGroup {
    pub id: AliasGroupId,
    /// Storage reads: (op_id, optional key operand).
    pub reads: Vec<(OpId, Option<OpRef>)>,
    /// Storage writes: (op_id, optional key operand).
    pub writes: Vec<(OpId, Option<OpRef>)>,
}

impl AliasMap {
    pub fn new() -> Self {
        AliasMap { groups: HashMap::new() }
    }

    /// Register a storage operation in the alias map.
    pub fn register(
        &mut self,
        group_id: AliasGroupId,
        op_id: OpId,
        is_write: bool,
        key: Option<OpRef>,
    ) {
        let group = self
            .groups
            .entry(group_id.clone())
            .or_insert_with(|| AliasGroup { id: group_id, reads: Vec::new(), writes: Vec::new() });
        if is_write {
            group.writes.push((op_id, key));
        } else {
            group.reads.push((op_id, key));
        }
    }

    /// Get all reads across all groups.
    pub fn all_reads(&self) -> Vec<(AliasGroupId, OpId, Option<OpRef>)> {
        let mut result = Vec::new();
        for (gid, group) in &self.groups {
            for (op_id, key) in &group.reads {
                result.push((gid.clone(), *op_id, *key));
            }
        }
        result
    }

    /// Get all writes across all groups.
    pub fn all_writes(&self) -> Vec<(AliasGroupId, OpId, Option<OpRef>)> {
        let mut result = Vec::new();
        for (gid, group) in &self.groups {
            for (op_id, key) in &group.writes {
                result.push((gid.clone(), *op_id, *key));
            }
        }
        result
    }
}

impl Display for AliasMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "AliasMap ({} groups):", self.groups.len())?;
        for (id, group) in &self.groups {
            writeln!(
                f,
                "  group {id}: {} reads, {} writes",
                group.reads.len(),
                group.writes.len()
            )?;
        }
        Ok(())
    }
}
