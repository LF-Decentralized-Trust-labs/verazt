use crate::ast::*;
use color_eyre::eyre::{Result, bail};
use std::{
    fmt::{self, Display},
    ops::Deref,
};

//-------------------------------------------------------------------------
// Data structures representing all expressions
//-------------------------------------------------------------------------

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Expr {
    Ident(Identifier),
    Lit(Lit),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Assign(AssignExpr),
    Call(CallExpr),
    CallOpts(CallOptsExpr),
    Tuple(TupleExpr),
    Index(IndexExpr),
    Slice(SliceExpr),
    Member(MemberExpr),
    Conditional(ConditionalExpr),
    InlineArray(InlineArrayExpr),
    New(NewExpr),
    TypeName(TypeNameExpr),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct AssignExpr {
    pub id: Option<isize>,
    pub operator: AssignOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum AssignOp {
    Assign,
    AssignAdd,
    AssignSub,
    AssignMul,
    AssignDiv,
    AssignMod,
    AssignBitAnd,
    AssignBitOr,
    AssignBitXor,
    AssignShl,
    AssignShr,
    AssignSar,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BinaryExpr {
    pub id: Option<isize>,
    pub operator: BinOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum BinOp {
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    And,
    Or,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Power,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Sar, // Bitwise shift arithmetic right (`>>>`)
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CallExpr {
    pub id: Option<isize>,
    pub callee: Box<Expr>,
    pub call_opts: Vec<CallOpt>,
    pub args: CallArgs,
    pub kind: CallKind,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CallKind {
    FuncCall,
    TypeConversionCall,
    StructConstructorCall,
    ErrorCall,
    EventCall,
    ModifierInvoc,
    BaseConstructorCall,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CallArgs {
    Unnamed(Vec<Expr>),
    Named(Vec<NamedArg>),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct NamedArg {
    pub name: String,
    pub value: Expr,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CallOptsExpr {
    pub id: Option<isize>,
    pub callee: Box<Expr>,
    pub call_opts: Vec<CallOpt>,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CallOpt {
    pub name: String,
    pub value: Expr,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ConditionalExpr {
    pub id: Option<isize>,
    pub cond: Box<Expr>,
    pub true_br: Box<Expr>,  // True branch
    pub false_br: Box<Expr>, // False branch
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct IndexExpr {
    pub id: Option<isize>,
    pub base_expr: Box<Expr>,
    pub index: Option<Box<Expr>>,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct InlineArrayExpr {
    pub id: Option<isize>,
    pub elems: Vec<Expr>,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MemberExpr {
    pub id: Option<isize>,
    pub base: Box<Expr>,
    pub member: Name,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct NewExpr {
    pub id: Option<isize>,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SliceExpr {
    pub id: Option<isize>,
    pub base_expr: Box<Expr>,
    pub start_index: Option<Box<Expr>>,
    pub end_index: Option<Box<Expr>>,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TupleExpr {
    pub id: Option<isize>,
    pub elems: Vec<Option<Expr>>,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TypeNameExpr {
    pub id: Option<isize>,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UnaryExpr {
    pub id: Option<isize>,
    pub op: UnaryOp,
    pub body: Box<Expr>,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum UnaryOp {
    PreIncr,  // Pre-increment
    PostIncr, // Post-increment
    PreDecr,  // Pre-decrement
    PostDecr, // Post-decrement
    Not,
    Neg,
    BitNot,
    Delete,
}

//-------------------------------------------------------------------------
// Implementation for expressions
//-------------------------------------------------------------------------

impl Expr {
    pub fn id(&self) -> Option<isize> {
        match self {
            Expr::Lit(_) => None,
            Expr::Ident(id) => id.id,
            Expr::Unary(e) => e.id,
            Expr::Binary(e) => e.id,
            Expr::Assign(e) => e.id,
            Expr::Call(e) => e.id,
            Expr::CallOpts(e) => e.id,
            Expr::Tuple(e) => e.id,
            Expr::Index(e) => e.id,
            Expr::Slice(e) => e.id,
            Expr::Member(e) => e.id,
            Expr::Conditional(e) => e.id,
            Expr::InlineArray(e) => e.id,
            Expr::New(e) => e.id,
            Expr::TypeName(e) => e.id,
        }
    }

    pub fn loc(&self) -> Option<Loc> {
        match self {
            Expr::Ident(id) => id.loc,
            Expr::Lit(lit) => lit.loc(),
            Expr::Unary(e) => e.loc,
            Expr::Binary(e) => e.loc,
            Expr::Assign(e) => e.loc,
            Expr::Call(e) => e.loc,
            Expr::CallOpts(e) => e.loc,
            Expr::Tuple(e) => e.loc,
            Expr::Index(e) => e.loc,
            Expr::Slice(e) => e.loc,
            Expr::Member(e) => e.loc,
            Expr::Conditional(e) => e.loc,
            Expr::InlineArray(e) => e.loc,
            Expr::New(e) => e.loc,
            Expr::TypeName(e) => e.loc,
        }
    }

    pub fn typ(&self) -> Type {
        match self {
            Expr::Ident(id) => id.typ.clone(),
            Expr::Lit(lit) => lit.typ(),
            Expr::Unary(e) => e.typ.clone(),
            Expr::Binary(e) => e.typ.clone(),
            Expr::Assign(e) => e.typ.clone(),
            Expr::Call(e) => e.typ.clone(),
            Expr::CallOpts(e) => e.typ.clone(),
            Expr::Tuple(e) => e.typ.clone(),
            Expr::Index(e) => e.typ.clone(),
            Expr::Slice(e) => e.typ.clone(),
            Expr::Member(e) => e.typ.clone(),
            Expr::Conditional(e) => e.typ.clone(),
            Expr::InlineArray(e) => e.typ.clone(),
            Expr::New(e) => e.typ.clone(),
            Expr::TypeName(e) => e.typ.clone(),
        }
    }

    pub fn is_atomic_expr(&self) -> bool {
        match self {
            Expr::Ident(_) | Expr::Lit(_) | Expr::New(_) | Expr::TypeName(_) => true,
            Expr::Call(call_expr) => call_expr.callee.to_string().eq("type"),
            _ => false,
        }
    }

    pub fn is_literal_based_expr(&self) -> bool {
        match self {
            Expr::Lit(_) => true,
            Expr::Unary(exp) => exp.body.is_literal_based_expr(),
            Expr::Binary(exp) => {
                exp.left.is_literal_based_expr() && exp.right.is_literal_based_expr()
            }
            Expr::Tuple(exp) => exp.elems.iter().all(|elem| match elem {
                Some(e) => e.is_literal_based_expr(),
                None => true,
            }),
            Expr::Conditional(exp) => {
                exp.cond.is_literal_based_expr()
                    && exp.true_br.is_literal_based_expr()
                    && exp.false_br.is_literal_based_expr()
            }
            _ => false,
        }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        match self {
            Expr::Ident(exp) => exp.update_data_type(new_type),
            Expr::Lit(exp) => exp.update_data_type(new_type),
            Expr::Unary(exp) => exp.update_data_type(new_type),
            Expr::Binary(exp) => exp.update_data_type(new_type),
            Expr::Assign(exp) => exp.update_data_type(new_type),
            Expr::Call(exp) => exp.update_data_type(new_type),
            Expr::CallOpts(exp) => exp.update_data_type(new_type),
            Expr::Tuple(exp) => exp.update_data_type(new_type),
            Expr::Index(exp) => exp.update_data_type(new_type),
            Expr::Slice(exp) => exp.update_data_type(new_type),
            Expr::Member(exp) => exp.update_data_type(new_type),
            Expr::Conditional(exp) => exp.update_data_type(new_type),
            Expr::InlineArray(exp) => exp.update_data_type(new_type),
            Expr::New(exp) => exp.update_data_type(new_type),
            Expr::TypeName(exp) => exp.update_data_type(new_type),
        }
    }
}

impl From<Lit> for Expr {
    fn from(lit: Lit) -> Self {
        Expr::Lit(lit)
    }
}

impl From<Identifier> for Expr {
    fn from(id: Identifier) -> Self {
        Expr::Ident(id)
    }
}

impl From<AssignExpr> for Expr {
    fn from(expr: AssignExpr) -> Expr {
        Expr::Assign(expr)
    }
}

impl From<UnaryExpr> for Expr {
    fn from(expr: UnaryExpr) -> Expr {
        Expr::Unary(expr)
    }
}

impl From<BinaryExpr> for Expr {
    fn from(expr: BinaryExpr) -> Expr {
        Expr::Binary(expr)
    }
}

impl From<ConditionalExpr> for Expr {
    fn from(expr: ConditionalExpr) -> Expr {
        Expr::Conditional(expr)
    }
}

impl From<CallExpr> for Expr {
    fn from(expr: CallExpr) -> Expr {
        Expr::Call(expr)
    }
}

impl From<CallOptsExpr> for Expr {
    fn from(expr: CallOptsExpr) -> Expr {
        Expr::CallOpts(expr)
    }
}

impl From<IndexExpr> for Expr {
    fn from(expr: IndexExpr) -> Expr {
        Expr::Index(expr)
    }
}

impl From<SliceExpr> for Expr {
    fn from(expr: SliceExpr) -> Expr {
        Expr::Slice(expr)
    }
}

impl From<MemberExpr> for Expr {
    fn from(expr: MemberExpr) -> Expr {
        Expr::Member(expr)
    }
}

impl From<InlineArrayExpr> for Expr {
    fn from(expr: InlineArrayExpr) -> Expr {
        Expr::InlineArray(expr)
    }
}

impl From<NewExpr> for Expr {
    fn from(expr: NewExpr) -> Expr {
        Expr::New(expr)
    }
}

impl From<TupleExpr> for Expr {
    fn from(expr: TupleExpr) -> Expr {
        Expr::Tuple(expr)
    }
}

impl From<TypeNameExpr> for Expr {
    fn from(expr: TypeNameExpr) -> Expr {
        Expr::TypeName(expr)
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Ident(id) => write!(f, "{id}"),
            Expr::Lit(c) => write!(f, "{c}"),
            Expr::New(e) => write!(f, "{e}"),
            Expr::Unary(e) => write!(f, "{e}"),
            Expr::Binary(e) => write!(f, "{e}"),
            Expr::Assign(e) => write!(f, "{e}"),
            Expr::Tuple(e) => write!(f, "{e}"),
            Expr::Index(e) => write!(f, "{e}"),
            Expr::Slice(e) => write!(f, "{e}"),
            Expr::Member(e) => write!(f, "{e}"),
            Expr::Conditional(e) => write!(f, "{e}"),
            Expr::Call(e) => write!(f, "{e}"),
            Expr::CallOpts(e) => write!(f, "{e}"),
            Expr::TypeName(typ) => write!(f, "{typ}"),
            Expr::InlineArray(e) => write!(f, "{e}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Assignment expression
//-------------------------------------------------------------------------

impl AssignExpr {
    pub fn new(
        id: Option<isize>,
        op: AssignOp,
        lhs: Expr,
        rhs: Expr,
        typ: Type,
        loc: Option<Loc>,
    ) -> Self {
        AssignExpr { id, operator: op, left: Box::new(lhs), right: Box::new(rhs), typ, loc }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for AssignExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.left, self.operator, self.right)
    }
}

//-------------------------------------------------------------------------
// Implementation for Assignment operator
//-------------------------------------------------------------------------

impl AssignOp {
    pub fn new(operator: &str) -> Result<Self> {
        match operator {
            "=" => Ok(AssignOp::Assign),
            "+=" => Ok(AssignOp::AssignAdd),
            "&=" => Ok(AssignOp::AssignBitAnd),
            "|=" => Ok(AssignOp::AssignBitOr),
            "^=" => Ok(AssignOp::AssignBitXor),
            "/=" => Ok(AssignOp::AssignDiv),
            "%=" => Ok(AssignOp::AssignMod),
            "*=" => Ok(AssignOp::AssignMul),
            ">>>=" => Ok(AssignOp::AssignSar),
            "<<=" => Ok(AssignOp::AssignShl),
            ">>=" => Ok(AssignOp::AssignShr),
            "-=" => Ok(AssignOp::AssignSub),
            _ => bail!("Failed to parse assignment operator: {}", operator),
        }
    }
}

impl Display for AssignOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssignOp::Assign => write!(f, "="),
            AssignOp::AssignAdd => write!(f, "+="),
            AssignOp::AssignSub => write!(f, "-="),
            AssignOp::AssignMul => write!(f, "*="),
            AssignOp::AssignDiv => write!(f, "/="),
            AssignOp::AssignMod => write!(f, "%="),
            AssignOp::AssignBitAnd => write!(f, "&="),
            AssignOp::AssignBitOr => write!(f, "|="),
            AssignOp::AssignBitXor => write!(f, "^="),
            AssignOp::AssignShl => write!(f, "<<="),
            AssignOp::AssignShr => write!(f, ">>="),
            AssignOp::AssignSar => write!(f, ">>>="),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Unary operator.
//-------------------------------------------------------------------------

impl UnaryOp {
    pub fn new(operator: &str, is_prefix: bool) -> Result<Self> {
        match (operator, is_prefix) {
            ("++", true) => Ok(UnaryOp::PreIncr),
            ("++", false) => Ok(UnaryOp::PostIncr),
            ("--", true) => Ok(UnaryOp::PreDecr),
            ("--", false) => Ok(UnaryOp::PostDecr),
            ("-", _) => Ok(UnaryOp::Neg),
            ("!", _) => Ok(UnaryOp::Not),
            ("~", _) => Ok(UnaryOp::BitNot),
            ("delete", _) => Ok(UnaryOp::Delete),
            _ => bail!("Invalid unary operator: {}", operator),
        }
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::PreIncr => write!(f, "++"),
            UnaryOp::PostIncr => write!(f, "++"),
            UnaryOp::PreDecr => write!(f, "--"),
            UnaryOp::PostDecr => write!(f, "--"),
            UnaryOp::Not => write!(f, "!"),
            UnaryOp::Neg => write!(f, "-"),
            UnaryOp::BitNot => write!(f, "~"),
            UnaryOp::Delete => write!(f, "delete"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Unary expression
//-------------------------------------------------------------------------

impl UnaryExpr {
    pub fn new(
        id: Option<isize>,
        op: UnaryOp,
        operand: Expr,
        typ: Type,
        loc: Option<Loc>,
    ) -> Self {
        UnaryExpr { id, op, body: Box::new(operand), typ, loc }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for UnaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (op, exp) = (&self.op, &*self.body);
        match self.op {
            UnaryOp::Delete => write!(f, "{op} {exp}"),
            UnaryOp::PostDecr => write!(f, "{exp}{op}"),
            UnaryOp::PostIncr => write!(f, "{exp}{op}"),
            _ => write!(f, "{op}{exp}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Binary expression
//-------------------------------------------------------------------------

impl BinaryExpr {
    pub fn new(
        id: Option<isize>,
        op: BinOp,
        lhs: Expr,
        rhs: Expr,
        typ: Type,
        loc: Option<Loc>,
    ) -> Self {
        BinaryExpr { id, operator: op, left: Box::new(lhs), right: Box::new(rhs), typ, loc }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for BinaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.left, self.operator, self.right)
    }
}

//-------------------------------------------------------------------------
// Implementation for Binary operator
//-------------------------------------------------------------------------

impl BinOp {
    pub fn new(operator: &str) -> Result<Self> {
        match operator {
            "+" => Ok(BinOp::Add),
            "-" => Ok(BinOp::Sub),
            "*" => Ok(BinOp::Mul),
            "/" => Ok(BinOp::Div),
            "%" => Ok(BinOp::Mod),
            "<" => Ok(BinOp::Lt),
            ">" => Ok(BinOp::Gt),
            "<=" => Ok(BinOp::Le),
            ">=" => Ok(BinOp::Ge),
            "==" => Ok(BinOp::Eq),
            "!=" => Ok(BinOp::Ne),
            "&&" => Ok(BinOp::And),
            "||" => Ok(BinOp::Or),
            "**" => Ok(BinOp::Power),
            "&" => Ok(BinOp::BitAnd),
            "|" => Ok(BinOp::BitOr),
            "^" => Ok(BinOp::BitXor),
            "<<" => Ok(BinOp::Shl),
            ">>" => Ok(BinOp::Shr),
            ">>>" => Ok(BinOp::Sar),
            _ => bail!("Invalid binary operator: {}", operator),
        }
    }
}

impl Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Mod => write!(f, "%"),
            BinOp::Lt => write!(f, "<"),
            BinOp::Gt => write!(f, ">"),
            BinOp::Le => write!(f, "<="),
            BinOp::Ge => write!(f, ">="),
            BinOp::Eq => write!(f, "=="),
            BinOp::Ne => write!(f, "!="),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
            BinOp::Power => write!(f, "**"),
            BinOp::BitAnd => write!(f, "&"),
            BinOp::BitOr => write!(f, "|"),
            BinOp::BitXor => write!(f, "^"),
            BinOp::Shl => write!(f, "<<"),
            BinOp::Shr => write!(f, ">>"),
            BinOp::Sar => write!(f, ">>>"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Call expression
//-------------------------------------------------------------------------

impl CallExpr {
    pub fn new_call_unnamed_args(
        id: Option<isize>,
        callee: Expr,
        call_opts: Vec<CallOpt>,
        args: Vec<Expr>,
        kind: CallKind,
        typ: Type,
        loc: Option<Loc>,
    ) -> Self {
        let args = CallArgs::new_unnamed_args(args);
        CallExpr { id, callee: Box::new(callee), call_opts, args, kind, typ, loc }
    }

    pub fn new_call_named_args(
        id: Option<isize>,
        callee: Expr,
        call_opts: Vec<CallOpt>,
        args: Vec<NamedArg>,
        kind: CallKind,
        typ: Type,
        loc: Option<Loc>,
    ) -> Self {
        let args = CallArgs::new_named_args(args);
        CallExpr { id, callee: Box::new(callee), call_opts, args, kind, typ, loc }
    }

    pub fn is_abi_call(&self) -> bool {
        match self.callee.deref() {
            Expr::Member(expr) => expr.base.to_string().eq("abi"),
            _ => false,
        }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for CallExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.call_opts.is_empty() {
            write!(f, "{}({})", self.callee, self.args)
        } else {
            let call_opts = self
                .call_opts
                .iter()
                .map(|opt| opt.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            write!(f, "{}{{{}}}({})", self.callee, call_opts, self.args)
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Call Arguments
//-------------------------------------------------------------------------

impl CallArgs {
    pub fn new_unnamed_args(args: Vec<Expr>) -> CallArgs {
        CallArgs::Unnamed(args)
    }

    pub fn new_named_args(args: Vec<NamedArg>) -> CallArgs {
        CallArgs::Named(args)
    }
}

impl Display for CallArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CallArgs::Unnamed(exps) => {
                let args = exps
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{args}")
            }
            CallArgs::Named(args) => {
                let args = args
                    .iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{{{args}}}")
            }
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Named Arguments
//-------------------------------------------------------------------------

impl NamedArg {
    pub fn new(name: String, value: Expr, loc: Option<Loc>) -> Self {
        Self { name, value, loc }
    }
}

impl Display for NamedArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

//-------------------------------------------------------------------------
// Implementation for Call option expression
//-------------------------------------------------------------------------

impl CallOptsExpr {
    pub fn new(
        id: Option<isize>,
        callee: Expr,
        call_opts: Vec<CallOpt>,
        typ: Type,
        loc: Option<Loc>,
    ) -> Self {
        CallOptsExpr { id, callee: Box::new(callee), call_opts, typ, loc }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for CallOptsExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let call_opts = self
            .call_opts
            .iter()
            .map(|opt| opt.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{}{{{}}}", self.callee, call_opts)
    }
}

//-------------------------------------------------------------------------
// Implementation for Call option
//-------------------------------------------------------------------------

impl CallOpt {
    pub fn new(name: String, value: Expr, loc: Option<Loc>) -> Self {
        CallOpt { name, value, loc }
    }
}

impl Display for CallOpt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

//-------------------------------------------------------------------------
// Implementation for Conditional expression
//-------------------------------------------------------------------------

impl ConditionalExpr {
    pub fn new(
        id: Option<isize>,
        condition: Expr,
        true_br: Expr,
        false_br: Expr,
        typ: Type,
        loc: Option<Loc>,
    ) -> Self {
        ConditionalExpr {
            id,
            cond: Box::new(condition),
            true_br: Box::new(true_br),
            false_br: Box::new(false_br),
            typ,
            loc,
        }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for ConditionalExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ? {} : {}", self.cond, self.true_br, self.false_br)
    }
}

//-------------------------------------------------------------------------
// Implementation for Index access expression
//-------------------------------------------------------------------------

impl IndexExpr {
    pub fn new(
        id: Option<isize>,
        base_expr: Expr,
        index: Option<Expr>,
        typ: Type,
        loc: Option<Loc>,
    ) -> Self {
        IndexExpr { id, base_expr: Box::new(base_expr), index: index.map(Box::new), typ, loc }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for IndexExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.index {
            Some(index) => write!(f, "{}[{}]", self.base_expr, index),
            None => write!(f, "{}[]", self.base_expr),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Inline array expression
//-------------------------------------------------------------------------

impl InlineArrayExpr {
    pub fn new(id: Option<isize>, elems: Vec<Expr>, typ: Type, loc: Option<Loc>) -> Self {
        InlineArrayExpr { id, elems, typ, loc }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for InlineArrayExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let elems = self
            .elems
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "[{elems}]")
    }
}

//-------------------------------------------------------------------------
// Implementation for Member access expression
//-------------------------------------------------------------------------

impl MemberExpr {
    pub fn new(id: Option<isize>, base: Expr, member: Name, typ: Type, loc: Option<Loc>) -> Self {
        MemberExpr { id, base: Box::new(base), member, typ, loc }
    }

    pub fn set_naming_index(&mut self, index: Option<usize>) {
        self.member.index = index
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for MemberExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.base, self.member)
    }
}

//-------------------------------------------------------------------------
// Implementation for New expression
//-------------------------------------------------------------------------

impl NewExpr {
    pub fn new(id: Option<isize>, typ: Type, loc: Option<Loc>) -> Self {
        NewExpr { id, typ, loc }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for NewExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "new {}", self.typ)
    }
}

//-------------------------------------------------------------------------
// Implementation for slice expression
//-------------------------------------------------------------------------

impl SliceExpr {
    pub fn new(
        id: Option<isize>,
        base_expr: Expr,
        start_index: Option<Expr>,
        end_index: Option<Expr>,
        typ: Type,
        loc: Option<Loc>,
    ) -> Self {
        SliceExpr {
            id,
            base_expr: Box::new(base_expr),
            start_index: start_index.map(Box::new),
            end_index: end_index.map(Box::new),
            typ,
            loc,
        }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for SliceExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[", self.base_expr).ok();
        if let Some(e) = &self.start_index {
            write!(f, "{e}").ok();
        }
        write!(f, ":").ok();
        if let Some(e) = &self.end_index {
            write!(f, "{e}").ok();
        }
        write!(f, "]")
    }
}

//-------------------------------------------------------------------------
// Implementation for Tuple expression
//-------------------------------------------------------------------------

impl TupleExpr {
    pub fn new(id: Option<isize>, elems: Vec<Option<Expr>>, typ: Type, loc: Option<Loc>) -> Self {
        TupleExpr { id, elems, typ, loc }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for TupleExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let elems = self
            .elems
            .iter()
            .map(|e| match e {
                None => "".to_string(),
                Some(expr) => expr.to_string(),
            })
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "({elems})")
    }
}

//-------------------------------------------------------------------------
// Implementation for Type name expression
//-------------------------------------------------------------------------

impl TypeNameExpr {
    pub fn new(id: Option<isize>, typ: Type, loc: Option<Loc>) -> Self {
        TypeNameExpr { id, typ, loc }
    }

    pub fn update_data_type(&mut self, new_type: Type) {
        self.typ = new_type;
    }
}

impl Display for TypeNameExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // REVIEW: the logic here seems complex. Need to check other cases.
        match &self.typ {
            Type::Magic(MagicType::MetaType(typ)) => match typ.as_ref() {
                // Print `payable` instead of `address payable`
                Type::Address(address_typ) if address_typ.payable => {
                    write!(f, "payable")
                }
                // Only print `string`, without `data_loc`
                Type::String(_) => write!(f, "string"),
                _ => write!(f, "{typ}"),
            },
            // Only print `string`, without `data_loc`
            Type::String(_) => write!(f, "string"),
            _ => write!(f, "{}", self.typ),
        }
    }
}
