use crate::{ast::utils::*, ast::*};
use meta::{DataLoc, Loc, NamingEnv};
use std::borrow::Borrow;

struct ExprFlattener {
    env: NamingEnv,
    current_mutability: Option<VarMut>,
}

impl ExprFlattener {
    fn new(env: Option<&NamingEnv>) -> Self {
        let env = match env {
            Some(env) => env.clone(),
            None => NamingEnv::new(),
        };
        ExprFlattener { env, current_mutability: None }
    }

    fn create_inter_var_decl(
        &mut self,
        typ: Type,
        state_var: bool,
        loc: Option<Loc>,
        value: Expr,
    ) -> VarDecl {
        // Unwrap type if the intermediate variable is defined from a slice type
        let typ = match typ {
            Type::Slice(styp) => (*(styp.base)).clone(),
            _ => typ,
        };
        let (var_name, nenv) = self.env.create_new_name("tmp");
        self.env = nenv;
        let mutability = self.current_mutability.clone().unwrap_or(VarMut::Mutable);
        VarDecl {
            name: var_name,
            typ: typ.clone(),
            value: Some(value),
            mutability,
            is_state_var: state_var,
            visibility: VarVis::Internal,
            data_loc: Some(DataLoc::Memory),
            loc,
            id: None,
            scope_id: None,
            overriding: Overriding::None,
        }
    }

    fn promote_complex_expr(&mut self, expr: Expr, nvdecls: &mut Vec<VarDecl>) -> Expr {
        if expr.is_atomic_expr() || expr.is_literal_based_expr() {
            expr
        } else {
            let typ = expr.typ();
            let loc = expr.loc();
            let vdecl = self.create_inter_var_decl(typ.clone(), false, loc, expr);
            nvdecls.push(vdecl.clone());
            Identifier::new(None, vdecl.name, typ, loc).into()
        }
    }

    fn flatten_expr(&mut self, expr: &Expr) -> (Vec<VarDecl>, Expr) {
        match expr {
            Expr::Lit(_) => (vec![], expr.clone()),
            Expr::Ident(_) => (vec![], expr.clone()),
            Expr::Unary(e) => self.flatten_unary_expr(e),
            Expr::Binary(e) => self.flatten_binary_expr(e),
            Expr::Assign(e) => self.flatten_assign_expr(e),
            Expr::Call(e) => self.flatten_call_expr(e),
            Expr::CallOpts(e) => self.flatten_call_opts_expr(e),
            Expr::Tuple(e) => self.flatten_tuple_expr(e),
            Expr::Index(e) => self.flatten_index_expr(e),
            Expr::Slice(e) => self.flatten_slice_expr(e),
            Expr::Member(e) => self.flatten_member_expr(e),
            Expr::Conditional(e) => self.flatten_conditional_expr(e),
            Expr::InlineArray(e) => self.flatten_inline_array_expr(e),
            Expr::New(_) => (vec![], expr.clone()),
            Expr::TypeName(_) => (vec![], expr.clone()),
        }
    }

    fn flatten_unary_expr(&mut self, expr: &UnaryExpr) -> (Vec<VarDecl>, Expr) {
        let (mut nvdecls, nopr) = self.flatten_expr(expr.body.borrow());
        let nopr = self.promote_complex_expr(nopr, &mut nvdecls);
        let nexpr = UnaryExpr { body: Box::new(nopr), ..expr.clone() };
        (nvdecls, nexpr.into())
    }

    fn flatten_binary_expr(&mut self, expr: &BinaryExpr) -> (Vec<VarDecl>, Expr) {
        let mut nvdecls = vec![];

        let (vdecls, nlhs) = self.flatten_expr(expr.left.borrow());
        nvdecls.extend(vdecls);
        let nlhs = self.promote_complex_expr(nlhs, &mut nvdecls);

        let (vdecls, nrhs) = self.flatten_expr(expr.right.borrow());
        nvdecls.extend(vdecls);
        let nrhs = self.promote_complex_expr(nrhs, &mut nvdecls);

        let nexpr = BinaryExpr { left: Box::new(nlhs), right: Box::new(nrhs), ..expr.clone() };
        (nvdecls, nexpr.into())
    }

