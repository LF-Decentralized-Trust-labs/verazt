//! SIR-layer analysis passes.
//!
//! The former `CfgPass` (SIR-level CFG construction stub) has been
//! removed.  CFG construction is now handled at the AIR level via
//! `frameworks::cfa`.  The `mlir::sir::cfg` data types remain
//! available for any SIR-level tooling.
