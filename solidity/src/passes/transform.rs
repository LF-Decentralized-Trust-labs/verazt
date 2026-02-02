//! Module to transform the original Solidity AST to Smart Contract IR.

use crate::ast;
use crate::ir::*;
use either::Either::{self, Left, Right};
use extlib::{error::Result, fail};
use log::trace;
use meta::{DataLoc, Loc};
use std::{borrow::BorrowMut, ops::Deref};

pub fn transform_source_unit(source_unit: &ast::SourceUnit) -> Result<SourceUnit> {
    let mut transformer = Transformer::new();
    transformer.transform_source_unit(source_unit)
}

pub struct Transformer {
    tmp_var_index: usize,
}

impl Transformer {
    pub fn new() -> Self {
        Transformer { tmp_var_index: 0 }
    }

    fn create_new_var_decl(&mut self, typ: Type, state_var: bool, loc: Option<Loc>) -> VarDecl {
        // Create a new index for variable name.
        self.tmp_var_index += 1;
        let name = format!("tmp__{}", self.tmp_var_index);

        // Return the variable.
        VarDecl::new(name, typ, None, state_var, Some(DataLoc::Memory), loc)
    }

    fn convert_to_atomic(&mut self, expr: Expr, msg: &str) -> Result<AtomicExpr> {
        match AtomicExpr::try_from(expr) {
            Ok(expr) => Ok(expr),
            Err(err) => fail!(
                "{}: {}.\n{}",
                msg,
                err,
                "Expression must be normalized before transforming to IR"
            ),
        }
    }

    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    fn transform_source_unit(&mut self, source_unit: &ast::SourceUnit) -> Result<SourceUnit> {
        // Create a source unit with empty body first
        let mut nsource_unit = SourceUnit::new(&source_unit.path, vec![]);

        // Transform and update the body
        let mut nbody: Vec<SourceUnitElem> = vec![];
        for elem in source_unit.elems.iter() {
            let nelems = self.transform_source_unit_element(elem)?;
            nbody.extend(nelems)
        }
        nsource_unit.borrow_mut().elems = nbody;

        Ok(nsource_unit)
    }

    fn transform_source_unit_element(
        &mut self,
        elem: &ast::SourceUnitElem,
    ) -> Result<Vec<SourceUnitElem>> {
        match elem {
            ast::SourceUnitElem::Pragma(pragma) => self.transform_pragma(pragma),
            ast::SourceUnitElem::Import(_) => {
                fail!("IR transformation: `import` must be eliminated: {}", elem)
            }
            ast::SourceUnitElem::Using(_) => {
                fail!("IR transformation: `using` must be eliminated: {}", elem)
            }
            ast::SourceUnitElem::Error(error) => match self.transform_error_def(error) {
                Ok(error) => Ok(vec![error.into()]),
                Err(err) => Err(err),
            },
            ast::SourceUnitElem::Var(vdecl) => {
                let (vdecl, stmts) = match self.transform_var_decl(vdecl) {
                    Ok((vdecl, stmts)) => (vdecl, stmts),
                    Err(err) => return Err(err),
                };
                if !stmts.is_empty() {
                    fail!("Handle statement declaring new vars");
                };
                Ok(vec![vdecl.into()])
            }
            ast::SourceUnitElem::UserType(_) => {
                fail!("User-defined type must be eliminated!")
            }
            ast::SourceUnitElem::Struct(struct_) => {
                Ok(vec![self.transform_struct_def(struct_)?.into()])
            }
            ast::SourceUnitElem::Enum(enum_) => Ok(vec![self.transform_enum_def(enum_).into()]),
            ast::SourceUnitElem::Func(func) => match self.transform_func_def(func) {
                Ok(nfunc) => Ok(vec![nfunc.into()]),
                Err(err) => Err(err),
            },
            ast::SourceUnitElem::Contract(contract) => {
                match self.transform_contract_def(contract) {
                    Ok(ncontract) => Ok(vec![ncontract.into()]),
                    Err(err) => Err(err),
                }
            }
        }
    }

    //-------------------------------------------------
    // Pragma directive
    //-------------------------------------------------

    fn transform_pragma(&mut self, pragma: &ast::PragmaDir) -> Result<Vec<SourceUnitElem>> {
        println!("TODO: Transform Pragma {pragma}");
        Ok(vec![])
    }

    //-------------------------------------------------
    // Error and event definitions.
    //-------------------------------------------------

    fn transform_error_def(&mut self, error: &ast::ErrorDef) -> Result<ErrorDef> {
        let name = error.name.to_string();
        let mut nparams = vec![];
        for param in error.params.iter() {
            let (param, _) = self.transform_var_decl(param)?;
            nparams.push(param)
        }
        let nerror = ErrorDef::new(name, nparams, error.loc);
        Ok(nerror)
    }

    fn transform_event_def(&mut self, event: &ast::EventDef) -> Result<EventDef> {
        let name = event.name.to_string();
        let mut nparams = vec![];
        for param in event.params.iter() {
            let (param, _) = self.transform_var_decl(param)?;
            nparams.push(param);
        }
        let nevent = EventDef::new(name, event.is_anonymous, nparams, event.loc);
        Ok(nevent)
    }

    //-------------------------------------------------
    // Type definitions.
    //-------------------------------------------------

    fn transform_struct_def(&mut self, struct_: &ast::StructDef) -> Result<StructDef> {
        let name = struct_.name.to_string();
        let mut fields = vec![];
        for f in struct_.fields.iter() {
            let field = self.transform_struct_field(f)?;
            fields.push(field);
        }
        Ok(StructDef::new(name, fields, struct_.loc))
    }

    fn transform_struct_field(&mut self, field: &ast::StructField) -> Result<StructField> {
        let ftyp = self.transform_type(&field.typ)?;
        Ok(StructField::new(field.name.clone(), ftyp, field.loc))
    }

    fn transform_enum_def(&mut self, enum_: &ast::EnumDef) -> EnumDef {
        EnumDef::new(enum_.name.to_string(), enum_.elems.clone(), enum_.loc)
    }

    //-------------------------------------------------
    // Contract definition.
    //-------------------------------------------------

