//! AST Pattern Matching Framework
//!
//! This module provides a declarative pattern matching framework for Solidity
//! AST. It includes:
//!
//! - **Pattern Trait**: Core abstraction for matching patterns
//! - **Primitives**: Basic patterns (identifier, member access, function call,
//!   wildcards)
//! - **Composites**: Pattern combinators (And, Or, Not, Contains, Where)
//! - **Builder**: Fluent DSL for creating patterns
//! - **Matcher**: Multi-pattern single-pass matching engine
//!
//! # Example
//!
//! ```ignore
//! use smarthunt::grep::{PatternBuilder as P, PatternMatcher, MatchContext};
//!
//! // Create a pattern to match tx.origin == address
//! let pattern = P::binary_eq(P::tx_origin(), P::any());
//!
//! // Run matcher on AST
//! let mut matcher = PatternMatcher::new();
//! matcher.add_pattern("tx_origin_check", pattern);
//!
//! let ctx = MatchContext::new();
//! let results = matcher.match_all(&source_units, &ctx);
//! ```

pub mod builder;
pub mod composite;
pub mod core;
pub mod matcher;
pub mod primitives;

pub mod detectors;

// Re-export commonly used types
pub use builder::{BinaryPattern, PatternBuilder, PatternExt};
pub use composite::{AndPattern, ContainsPattern, NotPattern, OrPattern, WherePattern};
pub use core::{CapturedNode, Match, MatchContext, Pattern};
pub use matcher::PatternMatcher;
pub use primitives::{AnyExpr, AnyStmt, CallPattern, IdentPattern, MemberAccessPattern};
