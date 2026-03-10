//! Analysis passes organised by IR layer.
//!
//! - `sir/` — passes operating on the SIR (source-level IR)
//! - `air/` — passes operating on the AIR (analysis IR)
//! - `vir/` — placeholder for future VIR-layer passes

pub mod air;
pub mod sir;
pub mod vir;
