//! Pass 2a: CIR → AIR lowering.
//!
//! This module orchestrates the four-step transformation from
//! CIR (Canonical IR) into AIR.

pub mod cfg;
pub mod dialect_lower;
pub mod icfg;
pub mod ssa;

use crate::air::module::Module;
use thiserror::Error;

/// Errors that can occur during CIR → AIR lowering.
#[derive(Debug, Error)]
pub enum LowerError {
    #[error("Untagged dialect op after Step 3: {0}")]
    UntaggedDialectOp(String),

    #[error("SSA renaming error: {0}")]
    SsaError(String),

    #[error("CFG construction error: {0}")]
    CfgError(String),

    #[error("ICFG construction error: {0}")]
    IcfgError(String),
}

/// Lower a CIR CanonModule into an AIR Module.
///
/// This runs the four-step Pass 2a transformation:
///   1. CFG Construction
///   2. SSA Renaming
///   3. Dialect Lowering
///   4. ICFG + Alias + Taint init
///
/// NOTE: Modifier expansion (formerly Step 1) is now handled by the
/// CIR lowering pass.
pub fn lower_module(cir: &crate::cir::CanonModule) -> Result<Module, LowerError> {
    use crate::air::cfg::{Function, FunctionId};

    let mut air_module = Module::new(cir.id.clone());

    // Iterate over each contract declaration
    for decl in &cir.decls {
        let contract = match decl {
            crate::cir::CanonDecl::Contract(c) => c,
            crate::cir::CanonDecl::Dialect(_) => continue,
        };

        // Process each member declaration
        for member in &contract.members {
            match member {
                crate::cir::CanonMemberDecl::Function(func_decl) => {
                    // CIR guarantees modifiers are already inlined — no Step 1.
                    let body = &func_decl.body;

                    // Convert CIR body to SIR stmts for CFG construction.
                    // (CFG construction still operates on sir::Stmt internally.)
                    let sir_body = cir_stmts_to_sir(body);

                    // Convert CIR params to SIR params for CFG construction.
                    let sir_params: Vec<crate::sir::Param> = func_decl
                        .params
                        .iter()
                        .map(|p| crate::sir::Param::new(p.name.clone(), p.ty.clone()))
                        .collect();

                    // Determine visibility
                    let is_public = func_decl.attrs.iter().any(|a| {
                        a.namespace == "sir"
                            && a.key == "visibility"
                            && matches!(&a.value, crate::sir::AttrValue::String(s) if s == "public" || s == "external")
                    });

                    let func_id = FunctionId(format!("{}.{}", contract.name, func_decl.name));

                    // Step 1: CFG construction
                    let mut blocks = cfg::build_cfg(&sir_body, &sir_params);

                    // Step 2: SSA renaming
                    ssa::rename_to_ssa(&mut blocks);

                    // Step 3: Dialect lowering
                    dialect_lower::lower_dialect_ops(&mut blocks, &cir.attrs)?;

                    let mut air_func = Function::new(func_id, is_public);
                    air_func.blocks = blocks;
                    air_module.functions.push(air_func);
                }
                _ => { /* StorageDecl, TypeAlias, etc. — not lowered to AIR functions */ }
            }
        }
    }

    // Step 4: ICFG, alias sets, and taint graph initialization
    icfg::build_icfg(&mut air_module);

    Ok(air_module)
}

/// Convert CIR canonical statements back to SIR statements.
///
/// This is a temporary bridge while the CFG construction still operates
/// on `sir::Stmt`. In the future, CFG construction will be updated to
/// work directly on `cir::CanonStmt`.
fn cir_stmts_to_sir(stmts: &[crate::cir::CanonStmt]) -> Vec<crate::sir::Stmt> {
    stmts.iter().map(cir_stmt_to_sir).collect()
}