    fn transform_contract_def(&mut self, contract: &ast::ContractDef) -> Result<ContractDef> {
        trace!("Transform contract: {}", contract.name);

        if !contract.base_contracts.is_empty() {
            fail!("Base contracts must be eliminated before transforming to IR!")
        }

        let kind = match &contract.kind {
            ast::ContractKind::Contract => ContractKind::Contract,
            ast::ContractKind::Interface => ContractKind::Interface,
            ast::ContractKind::Library => fail!("Library must be eliminated!"),
        };

        // Transform its elements with back links to the parent contract
        let mut nbody = vec![];
        for elem in contract.body.iter() {
            let mut nelems = self.transform_contract_element(elem)?;
            nbody.append(&mut nelems)
        }

        Ok(ContractDef::new(contract.name.to_string(), kind, nbody, contract.loc))
    }

    fn transform_contract_element(
        &mut self,
        elem: &ast::ContractElem,
    ) -> Result<Vec<ContractElem>> {
        match elem {
            ast::ContractElem::Using(_) => {
                fail!("Transform contract element: using directive should be eliminated: {}", elem)
            }
            ast::ContractElem::Event(event) => {
                let nevent = self.transform_event_def(event)?;
                Ok(vec![nevent.into()])
            }
            ast::ContractElem::Error(error) => {
                let nerror = self.transform_error_def(error)?;
                Ok(vec![nerror.into()])
            }
            ast::ContractElem::Struct(struct_) => {
                let nstruct = self.transform_struct_def(struct_)?;
                Ok(vec![nstruct.into()])
            }
            ast::ContractElem::Enum(enum_) => {
                let nenum = self.transform_enum_def(enum_);
                Ok(vec![nenum.into()])
            }
            ast::ContractElem::Type(_) => {
                fail!("Transform contract element: user-defined type must be eliminated!")
            }
            ast::ContractElem::Var(vdecl) => {
                let (nvdecl, stmts) = self.transform_var_decl(vdecl)?;
                if !stmts.is_empty() {
                    fail!("Handle statement declaring new vars");
                };
                Ok(vec![nvdecl.into()])
            }
            ast::ContractElem::Func(func) => {
                let nfunc = self.transform_func_def(func)?;
                Ok(vec![nfunc.into()])
            }
        }
    }

    /// Transform call arguments of a modifier invocation or a function call.
    ///
    /// Outputs are the transformed expression and new statements generated
    /// during the transformation.
    fn transform_call_args(
        &mut self,
        arg: &ast::CallArgs,
    ) -> Result<(Vec<AtomicExpr>, Vec<Stmt>)> {
        if let ast::CallArgs::Unnamed(exprs) = arg {
            let mut nstmts = vec![];
            let (nexprs, stmts) = self.transform_exprs(exprs)?;
            nstmts.extend(stmts);

            let mut output_exprs: Vec<AtomicExpr> = vec![];
            for e in nexprs {
                let ne = self.convert_to_atomic(e, "Transform args")?;
                output_exprs.push(ne);
            }
            Ok((output_exprs, nstmts))
        } else {
            fail!("Arguments must be normalized to un-named!")
        }
    }

    //-------------------------------------------------
    // Function
    //-------------------------------------------------

    fn transform_func_def(&mut self, func: &ast::FuncDef) -> Result<FuncDef> {
        trace!("Transform function: {}", func.name);
        let (nparams, _) = self.transform_var_decls(&func.params)?;
        let (nreturns, _) = self.transform_var_decls(&func.returns)?;
        let nbody = match &func.body {
            Some(blk) => Some(self.transform_block(blk)?),
            None => None,
        };
        Ok(FuncDef {
            name: func.name.to_string(),
            is_virtual: func.is_virtual,
            params: nparams,
            returns: nreturns,
            body: nbody,
            loc: func.loc,
        })
    }

    //-------------------------------------------------
    // Block
    //-------------------------------------------------

    /// Transform a block.
    ///
    /// Assumption: variables in the input block are renamed to resolve all
    /// shadowed variables in nested blocks.
    ///
    /// This transformation will flatten all nested blocks.
    fn transform_block(&mut self, blk: &ast::Block) -> Result<Block> {
        // Transform and update body
        let mut nstmts = vec![];
        for stmt in blk.body.iter() {
            match self.transform_stmt(stmt)? {
                Left(mut s) => nstmts.append(&mut s),
                Right(nblk) => {
                    let mut s = nblk.stmts.clone();
                    nstmts.append(&mut s)
                }
            }
        }

        let nblk = Block::new(nstmts, blk.unchecked, blk.loc);
        Ok(nblk)
    }

    //-------------------------------------------------
    // Statement
    //-------------------------------------------------

    fn transform_stmt(&mut self, stmt: &ast::Stmt) -> Result<Either<Vec<Stmt>, Block>> {
        match stmt {
            ast::Stmt::Asm(s) => Ok(Left(self.transform_asm_stmt(s))),
            ast::Stmt::Block(blk) => {
                let nblk = self.transform_block(blk)?;
                Ok(Right(nblk))
            }
            ast::Stmt::Break(s) => Ok(Left(self.transform_break_stmt(s))),
            ast::Stmt::Placeholder(_) => {
                fail!("PlaceholderStmt must be normalized at AST level first!");
            }
            ast::Stmt::Continue(s) => Ok(Left(self.transform_continue_stmt(s))),
            ast::Stmt::DoWhile(s) => {
                let nstmts = self.transform_do_while_stmt(s)?;
                Ok(Left(nstmts))
            }
            ast::Stmt::Emit(s) => {
                let nstmts = self.transform_emit_stmt(s)?;
                Ok(Left(nstmts))
            }
            ast::Stmt::Expr(s) => {
                let nstmts = self.transform_expr_stmt(s)?;
                Ok(Left(nstmts))
            }
            ast::Stmt::For(s) => {
                let nstmts = self.transform_for_stmt(s)?;
                Ok(Left(nstmts))
            }
            ast::Stmt::If(s) => {
                let nstmts = self.transform_if_stmt(s)?;
                Ok(Left(nstmts))
            }
            ast::Stmt::Return(s) => {
                let nstmts = self.transform_return_stmt(s)?;
                Ok(Left(nstmts))
            }
            ast::Stmt::Revert(s) => {
                let nstmts = self.transform_revert_stmt(s)?;
                Ok(Left(nstmts))
            }
            ast::Stmt::Throw(s) => Ok(Left(self.transform_throw_stmt(s))),
            ast::Stmt::Try(s) => {
                let nstmts = self.transform_try_stmt(s)?;
                Ok(Left(nstmts))
            }
            ast::Stmt::VarDecl(s) => {
                let nstmts = self.transform_var_decl_stmt(s)?;
                Ok(Left(nstmts))
            }
            ast::Stmt::While(s) => {
                let nstmts = self.transform_while_stmt(s)?;
                Ok(Left(nstmts))
            }
        }
    }

