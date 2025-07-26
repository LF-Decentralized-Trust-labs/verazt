//! Module to unroll tuples of single element, such as `(exp)` to `exp`.

use crate::{ast::*, util::map::default, util::*};
use std::ops::Deref;

/// Data structure to unroll tuples containing single element.
struct TupleUnroller {}

impl TupleUnroller {
    /// Unroll tuples containing single element in a source unit.
    pub fn unrolle_single_tuple(&self, source_unit: &SourceUnit) -> SourceUnit {
        let mut unroller = TupleUnroller {};
        unroller.map_source_unit(source_unit)
    }
}

/// Implement the `Map` utility to unroll tuples containing single element.
impl Map<'_> for TupleUnroller {
    /// Override `map_call_expr` to unroll callee expression which is a
    /// tuple containing single elements
    fn map_call_expr(&mut self, expr: &CallExpr) -> CallExpr {
        let nexpr = default::map_call_expr(self, expr);
        if let Expr::Tuple(tuple) = nexpr.callee.deref() {
            if let [Some(elem)] = &tuple.elems[..] {
                return CallExpr { callee: Box::new(elem.clone()), ..nexpr.clone() };
            }
        }
        nexpr
    }
}

/// Function to unroll unary tuples which only have single element in source
/// units.
pub fn unroll_unary_tuple(source_units: &[SourceUnit]) -> Vec<SourceUnit> {
    println!("Normalize AST: unroll single tuples");
    let mut nsource_units = vec![];
    for sunit in source_units {
        let unroller = TupleUnroller {};
        nsource_units.push(unroller.unrolle_single_tuple(sunit))
    }
    nsource_units
}
