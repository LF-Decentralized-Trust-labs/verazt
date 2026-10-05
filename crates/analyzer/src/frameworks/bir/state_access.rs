//! Abstract accesses to persistent contract state, with a conservative
//! may-alias relation between them.

use super::function_view::FunctionView;
use scirs::bir::ops::{Op, OpRef, Resource};
use scirs::sir::{Lit, Num, NumLit};

// ═══════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════

/// An access to contract state: a location such as `@balances` and the
/// keys it is indexed by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateAccess {
    /// Each key's integer value when it is a constant (e.g. `balances[2]`),
    /// `None` otherwise. Empty when the keys are unknown altogether.
    pub keys: Vec<Option<String>>,
    pub location: String,
}

// ═══════════════════════════════════════════════════════════════════
// StateAccess Implementations
// ═══════════════════════════════════════════════════════════════════

impl StateAccess {
    /// The access to `resource` indexed by `keys`, with constant integer
    /// keys resolved within `view`.
    pub fn resolve(view: &FunctionView, resource: &Resource, keys: &[OpRef]) -> Self {
        let int_key = |key: &OpRef| match view.constant(*key)? {
            Lit::Num(NumLit { value: Num::Int(n), .. }) => Some(n.value.to_string()),
            _ => None,
        };
        StateAccess { keys: keys.iter().map(int_key).collect(), location: resource.to_string() }
    }

    /// An access to `location` with unknown keys, which may touch any entry.
    pub fn unkeyed(location: String) -> Self {
        StateAccess { keys: vec![], location }
    }

    /// The state read by `op`, if any.
    pub fn read_by(view: &FunctionView, op: &Op) -> Option<Self> {
        let access = op.kind.storage_access()?;
        (!access.is_write).then(|| Self::resolve(view, &access.resource, &access.keys))
    }

    /// The state written by `op` itself, if any.
    pub fn written_by(view: &FunctionView, op: &Op) -> Option<Self> {
        let access = op.kind.storage_access()?;
        access
            .is_write
            .then(|| Self::resolve(view, &access.resource, &access.keys))
    }

    /// Returns `true` if the two accesses may touch the same state: their
    /// locations overlap and no key position holds two different constants.
    pub fn may_alias(&self, other: &StateAccess) -> bool {
        let keys_may_match = self.keys.iter().zip(&other.keys).all(|keys| match keys {
            (Some(a), Some(b)) => a == b,
            _ => true,
        });
        overlaps(&self.location, &other.location) && keys_may_match
    }
}

/// Returns `true` if two state locations may overlap: they are equal, or
/// one is a field path inside the other (`@accounts` and `@accounts.balance`).
fn overlaps(a: &str, b: &str) -> bool {
    let inside = |inner: &str, outer: &str| {
        inner
            .strip_prefix(outer)
            .is_some_and(|rest| rest.starts_with('.'))
    };
    a == b || inside(a, b) || inside(b, a)
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn access(location: &str, keys: &[Option<&str>]) -> StateAccess {
        let keys = keys.iter().map(|k| k.map(str::to_string)).collect();
        StateAccess { keys, location: location.to_string() }
    }

    #[test]
    fn test_may_alias_field_path_but_not_name_prefix() {
        let accounts = StateAccess::unkeyed("@accounts".to_string());
        assert!(accounts.may_alias(&StateAccess::unkeyed("@accounts.balance".to_string())));
        assert!(!accounts.may_alias(&StateAccess::unkeyed("@accountsOld".to_string())));
    }

    #[test]
    fn test_may_alias_rejects_only_distinct_constant_keys() {
        let at = |key| access("@balances", &[key]);
        assert!(!at(Some("1")).may_alias(&at(Some("2"))));
        assert!(at(Some("1")).may_alias(&at(None)));
        assert!(at(Some("1")).may_alias(&StateAccess::unkeyed("@balances".to_string())));
    }
}