    fn flatten_assign_expr(&mut self, expr: &AssignExpr) -> (Vec<VarDecl>, Expr) {
        let mut nvdecls = vec![];

        let (mut vdecls, nlhs) = self.flatten_expr(&expr.left);
        nvdecls.append(&mut vdecls);

        let (mut vdecls, nrhs) = self.flatten_expr(expr.right.borrow());
        nvdecls.append(&mut vdecls);

        // Transform self-assignment expressions into a normal assignment
        let typ = expr.typ.clone();
        let loc = expr.loc;
        let nrhs = match expr.operator {
            AssignOp::Assign => nrhs,
            AssignOp::AssignAdd => BinaryExpr::new(None, BinOp::Add, nlhs, nrhs, typ, loc).into(),
            AssignOp::AssignSub => BinaryExpr::new(None, BinOp::Sub, nlhs, nrhs, typ, loc).into(),
            AssignOp::AssignMul => BinaryExpr::new(None, BinOp::Mul, nlhs, nrhs, typ, loc).into(),
            AssignOp::AssignDiv => BinaryExpr::new(None, BinOp::Div, nlhs, nrhs, typ, loc).into(),
            AssignOp::AssignMod => BinaryExpr::new(None, BinOp::Mod, nlhs, nrhs, typ, loc).into(),
            AssignOp::AssignBitAnd => {
                BinaryExpr::new(None, BinOp::BitAnd, nlhs, nrhs, typ, loc).into()
            }
            AssignOp::AssignBitOr => {
                BinaryExpr::new(None, BinOp::BitOr, nlhs, nrhs, typ, loc).into()
            }
            AssignOp::AssignBitXor => {
                BinaryExpr::new(None, BinOp::BitXor, nlhs, nrhs, typ, loc).into()
            }
            AssignOp::AssignShl => BinaryExpr::new(None, BinOp::Shl, nlhs, nrhs, typ, loc).into(),
            AssignOp::AssignShr => BinaryExpr::new(None, BinOp::Shr, nlhs, nrhs, typ, loc).into(),
            AssignOp::AssignSar => BinaryExpr::new(None, BinOp::Sar, nlhs, nrhs, typ, loc).into(),
        };

        let nexpr =
            AssignExpr { operator: AssignOp::Assign, right: Box::new(nrhs), ..expr.clone() };
        (nvdecls, nexpr.into())
    }

    fn flatten_call_expr(&mut self, expr: &CallExpr) -> (Vec<VarDecl>, Expr) {
        // Do not flatten ABI encoding/decoding function calls
        if expr.is_abi_call() {
            return (vec![], expr.clone().into());
        }

        let mut nvdecls = vec![];
        let mut ncall_opts = vec![];

        let (vdecls, ncallee) = self.flatten_expr(&expr.callee);
        nvdecls.extend(vdecls);

        for call_opt in &expr.call_opts {
            let (vdecls, ncall_opt) = self.flatten_call_opt(call_opt);
            nvdecls.extend(vdecls);
            ncall_opts.push(ncall_opt);
        }

        let (vdecls, nargs) = self.flatten_call_args(&expr.args);
        nvdecls.extend(vdecls);

        let nexpr = CallExpr {
            callee: Box::new(ncallee),
            call_opts: ncall_opts,
            args: nargs,
            ..expr.clone()
        };
        (nvdecls, nexpr.into())
    }

    fn flatten_call_opts_expr(&mut self, expr: &CallOptsExpr) -> (Vec<VarDecl>, Expr) {
        let mut nvdecls = vec![];
        let mut ncall_opts = vec![];

        for call_opt in &expr.call_opts {
            let (vdecls, ncall_opt) = self.flatten_call_opt(call_opt);
            nvdecls.extend(vdecls);
            ncall_opts.push(ncall_opt);
        }

        let nexpr = CallOptsExpr { call_opts: ncall_opts, ..expr.clone() };
        (nvdecls, nexpr.into())
    }