    //-------------------------------------------------
    // Assembly statement
    //-------------------------------------------------

    fn transform_asm_stmt(&mut self, stmt: &ast::AsmStmt) -> Vec<Stmt> {
        let nstmt =
            AsmStmt::new(stmt.is_evmasm, stmt.asm_flags.clone(), stmt.body.clone(), stmt.loc);
        vec![nstmt.into()]
    }

    //-------------------------------------------------
    // Break statement
    //-------------------------------------------------

    fn transform_break_stmt(&mut self, stmt: &ast::BreakStmt) -> Vec<Stmt> {
        let nstmt = BreakStmt::new(stmt.loc);
        vec![nstmt.into()]
    }

    //-------------------------------------------------
    // Continue statement
    //-------------------------------------------------

    fn transform_continue_stmt(&mut self, stmt: &ast::ContinueStmt) -> Vec<Stmt> {
        let nstmt = ContinueStmt::new(stmt.loc);
        vec![nstmt.into()]
    }

    //-------------------------------------------------
    // Expression statement
    //-------------------------------------------------

    fn transform_expr_stmt(&mut self, stmt: &ast::ExprStmt) -> Result<Vec<Stmt>> {
        let mut nstmts = vec![];
        let (nexpr, mut stmts) = self.transform_expr(&stmt.expr)?;
        nstmts.append(&mut stmts);
        let nstmt = ExprStmt::new(nexpr, stmt.loc).into();
        nstmts.push(nstmt);
        Ok(nstmts)
    }

    //-------------------------------------------------
    // If statement
    //-------------------------------------------------

    /// Transform an `if` statement.
    ///
    /// Output is a list of output statements generated during the
    /// transformation.
    fn transform_if_stmt(&mut self, stmt: &ast::IfStmt) -> Result<Vec<Stmt>> {
        let mut nstmts = vec![];
        let ncond = match self.transform_expr(&stmt.condition) {
            Ok((cond, mut stmts)) => {
                nstmts.append(&mut stmts);
                cond
            }
            Err(err) => return Err(err),
        };
        let ntrue_br = {
            let nstmts = match self.transform_stmt(&stmt.true_branch)? {
                Left(stmts) => stmts,
                Right(blk) => blk.stmts.clone(),
            };
            Block::new(nstmts, stmt.true_branch.is_unchecked_block(), stmt.true_branch.loc())
        };
        let nfalse_br = match &stmt.false_branch {
            Some(s) => {
                let nstmts = match self.transform_stmt(s)? {
                    Left(stmts) => stmts,
                    Right(blk) => blk.stmts.clone(),
                };
                Some(Block::new(nstmts, s.is_unchecked_block(), s.loc()))
            }
            None => None,
        };
        let nstmt = IfStmt::new(ncond, ntrue_br, nfalse_br, stmt.loc);
        nstmts.push(nstmt.into());
        Ok(nstmts)
    }

    //-------------------------------------------------
    // For statement
    //-------------------------------------------------

    fn transform_for_stmt(&mut self, stmt: &ast::ForStmt) -> Result<Vec<Stmt>> {
        let mut nstmts = vec![];
        let ncond = match &stmt.condition {
            Some(expr) => {
                let (cond, stmts) = self.transform_expr(expr)?;
                nstmts.extend(stmts);
                Some(cond)
            }
            None => None,
        };
        let npre = match &stmt.pre_loop {
            Some(s) => {
                let nstmts = match self.transform_stmt(s)? {
                    Left(stmts) => stmts,
                    Right(b) => b.stmts.clone(),
                };
                Some(Block::new(nstmts, s.is_unchecked_block(), s.loc()))
            }
            None => None,
        };
        let npost = match &stmt.post_loop {
            Some(s) => {
                let nstmts = match self.transform_stmt(s)? {
                    Left(stmts) => stmts,
                    Right(b) => b.stmts.clone(),
                };
                Some(Block::new(nstmts, s.is_unchecked_block(), s.loc()))
            }
            None => None,
        };
        let nbody = {
            let nstmts = match self.transform_stmt(&stmt.body)? {
                Either::Left(stmts) => stmts,
                Either::Right(blk) => blk.stmts.clone(),
            };
            Block::new(nstmts, stmt.body.is_unchecked_block(), stmt.body.loc())
        };
        let nstmt = LoopStmt::new(npre, ncond, npost, nbody, stmt.loc);
        nstmts.push(nstmt.into());
        Ok(nstmts)
    }

    //-------------------------------------------------
    // While statement
    //-------------------------------------------------

    fn transform_while_stmt(&mut self, stmt: &ast::WhileStmt) -> Result<Vec<Stmt>> {
        let mut nstmts = vec![];
        let (ncond, mut stmts) = self.transform_expr(&stmt.condition)?;
        nstmts.append(&mut stmts);
        let nbody = {
            let nstmts = match self.transform_stmt(&stmt.body)? {
                Left(stmts) => stmts,
                Right(blk) => blk.stmts.clone(),
            };
            Block::new(nstmts, stmt.body.is_unchecked_block(), stmt.body.loc())
        };
        let nstmt = LoopStmt::new(None, Some(ncond), None, nbody, stmt.loc);
        nstmts.push(nstmt.into());
        Ok(nstmts)
    }

    //-------------------------------------------------
    // Do-While statement
    //-------------------------------------------------

    fn transform_do_while_stmt(&mut self, stmt: &ast::DoWhileStmt) -> Result<Vec<Stmt>> {
        let mut nstmts = vec![];
        let (cond, mut stmts) = self.transform_expr(&stmt.condition)?;
        nstmts.append(&mut stmts);
        let body = {
            let nstmts = match self.transform_stmt(&stmt.body)? {
                Either::Left(stmts) => stmts,
                Either::Right(blk) => blk.stmts.clone(),
            };
            Block::new(nstmts, stmt.body.is_unchecked_block(), stmt.body.loc())
        };
        let nstmt = LoopStmt::new(None, Some(cond), None, body, stmt.loc);
        nstmts.push(nstmt.into());
        Ok(nstmts)
    }

    //-------------------------------------------------
    // Try-catch statement
    //-------------------------------------------------