fn cir_stmt_to_sir(stmt: &crate::cir::CanonStmt) -> crate::sir::Stmt {
    match stmt {
        crate::cir::CanonStmt::LocalVar(s) => {
            crate::sir::Stmt::LocalVar(crate::sir::LocalVarStmt {
                vars: s
                    .vars
                    .iter()
                    .map(|v| {
                        v.as_ref().map(|d| crate::sir::LocalVarDecl {
                            name: d.name.clone(),
                            ty: d.ty.clone(),
                        })
                    })
                    .collect(),
                init: s.init.as_ref().map(cir_expr_to_sir),
                span: s.span,
            })
        }
        crate::cir::CanonStmt::Assign(s) => {
            crate::sir::Stmt::Assign(crate::sir::AssignStmt {
                lhs: cir_expr_to_sir(&s.lhs),
                rhs: cir_expr_to_sir(&s.rhs),
                span: s.span,
            })
        }
        crate::cir::CanonStmt::AugAssign(s) => {
            crate::sir::Stmt::AugAssign(crate::sir::AugAssignStmt {
                op: s.op,
                lhs: cir_expr_to_sir(&s.lhs),
                rhs: cir_expr_to_sir(&s.rhs),
                span: s.span,
            })
        }
        crate::cir::CanonStmt::Expr(s) => crate::sir::Stmt::Expr(crate::sir::ExprStmt {
            expr: cir_expr_to_sir(&s.expr),
            span: s.span,
        }),
        crate::cir::CanonStmt::If(s) => crate::sir::Stmt::If(crate::sir::IfStmt {
            cond: cir_expr_to_sir(&s.cond),
            then_body: cir_stmts_to_sir(&s.then_body),
            else_body: s.else_body.as_ref().map(|stmts| cir_stmts_to_sir(stmts)),
            span: s.span,
        }),
        crate::cir::CanonStmt::While(s) => crate::sir::Stmt::While(crate::sir::WhileStmt {
            cond: cir_expr_to_sir(&s.cond),
            body: cir_stmts_to_sir(&s.body),
            invariant: s.invariant.as_ref().map(cir_expr_to_sir),
            span: s.span,
        }),
        crate::cir::CanonStmt::For(s) => crate::sir::Stmt::For(crate::sir::ForStmt {
            init: s.init.as_ref().map(|stmt| Box::new(cir_stmt_to_sir(stmt))),
            cond: s.cond.as_ref().map(cir_expr_to_sir),
            update: s
                .update
                .as_ref()
                .map(|stmt| Box::new(cir_stmt_to_sir(stmt))),
            body: cir_stmts_to_sir(&s.body),
            invariant: s.invariant.as_ref().map(cir_expr_to_sir),
            span: s.span,
        }),
        crate::cir::CanonStmt::Return(s) => {
            crate::sir::Stmt::Return(crate::sir::ReturnStmt {
                value: s.value.as_ref().map(cir_expr_to_sir),
                span: s.span,
            })
        }
        crate::cir::CanonStmt::Revert(s) => {
            crate::sir::Stmt::Revert(crate::sir::RevertStmt {
                error: s.error.clone(),
                args: s.args.iter().map(cir_expr_to_sir).collect(),
                span: s.span,
            })
        }
        crate::cir::CanonStmt::Assert(s) => {
            crate::sir::Stmt::Assert(crate::sir::AssertStmt {
                cond: cir_expr_to_sir(&s.cond),
                message: s.message.as_ref().map(cir_expr_to_sir),
                span: s.span,
            })
        }
        crate::cir::CanonStmt::Break => crate::sir::Stmt::Break,
        crate::cir::CanonStmt::Continue => crate::sir::Stmt::Continue,
        crate::cir::CanonStmt::Block(stmts) => {
            crate::sir::Stmt::Block(cir_stmts_to_sir(stmts))
        }
        crate::cir::CanonStmt::Dialect(s) => crate::sir::Stmt::Dialect(s.clone()),
    }
}

fn cir_expr_to_sir(expr: &crate::cir::CanonExpr) -> crate::sir::Expr {
    match expr {
        crate::cir::CanonExpr::Var(v) => crate::sir::Expr::Var(crate::sir::VarExpr {
            name: v.name.clone(),
            ty: v.ty.clone(),
            span: v.span,
        }),
        crate::cir::CanonExpr::Lit(l) => crate::sir::Expr::Lit(l.clone()),
        crate::cir::CanonExpr::BinOp(e) => crate::sir::Expr::BinOp(crate::sir::BinOpExpr {
            op: e.op,
            lhs: Box::new(cir_expr_to_sir(&e.lhs)),
            rhs: Box::new(cir_expr_to_sir(&e.rhs)),
            overflow: e.overflow,
            span: e.span,
        }),
        crate::cir::CanonExpr::UnOp(e) => crate::sir::Expr::UnOp(crate::sir::UnOpExpr {
            op: e.op,
            operand: Box::new(cir_expr_to_sir(&e.operand)),
            span: e.span,
        }),
        crate::cir::CanonExpr::IndexAccess(e) => {
            crate::sir::Expr::IndexAccess(crate::sir::IndexAccessExpr {
                base: Box::new(cir_expr_to_sir(&e.base)),
                index: e.index.as_ref().map(|idx| Box::new(cir_expr_to_sir(idx))),
                ty: e.ty.clone(),
                span: e.span,
            })
        }
        crate::cir::CanonExpr::FieldAccess(e) => {
            crate::sir::Expr::FieldAccess(crate::sir::FieldAccessExpr {
                base: Box::new(cir_expr_to_sir(&e.base)),
                field: e.field.clone(),
                ty: e.ty.clone(),
                span: e.span,
            })
        }
        crate::cir::CanonExpr::FunctionCall(e) => {
            crate::sir::Expr::FunctionCall(crate::sir::CallExpr {
                callee: Box::new(cir_expr_to_sir(&e.callee)),
                args: e.args.iter().map(cir_expr_to_sir).collect(),
                ty: e.ty.clone(),
                span: e.span,
            })
        }
        crate::cir::CanonExpr::TypeCast(e) => {
            crate::sir::Expr::TypeCast(crate::sir::TypeCastExpr {
                ty: e.ty.clone(),
                expr: Box::new(cir_expr_to_sir(&e.expr)),
                span: e.span,
            })
        }
        crate::cir::CanonExpr::Old(inner) => {
            crate::sir::Expr::Old(Box::new(cir_expr_to_sir(inner)))
        }
        crate::cir::CanonExpr::Result(idx) => crate::sir::Expr::Result(*idx),
        crate::cir::CanonExpr::Forall { var, ty, body } => crate::sir::Expr::Forall {
            var: var.clone(),
            ty: ty.clone(),
            body: Box::new(cir_expr_to_sir(body)),
        },
        crate::cir::CanonExpr::Exists { var, ty, body } => crate::sir::Expr::Exists {
            var: var.clone(),
            ty: ty.clone(),
            body: Box::new(cir_expr_to_sir(body)),
        },
        crate::cir::CanonExpr::Dialect(d) => crate::sir::Expr::Dialect(d.clone()),
    }
}
