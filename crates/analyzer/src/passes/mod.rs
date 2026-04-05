//! Analysis passes organised by IR layer.
//!
//! - `base/` — abstract infrastructure: traits, metadata
//! - `sir/` — passes operating on the SIR (source-level IR)
//! - `bir/` — passes operating on the BIR (analysis IR)

pub mod base;
pub mod bir;
pub mod sir;
