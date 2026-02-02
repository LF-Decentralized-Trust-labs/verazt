//! Module normalizing AST.

use crate::ast::*;

pub mod elim_func_modifier;
pub mod elim_import_directives;
pub mod elim_named_args;
pub mod elim_using_directives;
pub mod flatten_expr;
pub mod flatten_name_index;
pub mod merge_pragmas;
pub mod rename_callees;
pub mod rename_contracts;
pub mod rename_definitions;
pub mod rename_variables;
pub mod resolve_inheritance;
pub mod substitution;
pub mod transform;
pub mod unroll_unary_tuple;

#[macro_use]
pub mod utils;

pub use elim_func_modifier::eliminate_modifier_invocs;
pub use elim_import_directives::eliminate_import;
pub use elim_named_args::eliminate_named_args;
pub use elim_using_directives::eliminate_using_directives;
pub use flatten_expr::flatten_expr;
pub use flatten_name_index::flatten_name;
pub use merge_pragmas::merge_pragmas;
use meta::NamingEnv;
pub use rename_callees::rename_callees;
pub use rename_contracts::rename_contracts;
pub use rename_definitions::rename_definitions;
pub use rename_variables::rename_variables;
pub use resolve_inheritance::resolve_inheritance;
pub use unroll_unary_tuple::unroll_unary_tuple;

/// Supporting function to print output source unit of a normalization step.
fn print_output_source_units(source_units: &[SourceUnit]) {
    trace!("Output source unit:");
    for source_unit in source_units {
        if log::max_level() >= log::Level::Trace {
            source_unit.print_highlighted_code();
            println!();
        }
    }
}

/// Run all normalization passes on source units.
///
/// The order of normalization steps is important.
pub fn run_passes(source_units: &[SourceUnit]) -> Vec<SourceUnit> {
    // First, unroll all single tuple to eliminate parenthesized-based expression.
    let source_units = unroll_unary_tuple(source_units);
    print_output_source_units(&source_units);

    // Rename contracts,
    let env = NamingEnv::new();
    let (source_units, env) = rename_contracts(&source_units, Some(&env));
    print_output_source_units(&source_units);

    // First, rename all variables.
    let (source_units, env) = rename_variables(&source_units, Some(&env));
    print_output_source_units(&source_units);

    // Eliminate using directives.
    let source_units = eliminate_using_directives(&source_units);
    print_output_source_units(&source_units);

    // Then, rename all definitions' names.
    let (source_units, env) = rename_definitions(&source_units, Some(&env));
    print_output_source_units(&source_units);

    // After that, merge imported elements to relevant source units.
    let source_units = eliminate_import(&source_units);
    print_output_source_units(&source_units);

    // Merge pragma directives
    let source_units = merge_pragmas(&source_units);
    print_output_source_units(&source_units);

    // Resolve inheritance.
    let source_units = resolve_inheritance(&source_units);
    print_output_source_units(&source_units);

    // Now, rename callees of call oprations to the previously renamed definitions.
    let (source_units, env) = rename_callees(&source_units, Some(&env));
    print_output_source_units(&source_units);

    let source_units = eliminate_named_args(&source_units);
    print_output_source_units(&source_units);

    let source_units = eliminate_modifier_invocs(&source_units);
    print_output_source_units(&source_units);

    let source_units = flatten_expr(&source_units, Some(&env));
    print_output_source_units(&source_units);

    // Unroll all single tuples again to eliminate parenthesized-based expression
    // introduced during the flattening
    unroll_unary_tuple(&source_units)
}
