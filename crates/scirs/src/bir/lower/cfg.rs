//! Step 2: CFG Construction
//!
//! Converts structured SIR statements into basic blocks with explicit
//! control flow edges (terminators).

use crate::bir::cfg::{BasicBlock, BlockId, Terminator};
use crate::bir::ops::{Op, OpId, OpKind, OpRef, SsaName};
use crate::sir::{Expr, Param, Stmt, Type};

/// State for the CFG builder.
struct CfgBuilder {
    blocks: Vec<BasicBlock>,
    next_block_id: usize,
    next_op_id: usize,
}

impl CfgBuilder {
    fn new() -> Self {
        CfgBuilder { blocks: Vec::new(), next_block_id: 0, next_op_id: 0 }
    }

    fn new_block(&mut self) -> BlockId {
        let id = BlockId(self.next_block_id);
        self.next_block_id += 1;
        self.blocks.push(BasicBlock::new(id));
        id
    }

    fn new_op_id(&mut self) -> OpId {
        let id = OpId(self.next_op_id);
        self.next_op_id += 1;
        id
    }

    fn block_mut(&mut self, id: BlockId) -> &mut BasicBlock {
        &mut self.blocks[id.0]
    }

    fn append_op(&mut self, block: BlockId, op: Op) -> OpRef {
        let op_ref = OpRef(op.id);
        self.block_mut(block).ops.push(op);
        op_ref
    }

    fn set_terminator(&mut self, block: BlockId, term: Terminator) {
        self.block_mut(block).term = term;
    }
}

/// Build a CFG from a list of SIR statements (function body).
pub fn build_cfg(stmts: &[Stmt], params: &[Param]) -> Vec<BasicBlock> {
    let mut builder = CfgBuilder::new();
    let entry = builder.new_block();

    // Create parameter ops
    for (i, param) in params.iter().enumerate() {
        let op_id = builder.new_op_id();
        let op = Op::new(op_id, OpKind::Param { index: i })
            .with_result(SsaName::new(&param.name, 0), param.ty.clone());
        builder.append_op(entry, op);
    }

    // Flatten the statement list into basic blocks
    let exit = flatten_stmts(&mut builder, stmts, entry);

    // If the exit block doesn't have a terminator, add TxnExit
    if builder.block_mut(exit).term == Terminator::Unreachable {
        builder.set_terminator(exit, Terminator::TxnExit { reverted: false });
    }

    builder.blocks
}

/// Flatten a list of statements into basic blocks, returning the exit block.
fn flatten_stmts(builder: &mut CfgBuilder, stmts: &[Stmt], mut current: BlockId) -> BlockId {
    for stmt in stmts {
        current = flatten_stmt(builder, stmt, current);
    }
    current
}