    fn flatten_call_opt(&mut self, call_opt: &CallOpt) -> (Vec<VarDecl>, CallOpt) {
        let (mut nvdecls, nvalue) = self.flatten_expr(&call_opt.value);
        let nvalue = self.promote_complex_expr(nvalue, &mut nvdecls);
        let ncall_opt = CallOpt { value: nvalue, ..call_opt.clone() };
        (nvdecls, ncall_opt)
    }

    fn flatten_call_args(&mut self, call_args: &CallArgs) -> (Vec<VarDecl>, CallArgs) {
        let mut nvdecls = vec![];
        match call_args {
            CallArgs::Unnamed(args) => {
                let mut nargs = vec![];
                for arg in args {
                    let (var_decls, narg) = self.flatten_expr(arg);
                    nvdecls.extend(var_decls);
                    let narg = self.promote_complex_expr(narg, &mut nvdecls);
                    nargs.push(narg);
                }
                (nvdecls, CallArgs::Unnamed(nargs))
            }
            CallArgs::Named(args) => {
                let mut nargs = vec![];
                for arg in args {
                    let (var_decls, nvalue) = self.flatten_expr(&arg.value);
                    nvdecls.extend(var_decls);
                    let nvalue = self.promote_complex_expr(nvalue, &mut nvdecls);
                    let narg = NamedArg { value: nvalue, ..arg.clone() };
                    nargs.push(narg);
                }
                (nvdecls, CallArgs::Named(nargs))
            }
        }
    }

    fn flatten_tuple_expr(&mut self, expr: &TupleExpr) -> (Vec<VarDecl>, Expr) {
        let mut nvdecls = vec![];
        let mut nelems = vec![];
        for elem in &expr.elems {
            match elem {
                Some(e) => {
                    let (var_decls, nelem) = self.flatten_expr(e);
                    nvdecls.extend(var_decls);
                    let nelem = self.promote_complex_expr(nelem, &mut nvdecls);
                    nelems.push(Some(nelem));
                }
                None => nelems.push(None),
            }
        }
        let nexpr = TupleExpr { elems: nelems, ..expr.clone() };
        (nvdecls, nexpr.into())
    }

    fn flatten_index_expr(&mut self, expr: &IndexExpr) -> (Vec<VarDecl>, Expr) {
        let mut nvar_decls = vec![];

        let (var_decls, nbase) = self.flatten_expr(expr.base_expr.borrow());
        nvar_decls.extend(var_decls);
        let nbase = self.promote_complex_expr(nbase, &mut nvar_decls);

        let nindex = expr.index.as_ref().map(|idx| {
            let (var_decls, nidx) = self.flatten_expr(idx.borrow());
            nvar_decls.extend(var_decls);
            let nidx = self.promote_complex_expr(nidx, &mut nvar_decls);
            Box::new(nidx)
        });

        let nexpr = IndexExpr { base_expr: Box::new(nbase), index: nindex, ..expr.clone() };
        (nvar_decls, nexpr.into())
    }

    fn flatten_slice_expr(&mut self, expr: &SliceExpr) -> (Vec<VarDecl>, Expr) {
        let mut nvar_decls = vec![];

        let (vdecls, nbase) = self.flatten_expr(expr.base_expr.borrow());
        nvar_decls.extend(vdecls);

        let nstart = expr.start_index.as_ref().map(|start| {
            let (var_decls, nstart) = self.flatten_expr(start.borrow());
            nvar_decls.extend(var_decls);
            let nstart = self.promote_complex_expr(nstart, &mut nvar_decls);
            Box::new(nstart)
        });

        let nend = expr.end_index.as_ref().map(|end| {
            let (var_decls, nend) = self.flatten_expr(end.borrow());
            nvar_decls.extend(var_decls);
            let nend = self.promote_complex_expr(nend, &mut nvar_decls);
            Box::new(nend)
        });

        let nexpr = SliceExpr {
            base_expr: Box::new(nbase),
            start_index: nstart,
            end_index: nend,
            ..expr.clone()
        };
        (nvar_decls, nexpr.into())
    }