    fn transform_try_stmt(&mut self, stmt: &ast::TryStmt) -> Result<Vec<Stmt>> {
        let mut nstmts = vec![];
        let (nexpr, mut stmts) = self.transform_expr(&stmt.guarded_expr)?;
        nstmts.append(&mut stmts);
        let (nreturns, mut stmts) = self.transform_var_decls(&stmt.returns)?;
        nstmts.append(&mut stmts);
        let nbody = self.transform_block(&stmt.body)?;
        let mut ncatch_cls: Vec<CatchClause> = vec![];
        for cls in stmt.catch_clauses.iter() {
            let ncls = self.transform_catch_clause(cls)?;
            ncatch_cls.push(ncls);
        }
        let nstmt = TryStmt::new(nexpr, nreturns, nbody, ncatch_cls, stmt.loc);
        nstmts.push(nstmt.into());
        Ok(nstmts)
    }

    fn transform_catch_clause(&mut self, cls: &ast::CatchClause) -> Result<CatchClause> {
        let error = cls.error.clone();
        let (nparams, _) = self.transform_var_decls(&cls.params)?;
        let nbody = self.transform_block(&cls.body)?;
        Ok(CatchClause::new(error, nparams, nbody, cls.loc))
    }

    //-------------------------------------------------
    // Throw statement
    //-------------------------------------------------

    fn transform_throw_stmt(&mut self, stmt: &ast::ThrowStmt) -> Vec<Stmt> {
        let nstmt = ThrowStmt::new(stmt.loc).into();
        vec![nstmt]
    }

    //-------------------------------------------------
    // Revert statement
    //-------------------------------------------------

    fn transform_revert_stmt(&mut self, stmt: &ast::RevertStmt) -> Result<Vec<Stmt>> {
        let mut nstmts = vec![];
        // Find the error definition
        let error = match &stmt.error {
            Some(ast::Expr::Ident(ident)) => Some(ident.name.to_owned()),
            Some(e) => fail!("TODO: transform error: {} in revert stmt: {}", e, stmt),
            None => None,
        };
        let (args, mut stmts) = self.transform_call_args(&stmt.args)?;
        nstmts.append(&mut stmts);
        let nstmt = RevertStmt::new(error, args, stmt.loc).into();
        nstmts.push(nstmt);
        Ok(nstmts)
    }

    //-------------------------------------------------
    // Emit statement
    //-------------------------------------------------

    fn transform_emit_stmt(&mut self, stmt: &ast::EmitStmt) -> Result<Vec<Stmt>> {
        let mut nstmts = vec![];
        // Find the event definition
        let event = match &stmt.event {
            ast::Expr::Ident(ident) => ident.name.to_owned(),
            e => fail!("TODO: transform event: {} in emit stmt: {}", e, stmt),
        };
        let (args, mut stmts) = self.transform_call_args(&stmt.args)?;
        nstmts.append(&mut stmts);
        let nstmt = EmitStmt::new(event, args, stmt.loc).into();
        nstmts.push(nstmt);
        Ok(nstmts)
    }

    //-------------------------------------------------
    // Return statement
    //-------------------------------------------------

    fn transform_return_stmt(&mut self, stmt: &ast::ReturnStmt) -> Result<Vec<Stmt>> {
        let mut nstmts = vec![];
        let nexpr = match stmt.expr {
            Some(ref e) => {
                let (ne, mut stmts) = self.transform_expr(e)?;
                nstmts.append(&mut stmts);
                Some(ne)
            }
            None => None,
        };
        let nstmt = ReturnStmt::new(nexpr, stmt.loc).into();
        nstmts.push(nstmt);
        Ok(nstmts)
    }

    //-------------------------------------------------
    // Variable declaration statement
    //-------------------------------------------------

    fn transform_var_decl_stmt(&mut self, stmt: &ast::VarDeclStmt) -> Result<Vec<Stmt>> {
        let mut nstmts = vec![];
        let mut vdecls: Vec<Option<VarDecl>> = vec![];
        for vopt in stmt.var_decls.iter() {
            match vopt {
                Some(v) => {
                    let (nv, stmts) = self.transform_var_decl(v)?;
                    nstmts.extend(stmts);
                    vdecls.push(Some(nv))
                }
                None => vdecls.push(None),
            }
        }
        let value = match stmt.value {
            Some(ref e) => {
                let (ne, stmts) = self.transform_expr(e)?;
                nstmts.extend(stmts);
                Some(ne)
            }
            None => None,
        };
        let nstmt = VarDeclStmt::new(vdecls, value, stmt.loc).into();
        nstmts.push(nstmt);
        Ok(nstmts)
    }

    //-------------------------------------------------
    // Variable declaration.
    //-------------------------------------------------

    fn transform_var_decl(&mut self, vdecl: &ast::VarDecl) -> Result<(VarDecl, Vec<Stmt>)> {
        if vdecl.overriding.is_some() {
            fail!("Transform var decl: need to elminate override specification: {}", vdecl);
        }
        let mut nstmts = vec![];
        let name = vdecl.name.clone();
        let value = match &vdecl.value {
            Some(e) => {
                let (ne, stmts) = self.transform_expr(e)?;
                nstmts.extend(stmts);
                Some(ne)
            }
            None => None,
        };
        let var = VarDecl::new(
            name.to_string(),
            self.transform_type(&vdecl.typ)?,
            value,
            vdecl.is_state_var,
            vdecl.data_loc,
            vdecl.loc,
        );

        Ok((var, nstmts))
    }

    fn transform_var_decls(
        &mut self,
        vdecl: &[ast::VarDecl],
    ) -> Result<(Vec<VarDecl>, Vec<Stmt>)> {
        let mut nstmts = vec![];
        let mut nvdecls = vec![];
        for v in vdecl.iter() {
            let (nv, stmts) = self.transform_var_decl(v)?;
            nstmts.extend(stmts);
            nvdecls.push(nv);
        }
        Ok((nvdecls, nstmts))
    }

    //-------------------------------------------------
    // Expression.
    //-------------------------------------------------

