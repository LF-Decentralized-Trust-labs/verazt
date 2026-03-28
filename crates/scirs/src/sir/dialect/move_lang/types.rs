//! Move dialect types.

use crate::sir::types::Type;
use std::fmt::{self, Display};

/// Move-specific types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MoveType {
    /// `!move.resource<T>` — struct type with key ability (lives in global
    /// storage).
    Resource(Box<Type>),
    /// `!move.signer` — signer capability passed to entry functions.
    Signer,
    /// `!move.type_tag` — phantom type tag (used in forall quantifiers in
    /// specs).
    TypeTag,
}

impl Display for MoveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveType::Resource(ty) => write!(f, "!move.resource<{ty}>"),
            MoveType::Signer => write!(f, "!move.signer"),
            MoveType::TypeTag => write!(f, "!move.type_tag"),
        }
    }
}
