//! AST-Based Detectors
//!
//! This module previously contained AST-based detectors that have been migrated to:
//! - `dfa/detectors/` - For IR-based detectors using data flow analysis
//! - `grep/detectors/` - For AST-based detectors using declarative pattern matching
//!
//! All detectors have been successfully migrated. This directory is kept for
//! potential future AST-based detectors that don't fit the DFA or GREP frameworks.
