//! Analysis passes organised by IR layer.
//!
//! - `base/` — abstract infrastructure: traits, metadata
//! - `sir/` — passes operating on the SIR (source-level IR)
//! - `air/` — passes operating on the AIR (analysis IR)
//! - `vir/` — placeholder for future VIR-layer passes

pub mod air;
pub mod base;
pub mod sir;
pub mod vir;