    /// Transform an expression.
    ///
    /// Return an output expression and a list of intermediate expressions used
    /// to generate that output expression.
    fn transform_expr(&mut self, expr: &ast::Expr) -> Result<(Expr, Vec<Stmt>)> {
        let mut nstmts = vec![];
        let (nexpr, mut stmts) = match expr {
            ast::Expr::Lit(lit) => (self.transform_lit(lit)?.into(), vec![]),
            ast::Expr::Ident(v) => (self.transform_ident(v)?.into(), vec![]),
            ast::Expr::Unary(e) => self.transform_unary_expr(e)?,
            ast::Expr::Binary(e) => self.transform_binary_expr(e)?,
            ast::Expr::Assign(e) => self.transform_assign_expr(e)?,
            ast::Expr::Call(e) => self.transform_call_expr(e)?,
            ast::Expr::CallOpts(_) => fail!("Normalize function call opts before transforming"),
            ast::Expr::Tuple(e) => self.transform_tuple_expr(e)?,
            ast::Expr::Index(e) => self.transform_index_expr(e)?,
            ast::Expr::Slice(e) => self.transform_slice_expr(e)?,
            ast::Expr::Member(e) => self.transform_member_expr(e)?,
            ast::Expr::Conditional(e) => self.transform_conditional_expr(e)?,
            ast::Expr::InlineArray(e) => self.transform_inline_array_expr(e)?,
            ast::Expr::New(e) => {
                let (nexp, stmts) = self.transform_new_expr(e)?;
                (nexp.into(), stmts)
            }
            ast::Expr::TypeName(e) => {
                let (nexp, stmts) = self.transform_type_name_expr(e)?;
                (nexp.into(), stmts)
            }
        };
        nstmts.append(&mut stmts);
        Ok((nexpr, nstmts))
    }

    fn transform_exprs(&mut self, exprs: &[ast::Expr]) -> Result<(Vec<Expr>, Vec<Stmt>)> {
        let mut nstmts = vec![];
        let mut nexprs = vec![];
        for e in exprs.iter() {
            let (ne, mut ss) = self.transform_expr(e)?;
            nstmts.append(&mut ss);
            nexprs.push(ne);
        }
        Ok((nexprs, nstmts))
    }

    //-------------------------------------------------
    // Identifier
    //-------------------------------------------------

    fn transform_ident(&mut self, ident: &ast::Identifier) -> Result<Variable> {
        let typ = self.transform_type(&ident.typ)?;
        Ok(Variable::new(ident.name.to_string(), typ, ident.loc))
    }

    //-------------------------------------------------
    // Unary expression.
    //-------------------------------------------------

    fn transform_unary_expr(&mut self, expr: &ast::UnaryExpr) -> Result<(Expr, Vec<Stmt>)> {
        let mut nstmts = vec![];
        let typ = expr.typ.clone();

        // Transform the operand first
        let (opr, mut stmts) = self.transform_expr(&expr.body)?;
        let opr = self.convert_to_atomic(opr, "Transform unary expr")?;
        nstmts.append(&mut stmts);

        // Handle as ordinary unary expression
        if matches!(
            &expr.op,
            ast::UnaryOp::Not | ast::UnaryOp::Neg | ast::UnaryOp::BitNot | ast::UnaryOp::Delete
        ) {
            todo!("Transform UnaryExpr to Function Call!");
        }

        // Handle as pre/post increment/decrement expression.
        let var = match &opr {
            AtomicExpr::Var(v) => v.clone(),
            _ => fail!("Operand of unary pre-increment must be a variable"),
        };
        let op = &expr.op;
        let oloc = expr.body.loc();
        match &expr.op {
            ast::UnaryOp::PreIncr | ast::UnaryOp::PreDecr => {
                self.normalize_prefix_unary_expr(expr, op, var, oloc)
            }
            ast::UnaryOp::PostIncr | ast::UnaryOp::PostDecr => {
                self.normalize_postfix_unary_expr(expr, op, var, oloc)
            }
            _ => fail!("Expecting pre/post increment/decrement operator"),
        }
    }

    fn normalize_prefix_unary_expr(
        &mut self,
        expr: &ast::UnaryExpr,
        operator: &ast::UnaryOp,
        var: Variable,
        loc: Option<Loc>,
    ) -> Result<(Expr, Vec<Stmt>)> {
        let assign_stmt: Stmt = {
            // let lhs = LValue::from(var.clone());
            // let bin_op = match operator {
            //     ast::UnaryOp::PreIncrement => BinOp::Add,
            //     ast::UnaryOp::PreDecrement => BinOp::Sub,
            //     _ => fail!("Not a prefix unary expression: {}", expr),
            // };
            // let rhs = Expr::from(BinaryExpr::new(
            //     bin_op,
            //     AtomicExpr::from(var.clone()),
            //     AtomicExpr::from(Lit::one(loc)),
            //     var.typ.clone(),
            //     loc,
            // ));
            // let typ = var.typ.clone();
            // let expr = AssignExpr::new(lhs, rhs, typ, loc).into();
            // ExprStmt::new(expr, loc).into()
            todo!("Transform to function call!")
        };
        let output_expr = Expr::from(var);
        let nstmts = vec![assign_stmt];
        Ok((output_expr, nstmts))
    }

    fn normalize_postfix_unary_expr(
        &mut self,
        expr: &ast::UnaryExpr,
        operator: &ast::UnaryOp,
        var: Variable,
        loc: Option<Loc>,
    ) -> Result<(Expr, Vec<Stmt>)> {
        let typ = var.typ.clone();
        let vdecl = self.create_new_var_decl(typ.clone(), false, loc);
        let var_decl_stmt = Stmt::from(VarDeclStmt::new(
            vec![Some(vdecl.clone())],
            Some(Expr::from(var.clone())),
            loc,
        ));
        let assign_stmt: Stmt = {
            // let lhs = LValue::from(var.clone());
            // let bin_op = match operator {
            //     ast::UnaryOp::PostIncrement => BinOp::Add,
            //     ast::UnaryOp::PostDecrement => BinOp::Sub,
            //     _ => fail!("Not a postfix unary expression: {}", expr),
            // };
            // let rhs = Expr::from(BinaryExpr::new(
            //     bin_op,
            //     AtomicExpr::from(var),
            //     AtomicExpr::from(Lit::one(loc)),
            //     typ.clone(),
            //     loc,
            // ));
            // let expr = AssignExpr::new(lhs, rhs, typ.clone(), loc).into();
            // ExprStmt::new(expr, loc).into()
            todo!("Transform to function call!")
        };
        let nstmts = vec![var_decl_stmt, assign_stmt];
        let nvar = Variable::new(vdecl.name, typ, loc);
        let output_expr = Expr::from(nvar);
        Ok((output_expr, nstmts))
    }

    //-------------------------------------------------
    // Binary expression.
    //-------------------------------------------------

    fn transform_binary_expr(&mut self, expr: &ast::BinaryExpr) -> Result<(Expr, Vec<Stmt>)> {
        let mut nstmts = vec![];
        // let op = self.transform_binary_operator(&expr.operator);
        let typ = expr.typ.clone();

        let (lhs, mut stmts) = self.transform_expr(&expr.left)?;
        let nlhs = self.convert_to_atomic(lhs, "Transform binary expr")?;
        nstmts.append(&mut stmts);

        let (rhs, mut stmts) = self.transform_expr(&expr.right)?;
        let nrhs = self.convert_to_atomic(rhs, "Transform binary expr")?;
        nstmts.append(&mut stmts);

        // let output_expr = BinaryExpr::new(op, nlhs, nrhs, typ, expr.loc).into();
        // Ok((output_expr, nstmts))

        todo!("Transform to function call!");
    }

