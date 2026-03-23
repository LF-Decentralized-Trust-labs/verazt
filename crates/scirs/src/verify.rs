//! Common infrastructure shared by all IR verifiers.

use crate::sir::Span;
use std::fmt::{self, Display};

/// An error reported by an IR verifier pass.
#[derive(Debug, Clone)]
pub struct VerifyError {
    /// The pass that reported the error (e.g. "sir::type_well_formed").
    pub pass: &'static str,
    /// Human-readable description.
    pub message: String,
    /// Optional source location.
    pub span: Option<Span>,
}

impl VerifyError {
    pub fn new(pass: &'static str, message: impl Into<String>) -> Self {
        VerifyError { pass, message: message.into(), span: None }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
}

impl Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.pass, self.message)?;
        if let Some(span) = self.span {
            write!(f, " at {span}")?;
        }
        Ok(())
    }
}
