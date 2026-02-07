use crate::astmatch::composite::{AndPattern, ContainsPattern, NotPattern, OrPattern, WherePattern};
use crate::astmatch::core::{Match, Pattern};
use crate::astmatch::primitives::{
    AnyExpr, AnyStmt, CallPattern, IdentPattern, MemberAccessPattern,
};
use solidity::ast::BinOp;

/// Builder for creating patterns fluently
pub struct PatternBuilder;

impl PatternBuilder {
    /// Match identifier
    pub fn ident(name: &str) -> IdentPattern {
        IdentPattern::new(name)
    }

    /// Match member access: obj.member
    pub fn member(object: impl Pattern + 'static, member: &str) -> MemberAccessPattern {
        MemberAccessPattern::new(Box::new(object), member)
    }

    /// Match tx.origin
    pub fn tx_origin() -> MemberAccessPattern {
        Self::member(Self::ident("tx"), "origin")
    }

    /// Match msg.sender
    pub fn msg_sender() -> MemberAccessPattern {
        Self::member(Self::ident("msg"), "sender")
    }

    /// Match msg.value
    pub fn msg_value() -> MemberAccessPattern {
        Self::member(Self::ident("msg"), "value")
    }

    /// Match msg.data
    pub fn msg_data() -> MemberAccessPattern {
        Self::member(Self::ident("msg"), "data")
    }

    /// Match block.timestamp
    pub fn block_timestamp() -> MemberAccessPattern {
        Self::member(Self::ident("block"), "timestamp")
    }

    /// Match block.number
    pub fn block_number() -> MemberAccessPattern {
        Self::member(Self::ident("block"), "number")
    }

    /// Match a function call
    pub fn call(callee: impl Pattern + 'static) -> CallPattern {
        CallPattern::new(Box::new(callee))
    }

    /// Match any expression
    pub fn any() -> AnyExpr {
        AnyExpr::new()
    }

    /// Match any statement
    pub fn any_stmt() -> AnyStmt {
        AnyStmt::new()
    }

    /// Match any of the patterns (OR)
    pub fn any_of(patterns: Vec<Box<dyn Pattern>>) -> OrPattern {
        OrPattern::new(patterns)
    }

    /// Match all patterns (AND)
    pub fn all_of(patterns: Vec<Box<dyn Pattern>>) -> AndPattern {
        AndPattern::new(patterns)
    }

    /// Match expression containing pattern
    pub fn contains(inner: impl Pattern + 'static) -> ContainsPattern {
        ContainsPattern::new(Box::new(inner))
    }

    /// Match with NOT
    pub fn not(inner: impl Pattern + 'static) -> NotPattern {
        NotPattern::new(Box::new(inner))
    }

    /// Binary comparison pattern helper
    pub fn binary_eq(left: impl Pattern + 'static, right: impl Pattern + 'static) -> BinaryPattern {
        BinaryPattern::new(Box::new(left), BinOp::Eq, Box::new(right))
    }

    /// Binary not-equal pattern helper
    pub fn binary_ne(left: impl Pattern + 'static, right: impl Pattern + 'static) -> BinaryPattern {
        BinaryPattern::new(Box::new(left), BinOp::NotEq, Box::new(right))
    }
}

/// Binary operation pattern
pub struct BinaryPattern {
    pub left: Box<dyn Pattern>,
    pub op: BinOp,
    pub right: Box<dyn Pattern>,
}

impl BinaryPattern {
    pub fn new(left: Box<dyn Pattern>, op: BinOp, right: Box<dyn Pattern>) -> Self {
        Self { left, op, right }
    }
}

impl Pattern for BinaryPattern {
    fn match_expr(&self, expr: &solidity::ast::Expr, ctx: &crate::astmatch::core::MatchContext) -> Option<Match> {
        if let solidity::ast::Expr::Binary(b) = expr {
            if b.op == self.op {
                let left_match = self.left.match_expr(&b.left, ctx)?;
                let right_match = self.right.match_expr(&b.right, ctx)?;

                // Merge captures
                let mut captures = left_match.captures;
                captures.extend(right_match.captures);

                return Some(Match {
                    loc: b.loc,
                    captures,
                    context: ctx.clone(),
                });
            }
        }
        None
    }

    fn match_stmt(&self, _stmt: &solidity::ast::Stmt, _ctx: &crate::astmatch::core::MatchContext) -> Option<Match> {
        None
    }

    fn name(&self) -> &str {
        "BinaryPattern"
    }

    fn description(&self) -> &str {
        "Matches a binary operation with specific operator"
    }
}

/// Extension trait for patterns
pub trait PatternExt: Pattern + Sized {
    /// Add a predicate condition
    fn where_fn<F: Fn(&Match) -> bool + Send + Sync + 'static>(self, pred: F) -> WherePattern
    where
        Self: 'static,
    {
        WherePattern::new(Box::new(self), pred)
    }

    /// Combine with AND
    fn and(self, other: impl Pattern + 'static) -> AndPattern
    where
        Self: 'static,
    {
        AndPattern::new(vec![Box::new(self), Box::new(other)])
    }

    /// Combine with OR
    fn or(self, other: impl Pattern + 'static) -> OrPattern
    where
        Self: 'static,
    {
        OrPattern::new(vec![Box::new(self), Box::new(other)])
    }
}

// Implement PatternExt for all Pattern types
impl<T: Pattern> PatternExt for T {}