    //-------------------------------------------------
    // Assignment expression.
    //-------------------------------------------------

    fn transform_assign_expr(&mut self, expr: &ast::AssignExpr) -> Result<(Expr, Vec<Stmt>)> {
        let typ = expr.typ.clone();

        let mut nstmts = vec![];
        let (nlhs, mut stmts) = self.transform_expr(&expr.left)?;
        nstmts.append(&mut stmts);
        let (nrhs, mut stmts) = self.transform_expr(&expr.right)?;
        nstmts.append(&mut stmts);

        // Transform self-assignment into a normal assignment
        let nrhs = match expr.operator {
            ast::AssignOp::Assign => nrhs,
            _ => fail!("Transform to IR: operator must be normalized: {}", expr.operator),
        };
        todo!("Transform assign expr!")
    }

    //-------------------------------------------------
    // Function call expression.
    //-------------------------------------------------

    /// Transform a function call expression.
    ///
    /// Return the output function call and a list of additionally generated
    /// statements.
    fn transform_call_expr(&mut self, expr: &ast::CallExpr) -> Result<(Expr, Vec<Stmt>)> {
        let mut nstmts = vec![];
        let typ = self.transform_type(&expr.typ)?;
        let mut call_opts = vec![];

        let callee = match expr.callee.deref() {
            ast::Expr::New(e) => {
                let (callee, mut stmts) = self.transform_new_expr(e)?;
                nstmts.append(&mut stmts);
                CalleeExpr::from(callee)
            }
            ast::Expr::TypeName(e) => {
                let (callee, mut stmts) = self.transform_type_name_expr(e)?;
                nstmts.append(&mut stmts);
                CalleeExpr::from(callee)
            }
            ast::Expr::Member(e) => {
                let (nexpr, mut stmts) = self.transform_member_expr(e)?;
                nstmts.append(&mut stmts);
                match CalleeExpr::try_from(nexpr) {
                    Ok(exp) => exp,
                    Err(err) => fail!("{}", err),
                }
            }
            ast::Expr::CallOpts(e) => {
                let (callee, mut stmts) = self.transform_expr(&e.callee)?;
                nstmts.append(&mut stmts);
                for call_opt in &e.call_opts {
                    let (call_opt, mut stmts) = self.transform_call_opt(call_opt)?;
                    call_opts.push(call_opt);
                    nstmts.append(&mut stmts);
                }
                match callee {
                    Expr::Member(e) => CalleeExpr::from(e),
                    Expr::New(e) => CalleeExpr::from(e),
                    Expr::TypeName(e) => CalleeExpr::from(e),
                    _ => fail!("Transform function call option: {}", &callee),
                }
            }
            _ => {
                // Default, transform to function call by its name
                CalleeExpr::from(expr.callee.to_string().as_str())
            }
        };

        let (args, mut stmts) = self.transform_call_args(&expr.args)?;
        nstmts.append(&mut stmts);

        let nexpr = CallExpr::new(callee, call_opts, args, typ, expr.loc).into();
        Ok((nexpr, nstmts))
    }

    //-------------------------------------------------
    // Callee option
    //-------------------------------------------------

    fn transform_call_opt(&mut self, call: &ast::CallOpt) -> Result<(CallOption, Vec<Stmt>)> {
        let (nvalue, nstmts) = self.transform_expr(&call.value)?;
        let ncall = CallOption::new(call.name.clone(), nvalue, call.loc);
        Ok((ncall, nstmts))
    }

    //-------------------------------------------------
    // Tuple expression
    //-------------------------------------------------

    fn transform_tuple_expr(&mut self, expr: &ast::TupleExpr) -> Result<(Expr, Vec<Stmt>)> {
        let mut nstmts = vec![];
        let typ = self.transform_type(&expr.typ)?;
        let mut elems = vec![];
        for elem in expr.elems.iter() {
            match elem {
                Some(exp) => {
                    let (nexp, mut stmts) = self.transform_expr(exp)?;
                    let nexp = self.convert_to_atomic(nexp, "Transform tuple expr")?;
                    nstmts.append(&mut stmts);
                    elems.push(Some(nexp));
                }
                None => elems.push(None),
            }
        }

        let nexpr = TupleExpr::new(elems, typ, expr.loc).into();
        Ok((nexpr, nstmts))
    }

    //-------------------------------------------------
    // Index access expression
    //-------------------------------------------------

    fn transform_index_expr(&mut self, expr: &ast::IndexExpr) -> Result<(Expr, Vec<Stmt>)> {
        let mut nstmts = vec![];
        let typ = self.transform_type(&expr.typ)?;
        let (base, mut stmts) = self.transform_expr(&expr.base_expr)?;
        nstmts.append(&mut stmts);
        let nexpr: Expr = match base {
            Expr::Var(v) => {
                let index = match &expr.index {
                    Some(idx) => {
                        let (idx, mut stmts) = self.transform_expr(idx)?;
                        nstmts.append(&mut stmts);
                        let idx = self.convert_to_atomic(idx, "Transform index access expr")?;
                        Some(idx)
                    }
                    None => None,
                };
                IndexExpr::new(v, index, typ, expr.loc).into()
            }
            Expr::TypeName(e) => {
                if expr.index.is_none() {
                    let data_loc = typ.data_loc();
                    let ntyp = ArrayType::new(e.typ, None, data_loc, false);
                    TypeNameExpr::new(ntyp.into(), expr.loc).into()
                } else {
                    fail!("Transform to IR: do not expect index in TypeNameExpr: {}", expr)
                }
            }
            _ => fail!("Transform to IR: handle IndexExpr: {}", expr),
        };
        Ok((nexpr, nstmts))
    }

    //-------------------------------------------------
    // Slice expression
    //-------------------------------------------------

