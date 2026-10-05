//! SIR building blocks for the BIR detector tests: a contract
//! `C { balances; locked; ... }` and the statements its functions use.

use crate::context::{AnalysisConfig, AnalysisContext};
use crate::detectors::BugDetectionPass;
use crate::passes::base::AnalysisPass;
use crate::passes::bir::{
    DominanceArtifact, DominancePass, FunctionEffectsArtifact, FunctionEffectsPass,
};
use bugs::bug::Bug;
use common::loc::Loc;
use num_traits::Zero;
use scirs::sir::attrs::sir_attrs;
use scirs::sir::evm::{EvmExpr, EvmLowLevelCall, EvmMsgSender, EvmTransfer};
use scirs::sir::{
    AssignStmt, Attr, AttrValue, BoolLit, CallArgs, CallExpr, ContractDecl, Decl, DialectExpr,
    Expr, ExprStmt, FunctionDecl, IfStmt, IndexAccessExpr, IntNum, Lit, MemberDecl, Module, Num,
    NumLit, Param, RevertStmt, Stmt, StorageDecl, StringLit, Type, VarExpr,
};

pub(crate) fn var(name: &str) -> Expr {
    Expr::Var(VarExpr::new(name.to_string(), Type::I256, None))
}

fn lit(value: bool) -> Expr {
    Expr::Lit(Lit::Bool(BoolLit::new(value, None)))
}

fn sender() -> Expr {
    Expr::Dialect(DialectExpr::Evm(EvmExpr::MsgSender(EvmMsgSender { loc: Loc::default() })))
}

fn stmt(expr: Expr) -> Stmt {
    Stmt::Expr(ExprStmt { expr, span: None })
}

/// `msg.sender.call{value: amount}("")`
pub(crate) fn call_out() -> Stmt {
    stmt(Expr::Dialect(DialectExpr::Evm(EvmExpr::LowLevelCall(EvmLowLevelCall {
        target: Box::new(sender()),
        data: Box::new(Expr::Lit(Lit::String(StringLit::new(String::new(), None)))),
        value: Some(Box::new(var("amount"))),
        gas: None,
        loc: Loc::default(),
    }))))
}

/// `msg.sender.transfer(amount)`
pub(crate) fn transfer_out() -> Stmt {
    stmt(Expr::Dialect(DialectExpr::Evm(EvmExpr::Transfer(EvmTransfer {
        target: Box::new(sender()),
        amount: Box::new(var("amount")),
        loc: Loc::default(),
    }))))
}

/// `balances[msg.sender] = false`
pub(crate) fn write_balance() -> Stmt {
    write_balance_at(sender())
}

/// `if (balances[msg.sender]) {}`: reads the balance.
pub(crate) fn read_balance() -> Stmt {
    read_balance_at(sender())
}

/// `balances[key]`
fn balance_at(key: Expr) -> Expr {
    Expr::IndexAccess(IndexAccessExpr {
        base: Box::new(var("balances")),
        index: Some(Box::new(key)),
        ty: Type::I256,
        span: None,
    })
}

/// `balances[key] = false`
pub(crate) fn write_balance_at(key: Expr) -> Stmt {
    Stmt::Assign(AssignStmt { lhs: balance_at(key), rhs: lit(false), span: None })
}

/// `if (balances[key]) {}`
pub(crate) fn read_balance_at(key: Expr) -> Stmt {
    Stmt::If(IfStmt { cond: balance_at(key), then_body: vec![], else_body: None, span: None })
}

/// The integer literal `0` or `1`.
pub(crate) fn int(one: bool) -> Expr {
    let value = if one {
        IntNum::one()
    } else {
        IntNum::new(Zero::zero(), Type::I256)
    };
    Expr::Lit(Lit::Num(NumLit::new(Num::Int(value), None)))
}

/// `if (locked) revert();`
pub(crate) fn check_locked() -> Stmt {
    let revert = Stmt::Revert(RevertStmt { error: None, args: vec![], span: None });
    Stmt::If(IfStmt { cond: var("locked"), then_body: vec![revert], else_body: None, span: None })
}

pub(crate) fn set_locked(value: bool) -> Stmt {
    Stmt::Assign(AssignStmt { lhs: var("locked"), rhs: lit(value), span: None })
}

pub(crate) fn call_internal(name: &str) -> Stmt {
    stmt(Expr::FunctionCall(CallExpr {
        callee: Box::new(var(name)),
        args: CallArgs::Positional(vec![]),
        ty: Type::None,
        span: None,
    }))
}

pub(crate) fn function(name: &str, public: bool, body: Vec<Stmt>) -> FunctionDecl {
    let params = vec![Param::new("amount".to_string(), Type::I256)];
    let mut func = FunctionDecl::new(name.to_string(), params, vec![], Some(body), None);
    if public {
        let visibility = AttrValue::String("public".to_string());
        func.attrs
            .push(Attr::new("sir", sir_attrs::VISIBILITY, visibility));
    }
    func
}

/// `func` marked with a reentrancy-guard attribute (e.g. `nonReentrant`).
pub(crate) fn guarded(mut func: FunctionDecl) -> FunctionDecl {
    func.attrs
        .push(Attr::new("sir", sir_attrs::REENTRANCY_GUARD, AttrValue::Bool(true)));
    func
}

/// Run `detector` on contract `C { balances; locked; functions }`.
pub(crate) fn detect_with(
    detector: &dyn BugDetectionPass,
    functions: Vec<FunctionDecl>,
) -> Vec<Bug> {
    let mut members: Vec<MemberDecl> = ["balances", "locked"]
        .iter()
        .map(|n| MemberDecl::Storage(StorageDecl::new(n.to_string(), Type::I256, None, None)))
        .collect();
    members.extend(functions.into_iter().map(MemberDecl::Function));
    let contract = ContractDecl::new("C".to_string(), members, None);
    let module = Module::new("test", vec![Decl::Contract(contract)]);
    let mut context = AnalysisContext::new(vec![module], AnalysisConfig::default());
    context.store::<DominanceArtifact>(DominancePass.run(&context).unwrap());
    context.store::<FunctionEffectsArtifact>(FunctionEffectsPass.run(&context).unwrap());
    detector.detect(&context).unwrap()
}
