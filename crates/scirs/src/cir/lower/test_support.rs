//! Test helpers: build small SIR contracts and lower them through the full
//! SIR → CIR → BIR pipeline, so BIR tests exercise the real classification
//! of state accesses and calls.

use crate::bir::cfg::{BasicBlock, FunctionId};
use crate::bir::module::Module;
use crate::sir::attrs::sir_attrs;
use crate::sir::{
    AssignStmt, Attr, AttrValue, BinOp, BinOpExpr, BoolLit, ContractDecl, Decl, Expr, ExprStmt,
    FunctionDecl, Lit, MemberDecl, OverflowSemantics, Param, Stmt, StorageDecl, Type, VarExpr,
};

/// Name of the test contract.
pub(crate) const CONTRACT: &str = "C";

/// A function of the test contract.
pub(crate) struct TestFunction {
    pub name: &'static str,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub public: bool,
}

pub(crate) fn var(name: &str) -> Expr {
    Expr::Var(VarExpr::new(name.to_string(), Type::I256, None))
}

pub(crate) fn lit(value: bool) -> Expr {
    Expr::Lit(Lit::Bool(BoolLit::new(value, None)))
}

pub(crate) fn binop(op: BinOp, lhs: Expr, rhs: Expr) -> Expr {
    Expr::BinOp(BinOpExpr {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        overflow: OverflowSemantics::Checked,
        span: None,
    })
}

pub(crate) fn assign(lhs: Expr, rhs: Expr) -> Stmt {
    Stmt::Assign(AssignStmt { lhs, rhs, span: None })
}

pub(crate) fn expr_stmt(expr: Expr) -> Stmt {
    Stmt::Expr(ExprStmt { expr, span: None })
}

pub(crate) fn params(names: &[&str]) -> Vec<Param> {
    names
        .iter()
        .map(|n| Param::new(n.to_string(), Type::I256))
        .collect()
}

/// Lower contract `C` with state variables `storage` and `functions`.
pub(crate) fn lower_contract(storage: &[&str], functions: Vec<TestFunction>) -> Module {
    let mut members: Vec<MemberDecl> = storage
        .iter()
        .map(|name| {
            MemberDecl::Storage(StorageDecl::new(name.to_string(), Type::I256, None, None))
        })
        .collect();
    for func in functions {
        let mut decl =
            FunctionDecl::new(func.name.to_string(), func.params, vec![], Some(func.body), None);
        if func.public {
            let public = AttrValue::String("public".to_string());
            decl.attrs
                .push(Attr::new("sir", sir_attrs::VISIBILITY, public));
        }
        members.push(MemberDecl::Function(decl));
    }
    let contract = ContractDecl::new(CONTRACT.to_string(), members, None);
    let sir = crate::sir::Module::new("test", vec![Decl::Contract(contract)]);
    let cir = crate::sir::lower::lower_module(&sir).expect("SIR → CIR lowering failed");
    super::lower_module(&cir).expect("CIR → BIR lowering failed")
}

/// Lower a single public function `C.f` and return its blocks.
pub(crate) fn lower_function(
    storage: &[&str],
    params: Vec<Param>,
    body: Vec<Stmt>,
) -> Vec<BasicBlock> {
    let func = TestFunction { name: "f", params, body, public: true };
    let module = lower_contract(storage, vec![func]);
    let id = FunctionId(format!("{CONTRACT}.f"));
    module
        .functions
        .into_iter()
        .find(|f| f.id == id)
        .expect("function C.f")
        .blocks
}
