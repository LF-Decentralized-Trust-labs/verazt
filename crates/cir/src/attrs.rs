//! Attribute system for CIR nodes.
//!
//! Attributes carry metadata on any IR node. The `#cir.*` namespace is
//! reserved for standard CIR annotations; dialect namespaces (`evm`, `move`,
//! etc.) are used for chain-specific metadata.

use std::fmt::{self, Display};

/// An attribute attached to an IR node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attr {
    pub namespace: String, // e.g. "cir", "evm", "move", "anchor"
    pub key: String,
    pub value: AttrValue,
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
        Attr { namespace: namespace.to_string(), key: key.to_string(), value }
    }

    /// Create a standard CIR attribute.
    pub fn cir(key: &str, value: AttrValue) -> Self {
        Self::new("cir", key, value)
    }

    /// Create an EVM dialect attribute.
    pub fn evm(key: &str, value: AttrValue) -> Self {
        Self::new("evm", key, value)
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

/// Standard `#cir.*` attribute keys.
pub mod cir_attrs {
    pub const VISIBILITY: &str = "visibility"; // "public" | "internal"
    pub const CALL_RISK: &str = "call_risk"; // reentrancy, delegate_storage
    pub const TAINT: &str = "taint";
    pub const SINK: &str = "sink";
    pub const ACCESS_CONTROL: &str = "access_control";
    pub const REENTRANCY_GUARD: &str = "reentrancy_guard";
    pub const UNCHECKED: &str = "unchecked";
    pub const OVERFLOW: &str = "overflow";
}
