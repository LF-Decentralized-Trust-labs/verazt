//! Analysis passes organised by IR layer.
//!
//! - `base/` — abstract infrastructure: traits, metadata
//! - `sir/` — passes operating on the SIR (source-level IR)
//! - `bir/` — passes operating on the BIR (analysis IR)
//! - `vir/` — placeholder for future VIR-layer passes

pub mod bir;
pub mod base;
pub mod sir;
pub mod vir;