    fn flatten_member_expr(&mut self, expr: &MemberExpr) -> (Vec<VarDecl>, Expr) {
        // Do not flatten ABI-related expressions.
        if expr.member.to_string().eq(keywords::SELECTOR) {
            return (vec![], expr.clone().into());
        }

        // do not flatten base expressions of magic type.
        if matches!(expr.base.typ(), Type::Magic(_)) {
            return (vec![], expr.clone().into());
        }

        debug!("===== FLATTEN BASE OF MEMBER ACCESS: {}", expr.base);

        let (mut nvar_decls, nbase) = self.flatten_expr(expr.base.borrow());
        let nbase = self.promote_complex_expr(nbase, &mut nvar_decls);
        let nexpr = MemberExpr { base: Box::new(nbase), ..expr.clone() };

        (nvar_decls, nexpr.into())
    }

    fn flatten_conditional_expr(&mut self, expr: &ConditionalExpr) -> (Vec<VarDecl>, Expr) {
        let mut nvdecls = vec![];

        let (vdecls, ncond) = self.flatten_expr(expr.cond.borrow());
        nvdecls.extend(vdecls);
        let (vdecls, ntrue_br) = self.flatten_expr(expr.true_br.borrow());
        nvdecls.extend(vdecls);
        let (vdecls, nfalse_br) = self.flatten_expr(expr.false_br.borrow());
        nvdecls.extend(vdecls);

        let nexpr = ConditionalExpr {
            cond: Box::new(ncond),
            true_br: Box::new(ntrue_br),
            false_br: Box::new(nfalse_br),
            ..expr.clone()
        };
        (nvdecls, nexpr.into())
    }

    fn flatten_inline_array_expr(&mut self, expr: &InlineArrayExpr) -> (Vec<VarDecl>, Expr) {
        let mut nvdecls = vec![];
        let mut nelems = vec![];
        for elem in &expr.elems {
            let (var_decls, nelem) = self.flatten_expr(elem);
            nvdecls.extend(var_decls);
            let nelem = self.promote_complex_expr(nelem, &mut nvdecls);
            nelems.push(nelem);
        }
        let nexpr = InlineArrayExpr { elems: nelems, ..expr.clone() };
        (nvdecls, nexpr.into())
    }
}