    fn transform_slice_expr(&mut self, expr: &ast::SliceExpr) -> Result<(Expr, Vec<Stmt>)> {
        let mut nstmts = vec![];
        let typ = self.transform_type(&expr.typ)?;
        let nbase = {
            let (base, mut stmts) = self.transform_expr(&expr.base_expr)?;
            nstmts.append(&mut stmts);
            match base {
                Expr::Var(v) => v,
                _ => fail!("Base of range access expr of must be a variable!"),
            }
        };
        let nstart = match &expr.start_index {
            Some(idx) => {
                let (idx, stmts) = self.transform_expr(idx)?;
                let idx = self.convert_to_atomic(idx, "Transform range access expr")?;
                nstmts.extend(stmts);
                Some(idx)
            }
            None => None,
        };
        let nend = match &expr.end_index {
            Some(idx) => {
                let (idx, stmts) = self.transform_expr(idx)?;
                let idx = self.convert_to_atomic(idx, "Transform range access expr")?;
                nstmts.extend(stmts);
                Some(idx)
            }
            None => None,
        };
        let nexpr = SliceExpr::new(nbase, nstart, nend, typ, expr.loc).into();
        Ok((nexpr, nstmts))
    }

    //-------------------------------------------------
    // Member access expression
    //-------------------------------------------------

    /// Transform a member access expression.
    ///
    /// Outputs are the transformed expression and new statements generated
    /// during the transformation.
    fn transform_member_expr(&mut self, expr: &ast::MemberExpr) -> Result<(Expr, Vec<Stmt>)> {
        let mut nstmts = vec![];
        let member = expr.member.to_string();
        let typ = self.transform_type(&expr.typ)?;
        let nexpr = match &*expr.base {
            ast::Expr::Call(call_expr) if call_expr.callee.to_string().eq("type") => {
                // Transform type information query to predefined function calls
                let fname = match &call_expr.args {
                    ast::CallArgs::Unnamed(args) => member + "__type__" + &args[0].to_string(),
                    ast::CallArgs::Named(_) => {
                        fail!("Transform member access expr: {}", expr)
                    }
                };
                CallExpr::new(CalleeExpr::from(fname.as_str()), vec![], vec![], typ, expr.loc)
                    .into()
            }
            _ => {
                let (base, mut stmts) = self.transform_expr(&expr.base)?;
                nstmts.append(&mut stmts);
                MemberExpr::new(Box::new(base), member, typ, expr.loc).into()
            }
        };
        Ok((nexpr, nstmts))
    }

    //-------------------------------------------------
    // Conditional expression
    //-------------------------------------------------

    /// Transform a conditional expression.
    ///
    /// Outputs are the transformed expression and new statements generated
    /// during the transformation.
    fn transform_conditional_expr(
        &mut self,
        expr: &ast::ConditionalExpr,
    ) -> Result<(Expr, Vec<Stmt>)> {
        let mut nstmts = vec![];
        let typ = self.transform_type(&expr.typ)?;
        let ncond = {
            let (ncond, mut stmts) = self.transform_expr(&expr.cond)?;
            nstmts.append(&mut stmts);
            self.convert_to_atomic(ncond, "Transform condition expr")?
        };
        let nopr1 = {
            let (nopr1, mut stmts) = self.transform_expr(&expr.true_br)?;
            nstmts.append(&mut stmts);
            self.convert_to_atomic(nopr1, "Transform condition expr")?
        };
        let nopr2 = {
            let (nopr2, mut stmts) = self.transform_expr(&expr.false_br)?;
            nstmts.append(&mut stmts);
            self.convert_to_atomic(nopr2, "Transform condition expr")?
        };
        todo!("Transform to If statement")
    }

    //-------------------------------------------------
    // Inline array expression
    //-------------------------------------------------

    /// Transform an inline array expression.
    ///
    /// Outputs are the transformed expression and new statements generated
    /// during the transformation.
    fn transform_inline_array_expr(
        &mut self,
        expr: &ast::InlineArrayExpr,
    ) -> Result<(Expr, Vec<Stmt>)> {
        let typ = self.transform_type(&expr.typ)?;
        let mut nstmts = vec![];
        let (elems, mut stmts) = self.transform_exprs(&expr.elems)?;
        nstmts.append(&mut stmts);
        let mut nelems = vec![];
        for e in elems.iter() {
            let ne = self.convert_to_atomic(e.clone(), "Transform inline array")?;
            nelems.push(ne);
        }
        let nexpr = InlineArrayExpr::new(nelems, typ, expr.loc).into();
        Ok((nexpr, nstmts))
    }

    //-------------------------------------------------
    // New expression
    //-------------------------------------------------

    /// Transform a new expression.
    ///
    /// Outputs are the transformed expression and new statements generated
    /// during the transformation.
    fn transform_new_expr(&mut self, expr: &ast::NewExpr) -> Result<(NewExpr, Vec<Stmt>)> {
        let typ = self.transform_type(&expr.typ)?;
        Ok((NewExpr::new(typ, expr.loc), vec![]))
    }

    //-------------------------------------------------
    // Elementary type name expression
    //-------------------------------------------------

    /// Transform an elementary type name expression.
    fn transform_type_name_expr(
        &mut self,
        expr: &ast::TypeNameExpr,
    ) -> Result<(TypeNameExpr, Vec<Stmt>)> {
        let typ = self.transform_type(&expr.typ)?;
        Ok((TypeNameExpr::new(typ, expr.loc), vec![]))
    }

    //-------------------------------------------------
    // Types
    //-------------------------------------------------

    fn transform_type(&mut self, typ: &ast::Type) -> Result<Type> {
        match typ {
            ast::Type::Bool => Ok(Type::Bool),
            ast::Type::Int(t) => Ok(self.transform_int_type(t).into()),
            ast::Type::Fixed(t) => Ok(self.transform_fixed_type(t).into()),
            ast::Type::String(t) => Ok(self.transform_string_type(t).into()),
            ast::Type::Address(t) => Ok(self.transform_address_type(t).into()),
            ast::Type::Bytes(t) => Ok(self.transform_bytes_type(t).into()),
            ast::Type::Array(t) => Ok(self.transform_array_type(t)?.into()),
            ast::Type::Slice(t) => Ok(self.transform_slice_type(t)?.into()),
            ast::Type::Struct(t) => Ok(self.transform_struct_type(t)?.into()),
            ast::Type::Enum(t) => Ok(self.transform_enum_type(t)?.into()),
            ast::Type::Module(name) => Ok(Type::Module(name.clone())),
            ast::Type::Tuple(t) => Ok(self.transform_tuple_type(t)?.into()),
            ast::Type::Func(t) => Ok(self.transform_function_type(t)?.into()),
            ast::Type::Mapping(t) => Ok(self.transform_mapping_type(t)?.into()),
            ast::Type::UserDefined(_) => fail!("User-defined type must be eliminated!"),
            ast::Type::Contract(t) => Ok(self.transform_contract_type(t)?.into()),
            ast::Type::Magic(t) => Ok(self.transform_magic_type(t)?.into()),
        }
    }