/// Flatten a single statement, returning the block to continue from.
fn flatten_stmt(builder: &mut CfgBuilder, stmt: &Stmt, current: BlockId) -> BlockId {
    match stmt {
        Stmt::LocalVar(local_var) => {
            let op_id = builder.new_op_id();
            // Determine variable name and type
            let (name, ty) = if let Some(Some(decl)) = local_var.vars.first() {
                (decl.name.clone(), decl.ty.clone())
            } else {
                ("_tmp".to_string(), Type::None)
            };

            let kind = if let Some(init) = &local_var.init {
                lower_expr_to_opkind(builder, current, init)
            } else {
                OpKind::Const(crate::sir::Lit::Bool(crate::sir::BoolLit::new(false, None)))
            };

            let op = Op::new(op_id, kind).with_result(SsaName::new(&name, 0), ty);
            if let Some(span) = local_var.span {
                let op = op.with_span(span);
                builder.append_op(current, op);
            } else {
                builder.append_op(current, op);
            }
            current
        }

        Stmt::Assign(assign) => {
            let op_id = builder.new_op_id();
            let kind = lower_expr_to_opkind(builder, current, &assign.rhs);
            let name = expr_name(&assign.lhs);
            let ty = assign.rhs.typ();
            let mut op = Op::new(op_id, kind).with_result(SsaName::new(&name, 0), ty);
            if let Some(span) = assign.span {
                op = op.with_span(span);
            }
            builder.append_op(current, op);
            current
        }

        Stmt::AugAssign(aug) => {
            let op_id = builder.new_op_id();
            // Create a BinOp with the LHS and RHS
            let lhs_ref = lower_expr(builder, current, &aug.lhs);
            let rhs_ref = lower_expr(builder, current, &aug.rhs);
            let kind = OpKind::BinOp {
                op: aug.op,
                lhs: lhs_ref,
                rhs: rhs_ref,
                overflow: crate::sir::OverflowSemantics::Checked,
            };
            let name = expr_name(&aug.lhs);
            let mut op = Op::new(op_id, kind).with_result(SsaName::new(&name, 0), aug.lhs.typ());
            if let Some(span) = aug.span {
                op = op.with_span(span);
            }
            builder.append_op(current, op);
            current
        }

        Stmt::Expr(expr_stmt) => {
            let expr_ref = lower_expr(builder, current, &expr_stmt.expr);
            let op_id = builder.new_op_id();
            let mut op = Op::new(op_id, OpKind::ExprStmt { expr: expr_ref });
            if let Some(span) = expr_stmt.span {
                op = op.with_span(span);
            }
            builder.append_op(current, op);
            current
        }

        Stmt::If(if_stmt) => {
            let cond_ref = lower_expr(builder, current, &if_stmt.cond);

            let then_block = builder.new_block();
            let merge_block = builder.new_block();

            let then_exit = flatten_stmts(builder, &if_stmt.then_body, then_block);
            if builder.block_mut(then_exit).term == Terminator::Unreachable {
                builder.set_terminator(then_exit, Terminator::Jump(merge_block));
            }

            if let Some(else_body) = &if_stmt.else_body {
                let else_block = builder.new_block();
                let else_exit = flatten_stmts(builder, else_body, else_block);
                if builder.block_mut(else_exit).term == Terminator::Unreachable {
                    builder.set_terminator(else_exit, Terminator::Jump(merge_block));
                }
                builder.set_terminator(
                    current,
                    Terminator::Branch {
                        cond: cond_ref,
                        then_bb: then_block,
                        else_bb: else_block,
                    },
                );
            } else {
                builder.set_terminator(
                    current,
                    Terminator::Branch {
                        cond: cond_ref,
                        then_bb: then_block,
                        else_bb: merge_block,
                    },
                );
            }

            merge_block
        }

        Stmt::While(while_stmt) => {
            let header = builder.new_block();
            let body_block = builder.new_block();
            let after_block = builder.new_block();

            // Jump from current to header
            builder.set_terminator(current, Terminator::Jump(header));

            // Header: evaluate condition
            let cond_ref = lower_expr(builder, header, &while_stmt.cond);
            builder.set_terminator(
                header,
                Terminator::Branch { cond: cond_ref, then_bb: body_block, else_bb: after_block },
            );

            // Body
            let body_exit = flatten_stmts(builder, &while_stmt.body, body_block);
            if builder.block_mut(body_exit).term == Terminator::Unreachable {
                builder.set_terminator(body_exit, Terminator::Jump(header));
            }

            after_block
        }

        Stmt::For(for_stmt) => {
            // Lower for-loop as: init; while(cond) { body; update; }
            let init_exit = if let Some(init) = &for_stmt.init {
                flatten_stmt(builder, init, current)
            } else {
                current
            };

            let header = builder.new_block();
            let body_block = builder.new_block();
            let after_block = builder.new_block();

            builder.set_terminator(init_exit, Terminator::Jump(header));

            // Condition
            if let Some(cond) = &for_stmt.cond {
                let cond_ref = lower_expr(builder, header, cond);
                builder.set_terminator(
                    header,
                    Terminator::Branch {
                        cond: cond_ref,
                        then_bb: body_block,
                        else_bb: after_block,
                    },
                );
            } else {
                builder.set_terminator(header, Terminator::Jump(body_block));
            }

            // Body + update
            let body_exit = flatten_stmts(builder, &for_stmt.body, body_block);
            let update_exit = if let Some(update) = &for_stmt.update {
                flatten_stmt(builder, update, body_exit)
            } else {
                body_exit
            };
            if builder.block_mut(update_exit).term == Terminator::Unreachable {
                builder.set_terminator(update_exit, Terminator::Jump(header));
            }

            after_block
        }

        Stmt::Return(ret) => {
            if let Some(value) = &ret.value {
                let val_ref = lower_expr(builder, current, value);
                let op_id = builder.new_op_id();
                let mut op = Op::new(op_id, OpKind::Return(vec![val_ref]));
                if let Some(span) = ret.span {
                    op = op.with_span(span);
                }
                builder.append_op(current, op);
            } else {
                let op_id = builder.new_op_id();
                let mut op = Op::new(op_id, OpKind::Return(vec![]));
                if let Some(span) = ret.span {
                    op = op.with_span(span);
                }
                builder.append_op(current, op);
            }
            builder.set_terminator(current, Terminator::TxnExit { reverted: false });
            // After return, create an unreachable block for subsequent stmts
            builder.new_block()
        }

        Stmt::Revert(_revert) => {
            builder.set_terminator(current, Terminator::TxnExit { reverted: true });
            builder.new_block()
        }

        Stmt::Assert(assert_stmt) => {
            let cond_ref = lower_expr(builder, current, &assert_stmt.cond);
            let op_id = builder.new_op_id();
            let mut op = Op::new(op_id, OpKind::Assert { cond: cond_ref });
            if let Some(span) = assert_stmt.span {
                op = op.with_span(span);
            }
            builder.append_op(current, op);
            current
        }

        Stmt::Break | Stmt::Continue => {
            // Break/continue are handled as jumps to loop exit/header.
            // For simplicity, they are treated as opaque stmts.
            current
        }

        Stmt::Block(stmts) => flatten_stmts(builder, stmts, current),

        Stmt::Dialect(_dialect_stmt) => {
            // Dialect statements are retained as opaque ops.
            // They will be lowered in Step 4.
            let op_id = builder.new_op_id();
            let op = Op::new(op_id, OpKind::Opaque { description: format!("{stmt}") });
            builder.append_op(current, op);
            current
        }
    }
}

