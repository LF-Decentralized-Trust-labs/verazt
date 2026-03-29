//! Attribute system for SIR nodes.
//!
//! Attributes carry metadata on any IR node. The `#sir.*` namespace is
//! reserved for standard SIR annotations; dialect namespaces (`evm`, `move`,
//! etc.) are used for chain-specific metadata.

use std::fmt::{self, Display};

use common::loc::Loc;

/// An attribute attached to an IR node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attr {
    pub namespace: String, // e.g. "sir", "evm", "move", "anchor"
    pub key: String,
    pub value: AttrValue,
    pub span: Option<Loc>,
}

/// The value carried by an attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrValue {
    Bool(bool),
    Int(i64),
    String(String),
    Map(Vec<(String, AttrValue)>),
}

impl Attr {
    pub fn new(namespace: &str, key: &str, value: AttrValue) -> Self {
        Attr { namespace: namespace.to_string(), key: key.to_string(), value, span: None }
    }

    /// Create a standard SIR attribute.
    pub fn sir(key: &str, value: AttrValue) -> Self {
        Self::new("sir", key, value)
    }

    /// Create an EVM dialect attribute.
    pub fn evm(key: &str, value: AttrValue) -> Self {
        Self::new("evm", key, value)
    }

    /// Create a Move dialect attribute.
    pub fn move_dialect(key: &str, value: AttrValue) -> Self {
        Self::new("move", key, value)
    }

    /// Create an Anchor dialect attribute.
    pub fn anchor(key: &str, value: AttrValue) -> Self {
        Self::new("anchor", key, value)
    }

    /// Create a Spec dialect attribute.
    pub fn spec(key: &str, value: AttrValue) -> Self {
        Self::new("spec", key, value)
    }

    /// Attach a source span to this attribute.
    pub fn with_span(mut self, span: Option<Loc>) -> Self {
        self.span = span;
        self
    }
}

impl Display for Attr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#[{}.{} = {}]", self.namespace, self.key, self.value)
    }
}

impl Display for AttrValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttrValue::Bool(b) => write!(f, "{b}"),
            AttrValue::Int(i) => write!(f, "{i}"),
            AttrValue::String(s) => write!(f, "\"{s}\""),
            AttrValue::Map(entries) => {
                let items: Vec<String> =
                    entries.iter().map(|(k, v)| format!("{k}: {v}")).collect();
                write!(f, "{{{}}}", items.join(", "))
            }
        }
    }
}

/// Standard `#sir.*` attribute keys.
pub mod sir_attrs {
    pub const VISIBILITY: &str = "visibility"; // "public" | "internal" | "friend"
    pub const CALL_RISK: &str = "call_risk"; // reentrancy, delegate_storage
    pub const TAINT: &str = "taint";
    pub const SINK: &str = "sink";
    pub const ACCESS_CONTROL: &str = "access_control";
    pub const REENTRANCY_GUARD: &str = "reentrancy_guard";
    pub const UNCHECKED: &str = "unchecked";
    pub const OVERFLOW: &str = "overflow";
    pub const SOURCE_LANG: &str = "source_lang"; // "solidity" | "vyper" | "move" | "rust/anchor"
    pub const CHAIN_TARGET: &str = "chain_target"; // "evm" | "aptos" | "sui" | "solana"
    pub const COMPILER: &str = "compiler";
    pub const LOADED_DIALECTS: &str = "loaded_dialects";
    pub const IS_INTERFACE: &str = "is_interface";
    pub const MUTABILITY: &str = "mutability"; // "view" | "pure"
    pub const PRAGMA_SOLIDITY: &str = "pragma_solidity"; // e.g. "^0.8.0"
}

/// EVM dialect `#evm.*` attribute keys.
pub mod evm_attrs {
    pub const PAYABLE: &str = "payable";
    pub const NONREENTRANT: &str = "nonreentrant"; // key name for Vyper's @nonreentrant
    pub const IS_CONSTRUCTOR: &str = "is_constructor";
    pub const IS_CONSTANT: &str = "is_constant";
    pub const IS_IMMUTABLE: &str = "is_immutable";
}

/// Move dialect `#move.*` attribute keys.
pub mod move_attrs {
    pub const ABILITIES: &str = "abilities"; // ["key","store","copy","drop"]
    pub const ENTRY: &str = "entry";
    pub const VIEW: &str = "view";
    pub const ACQUIRES: &str = "acquires";
    pub const NATIVE: &str = "native";
    pub const PHANTOM: &str = "phantom";
    pub const IS_EVENT: &str = "is_event"; // Aptos #[event]
    pub const IS_INIT: &str = "is_init"; // Sui init function
    pub const SHARED: &str = "shared"; // Sui shared object
    pub const IS_OBJECT: &str = "is_object"; // Sui object (contains UID)
}

/// Anchor dialect `#anchor.*` attribute keys.
pub mod anchor_attrs {
    pub const PROGRAM: &str = "program";
    pub const ENTRY: &str = "entry";
    pub const CONSTRAINT: &str = "constraint"; // "init" | "mut" | "has_one:X" | "custom"
    pub const PAYER: &str = "payer";
    pub const SPACE: &str = "space";
    pub const SEEDS: &str = "seeds";
    pub const BUMP: &str = "bump";
    pub const CLOSE: &str = "close";
    pub const CHECK_NOTE: &str = "check_note";
    pub const DISCRIMINATOR_SIZE: &str = "discriminator_size";
}

/// Spec dialect `#spec.*` attribute keys.
pub mod spec_attrs {
    pub const GHOST: &str = "ghost";
}