    fn transform_int_type(&mut self, t: &ast::IntType) -> IntType {
        IntType::new(t.bitwidth, t.is_signed)
    }

    fn transform_fixed_type(&mut self, t: &ast::FixedType) -> FixedType {
        FixedType::new(t.is_signed)
    }

    fn transform_string_type(&mut self, t: &ast::StringType) -> StringType {
        StringType::new(t.data_loc, t.is_ptr)
    }

    fn transform_address_type(&mut self, t: &ast::AddressType) -> AddressType {
        AddressType::new(t.payable)
    }

    fn transform_bytes_type(&mut self, t: &ast::BytesType) -> BytesType {
        BytesType::new(t.length, t.data_loc, t.is_ptr)
    }

    fn transform_array_type(&mut self, t: &ast::ArrayType) -> Result<ArrayType> {
        let base = self.transform_type(&t.base)?;
        Ok(ArrayType::new(base, t.length.clone(), t.data_loc, t.is_ptr))
    }

    fn transform_slice_type(&mut self, t: &ast::SliceType) -> Result<SliceType> {
        let base = self.transform_type(&t.base)?;
        Ok(SliceType::new(base))
    }

    fn transform_struct_type(&mut self, t: &ast::StructType) -> Result<StructType> {
        if t.scope.is_some() {
            fail!("Struct scope must be eliminated!")
        }
        Ok(StructType::new(t.name.clone(), t.data_loc, t.is_ptr))
    }

    fn transform_enum_type(&mut self, t: &ast::EnumType) -> Result<EnumType> {
        if t.scope.is_some() {
            fail!("Enum scope must be eliminated!")
        }
        Ok(EnumType::new(t.name.clone()))
    }

    fn transform_tuple_type(&mut self, t: &ast::TupleType) -> Result<TupleType> {
        let mut elems = vec![];
        for elem in t.elems.iter() {
            match elem {
                Some(typ) => {
                    let ntyp = self.transform_type(typ)?;
                    elems.push(Some(Box::new(ntyp)));
                }
                None => elems.push(None),
            }
        }
        Ok(TupleType::new(elems))
    }

    fn transform_function_type(&mut self, t: &ast::FuncType) -> Result<FunctionType> {
        let mut params = vec![];
        for param in t.params.iter() {
            let nparam = self.transform_type(param)?;
            params.push(nparam);
        }
        let mut returns = vec![];
        for ret in t.returns.iter() {
            let nret = self.transform_type(ret)?;
            returns.push(nret);
        }
        Ok(FunctionType::new(params, returns))
    }

    fn transform_mapping_type(&mut self, t: &ast::MappingType) -> Result<MappingType> {
        let key = self.transform_type(&t.key)?;
        let value = self.transform_type(&t.value)?;
        Ok(MappingType::new(key, value, t.data_loc))
    }

    fn transform_contract_type(&mut self, t: &ast::ContractType) -> Result<ContractType> {
        if t.scope.is_some() {
            fail!("Contract scope must be eliminated!")
        }
        Ok(ContractType::new(t.name.clone(), t.is_lib))
    }

    fn transform_magic_type(&mut self, t: &ast::MagicType) -> Result<MagicType> {
        match t {
            ast::MagicType::BlockType => Ok(MagicType::BlockType),
            ast::MagicType::MessageType => Ok(MagicType::MessageType),
            ast::MagicType::TxnType => Ok(MagicType::TxnType),
            ast::MagicType::ABIType => Ok(MagicType::ABIType),
            ast::MagicType::MetaType(base) => {
                let nbase = self.transform_type(base)?;
                Ok(MagicType::MetaType(Box::new(nbase)))
            }
        }
    }

    //-------------------------------------------------
    // Literal
    //-------------------------------------------------

    fn transform_lit(&mut self, lit: &ast::Lit) -> Result<Lit> {
        match lit {
            ast::Lit::Bool(lit) => Ok(self.transform_bool_lit(lit).into()),
            ast::Lit::Num(lit) => Ok(self.transform_num_lit(lit)?.into()),
            ast::Lit::String(lit) => Ok(self.transform_string_lit(lit).into()),
            ast::Lit::Hex(lit) => Ok(self.transform_hex_lit(lit).into()),
            ast::Lit::Unicode(lit) => Ok(self.transform_unicode_lit(lit).into()),
        }
    }

    fn transform_bool_lit(&mut self, lit: &ast::BoolLit) -> BoolLit {
        BoolLit::new(lit.value, lit.loc)
    }

    fn transform_num_lit(&mut self, lit: &ast::NumLit) -> Result<NumLit> {
        if lit.unit.is_some() {
            fail!("Transform number literal: unit must be eliminated!")
        }
        let value = self.transform_number(&lit.value)?;
        Ok(NumLit::new(value, lit.loc))
    }

    fn transform_number(&mut self, num: &ast::Num) -> Result<Num> {
        match num {
            ast::Num::Int(n) => Ok(self.transform_int_number(n)?.into()),
            ast::Num::FixedNum(n) => Ok(self.transform_fix_number(n)?.into()),
            ast::Num::Hex(n) => Ok(self.transform_hex_number(n)?.into()),
        }
    }

    fn transform_int_number(&mut self, num: &ast::IntNum) -> Result<IntNum> {
        let typ = self.transform_type(&num.typ)?;
        Ok(IntNum::new(num.value.clone(), typ))
    }

    fn transform_fix_number(&mut self, num: &ast::FixedNum) -> Result<FixedNum> {
        let typ = self.transform_type(&num.typ)?;
        Ok(FixedNum::new(num.value, typ))
    }

    fn transform_hex_number(&mut self, num: &ast::HexNum) -> Result<HexNum> {
        let typ = self.transform_type(&num.typ)?;
        Ok(HexNum::new(num.value.clone(), typ))
    }

    fn transform_string_lit(&mut self, lit: &ast::StringLit) -> StringLit {
        StringLit::new(lit.value.clone(), lit.loc)
    }

    fn transform_hex_lit(&mut self, lit: &ast::HexLit) -> HexLit {
        HexLit::new(lit.value.clone(), lit.loc)
    }

    fn transform_unicode_lit(&mut self, lit: &ast::UnicodeLit) -> UnicodeLit {
        UnicodeLit::new(lit.value.clone(), lit.loc)
    }
}

impl Default for Transformer {
    fn default() -> Self {
        Self::new()
    }
}