/// Lower an expression to an OpRef, creating intermediate ops as needed.
fn lower_expr(builder: &mut CfgBuilder, block: BlockId, expr: &Expr) -> OpRef {
    let op_id = builder.new_op_id();
    let kind = lower_expr_to_opkind(builder, block, expr);
    let ty = expr.typ();
    let name = expr_name(expr);
    let mut op = Op::new(op_id, kind).with_result(SsaName::new(&name, 0), ty);
    if let Some(span) = expr.span() {
        op = op.with_span(span);
    }
    builder.append_op(block, op)
}

/// Lower an expression to an OpKind.
fn lower_expr_to_opkind(builder: &mut CfgBuilder, block: BlockId, expr: &Expr) -> OpKind {
    match expr {
        Expr::Lit(lit) => OpKind::Const(lit.clone()),
        Expr::Var(_) => {
            // Variables reference the current SSA name — handled by Step 3
            OpKind::PseudoValue {
                label: expr.to_string(),
                taint: crate::bir::interfaces::TaintLabel::Clean,
            }
        }
        Expr::BinOp(binop) => {
            let lhs = lower_expr(builder, block, &binop.lhs);
            let rhs = lower_expr(builder, block, &binop.rhs);
            OpKind::BinOp { op: binop.op, lhs, rhs, overflow: binop.overflow }
        }
        Expr::UnOp(unop) => {
            let operand = lower_expr(builder, block, &unop.operand);
            OpKind::UnOp { op: unop.op, operand }
        }
        Expr::FunctionCall(call) => {
            // Function calls are treated as opaque until Step 4
            let _args: Vec<OpRef> = call
                .args
                .exprs()
                .into_iter()
                .map(|a| lower_expr(builder, block, a))
                .collect();
            OpKind::Opaque { description: format!("{expr}") }
        }
        Expr::IndexAccess(_) | Expr::FieldAccess(_) => {
            // These may become storage ops in Step 4
            OpKind::Opaque { description: format!("{expr}") }
        }
        Expr::Dialect(_) => {
            // Dialect expressions are retained for Step 4 lowering
            OpKind::Opaque { description: format!("{expr}") }
        }
        _ => OpKind::Opaque { description: format!("{expr}") },
    }
}

/// Extract a name from an expression (for SSA naming).
fn expr_name(expr: &Expr) -> String {
    match expr {
        Expr::Var(v) => v.name.clone(),
        Expr::IndexAccess(idx) => format!("{}_idx", expr_name(&idx.base)),
        Expr::FieldAccess(field) => format!("{}_{}", expr_name(&field.base), field.field),
        _ => "_tmp".to_string(),
    }
}
