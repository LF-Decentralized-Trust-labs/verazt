use solidity::ast::{Expr, Identifier, Lit, Stmt, Type, Loc};
use std::collections::HashMap;
use std::sync::Arc;

// Forward declarations for types that will be defined in analysis module
pub type SymbolTable = (); // Placeholder
pub type TypeIndex = (); // Placeholder
pub type Name = String; // Placeholder
pub type FunctionId = String; // Placeholder

/// Result of a pattern match
#[derive(Clone, Debug)]
pub struct Match {
    pub loc: Option<Loc>,
    pub captures: HashMap<String, CapturedNode>,
    pub context: MatchContext,
}

/// Captured AST node from a pattern match
#[derive(Clone, Debug)]
pub enum CapturedNode {
    Expr(Box<Expr>),
    Stmt(Box<Stmt>),
    Type(Box<Type>),
    Ident(Identifier),
    Lit(Lit),
}

/// Context available during pattern matching
#[derive(Clone, Debug)]
pub struct MatchContext {
    pub symbol_table: Option<Arc<SymbolTable>>,
    pub type_index: Option<Arc<TypeIndex>>,
    pub current_contract: Option<Name>,
    pub current_function: Option<FunctionId>,
    pub in_modifier: bool,
    pub in_loop: bool,
}

impl MatchContext {
    pub fn new() -> Self {
        Self {
            symbol_table: None,
            type_index: None,
            current_contract: None,
            current_function: None,
            in_modifier: false,
            in_loop: false,
        }
    }
}

impl Default for MatchContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Core pattern trait
pub trait Pattern: Send + Sync {
    /// Try to match against an expression
    fn match_expr(&self, expr: &Expr, ctx: &MatchContext) -> Option<Match>;

    /// Try to match against a statement
    fn match_stmt(&self, stmt: &Stmt, ctx: &MatchContext) -> Option<Match>;

    /// Name for debugging
    fn name(&self) -> &str;

    /// Description for documentation
    fn description(&self) -> &str;
}