impl Normalize<'_, Vec<VarDecl>> for ExprFlattener {
    /// Override `normalize_source_unit` to flatten expression inside variable
    /// declarations in source unit elements.
    fn normalize_source_unit(
        &mut self,
        _acc: Vec<VarDecl>,
        source_unit: &SourceUnit,
    ) -> (Vec<VarDecl>, SourceUnit) {
        let mut nelems = vec![];
        for elem in source_unit.elems.iter() {
            let (vdecls, nelem) = self.normalize_source_unit_elem(vec![], elem);
            for vdecl in vdecls.iter() {
                nelems.push(vdecl.clone().into());
            }
            nelems.push(nelem);
        }
        let nsource_unit = SourceUnit { elems: nelems, ..source_unit.clone() };
        (vec![], nsource_unit)
    }

    /// Override `normalize_contract_definition` to flatten expression inside
    /// variable declarations in contract elements.
    fn normalize_contract_def(
        &mut self,
        _acc: Vec<VarDecl>,
        contract: &ContractDef,
    ) -> (Vec<VarDecl>, ContractDef) {
        let mut nelems = vec![];
        for elem in contract.body.iter() {
            let (vdecls, nelem) = self.normalize_contract_elem(vec![], elem);
            for vdecl in vdecls.iter() {
                nelems.push(vdecl.clone().into());
            }
            nelems.push(nelem);
        }
        let ncontract = ContractDef { body: nelems, ..contract.clone() };
        (vec![], ncontract)
    }

    /// Override `normalize_block` to accumulate all new variable declarations
    /// as statements of the current block.
    fn normalize_block(&mut self, acc: Vec<VarDecl>, block: &Block) -> (Vec<VarDecl>, Block) {
        let mut nstmts = vec![];
        for stmt in block.body.iter() {
            let (nvdecls, nstmt) = self.normalize_stmt(vec![], stmt);
            for mut vdecl in nvdecls.into_iter() {
                // Lift initial value of [`VarDecl`] to the RHS of the [`VarDeclStmt`].
                let value = vdecl.value.clone();
                let loc = vdecl.loc;
                vdecl.value = None;
                let vdecl_stmt = VarDeclStmt::new(None, vec![Some(vdecl.clone())], value, loc);
                nstmts.push(vdecl_stmt.into());
            }
            nstmts.push(nstmt);
        }
        let nblock = Block { body: nstmts, ..block.clone() };
        (acc, nblock)
    }

    /// Override `normalize_var_decl` to capture mutability.
    fn normalize_var_decl(
        &mut self,
        acc: Vec<VarDecl>,
        vdecl: &VarDecl,
    ) -> (Vec<VarDecl>, VarDecl) {
        let saved_mutability = self.current_mutability.clone();
        self.current_mutability = Some(vdecl.mutability.clone());
        let res = crate::ast::utils::normalize::default::normalize_var_decl(self, acc, vdecl);
        self.current_mutability = saved_mutability;
        res
    }

    /// Override `normalize_expr` to call the `flatten_expr` function.
    fn normalize_expr(&mut self, acc: Vec<VarDecl>, expr: &Expr) -> (Vec<VarDecl>, Expr) {
        let mut nvdecls = acc;
        let (vdecls, nexpr) = self.flatten_expr(expr);
        nvdecls.extend(vdecls);
        (nvdecls, nexpr)
    }
}

/// Flatten expressions in source units.
///
/// This function transform all expressions to three-value code.
pub fn flatten_expr(source_units: &[SourceUnit], env: Option<&NamingEnv>) -> Vec<SourceUnit> {
    println!("Normalize AST: flattening expressions");
    let mut nsource_units = vec![];
    for sunit in source_units.iter() {
        let mut flattener = ExprFlattener::new(env);
        let (_, nsource_unit) = flattener.normalize_source_unit(vec![], sunit);
        nsource_units.push(nsource_unit);
    }
    nsource_units
}

//-------------------------------------------------
// Unit tests
//-------------------------------------------------

/// Unit tests
#[cfg(test)]
mod tests {
    use crate::{
        ast::utils::syntactic_comparer::compare_source_units,
        compile::compile_solidity_source_code,
        ast::normalize::{flatten_expr, utils::configure_unit_test_env},
    };
    use indoc::indoc;

    // Test normalization in a single contract.
    #[test]
    fn test_flatten_expr() {
        let _ = configure_unit_test_env();

        // Input contract
        let input_contract = indoc! {r###"
        contract Flatten {
            function test1() public pure {
                uint256 k;
                k = k + 1 + 2 - 3;
            }
        }"###};

        // Expected output contract
        let expected_contract = indoc! {r###"
        contract Flatten {
            function test1() public pure {
                uint256 k;
                uint256 tmp_0 = k + 1;
                uint256 tmp_1 = tmp_0 + 2;
                k = tmp_1 - 3;
            }
        }"###};

        let input_sunits = match compile_solidity_source_code(input_contract, "0.8.1") {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse input source unit: {err}"),
        };
        let output_sunits = flatten_expr(&input_sunits, None);

        let expected_sunits = match compile_solidity_source_code(expected_contract, "0.8.1") {
            Ok(sunits) => sunits,
            Err(err) => panic!("Failed to parse expected source unit: {err}"),
        };

        if let Err(err) = compare_source_units(&output_sunits, &expected_sunits) {
            panic!("Failed to flatten expr: {err}!")
        }
    }
}
