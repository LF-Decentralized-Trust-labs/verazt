use crate::ir::*;
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing all expressions
//-------------------------------------------------------------------------

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Expr {
    Lit(Lit),
    Var(Variable),
    Call(CallExpr),
    Tuple(TupleExpr),
    Index(IndexExpr),
    Slice(SliceExpr),
    Member(MemberExpr),
    InlineArray(InlineArrayExpr),
    New(NewExpr),
    TypeName(TypeNameExpr),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CallExpr {
    pub callee: CalleeExpr,
    pub call_opts: Vec<CallOption>,
    pub args: Vec<AtomicExpr>,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CallOption {
    pub name: String,
    pub value: Expr,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CalleeExpr {
    BuiltIn(String),
    ContractDef(ContractDef),
    FuncDef(FuncDef),
    StructDef(StructDef),
    MemberExpr(MemberExpr),
    NewExpr(NewExpr),
    TypeNameExpr(TypeNameExpr),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ConditionalExpr {
    pub cond: Box<AtomicExpr>,
    pub true_br: Box<AtomicExpr>,
    pub false_br: Box<AtomicExpr>,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct IndexExpr {
    pub base: Variable,
    pub index: Option<Box<AtomicExpr>>,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct InlineArrayExpr {
    pub elems: Vec<AtomicExpr>,
    pub typ: Type,
    pub loc: Option<Loc>,
}

/// Member access expressions, such as field access, contract access, etc.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MemberExpr {
    pub base: Box<Expr>,
    pub member: String,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct NewExpr {
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SliceExpr {
    pub base: Variable,
    pub start_index: Option<Box<AtomicExpr>>,
    pub end_index: Option<Box<AtomicExpr>>,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TupleExpr {
    pub elems: Vec<Option<AtomicExpr>>,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TypeNameExpr {
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum AtomicExpr {
    Lit(Lit),
    Var(Variable),
    Type(TypeNameExpr),
}

//-------------------------------------------------------------------------
// Implementations for expressions
//-------------------------------------------------------------------------

impl Expr {
    pub fn typ(&self) -> Type {
        match self {
            Expr::Var(v) => v.typ.clone(),
            Expr::Lit(l) => l.typ(),
            Expr::Call(e) => e.typ.clone(),
            Expr::Tuple(e) => e.typ.clone(),
            Expr::Index(e) => e.typ.clone(),
            Expr::Slice(e) => e.typ.clone(),
            Expr::Member(e) => e.typ.clone(),
            Expr::InlineArray(e) => e.typ.clone(),
            Expr::New(e) => e.typ.clone(),
            Expr::TypeName(e) => e.typ.clone(),
        }
    }

    pub fn loc(&self) -> Option<Loc> {
        match self {
            Expr::Var(v) => v.loc,
            Expr::Lit(l) => l.loc(),
            Expr::Call(e) => e.loc,
            Expr::Tuple(e) => e.loc,
            Expr::Index(e) => e.loc,
            Expr::Slice(e) => e.loc,
            Expr::Member(e) => e.loc,
            Expr::InlineArray(e) => e.loc,
            Expr::New(e) => e.loc,
            Expr::TypeName(e) => e.loc,
        }
    }
}

impl From<Lit> for Expr {
    fn from(lit: Lit) -> Self {
        Expr::Lit(lit)
    }
}

impl From<Variable> for Expr {
    fn from(var: Variable) -> Self {
        Expr::Var(var)
    }
}

impl From<CallExpr> for Expr {
    fn from(expr: CallExpr) -> Expr {
        Expr::Call(expr)
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

impl From<AtomicExpr> for Expr {
    fn from(expr: AtomicExpr) -> Expr {
        match expr {
            AtomicExpr::Lit(l) => Expr::from(l),
            AtomicExpr::Var(v) => Expr::from(v),
            AtomicExpr::Type(e) => Expr::from(e),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Lit(c) => write!(f, "{c}"),
            Expr::Var(v) => write!(f, "{v}"),
            Expr::New(e) => write!(f, "new {e}"),
            Expr::Tuple(e) => write!(f, "{e}"),
            Expr::Index(e) => write!(f, "{e}"),
            Expr::Slice(e) => write!(f, "{e}"),
            Expr::Member(e) => write!(f, "{e}"),
            Expr::Call(e) => write!(f, "{e}"),
            Expr::TypeName(typ) => write!(f, "{typ}"),
            Expr::InlineArray(e) => write!(f, "{e}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Atomic expressions
//-------------------------------------------------------------------------

impl AtomicExpr {
    pub fn data_type(&self) -> Type {
        match self {
            AtomicExpr::Lit(l) => l.typ(),
            AtomicExpr::Var(v) => v.typ.clone(),
            AtomicExpr::Type(e) => e.typ.clone(),
        }
    }

    pub fn loc(&self) -> Option<Loc> {
        match self {
            AtomicExpr::Lit(l) => l.loc(),
            AtomicExpr::Var(v) => v.loc,
            AtomicExpr::Type(e) => e.loc,
        }
    }
}

impl From<Lit> for AtomicExpr {
    fn from(l: Lit) -> Self {
        AtomicExpr::Lit(l)
    }
}

impl From<Variable> for AtomicExpr {
    fn from(v: Variable) -> Self {
        AtomicExpr::Var(v)
    }
}

impl From<TypeNameExpr> for AtomicExpr {
    fn from(e: TypeNameExpr) -> Self {
        AtomicExpr::Type(e)
    }
}

impl TryFrom<Expr> for AtomicExpr {
    type Error = String;
    fn try_from(e: Expr) -> Result<Self, Self::Error> {
        match e {
            Expr::Lit(l) => Ok(l.into()),
            Expr::Var(v) => Ok(v.into()),
            Expr::TypeName(t) => Ok(t.into()),
            _ => Err(format!("Unable to convert '{e}' into AtomicExpr")),
        }
    }
}

impl Display for AtomicExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AtomicExpr::Lit(c) => write!(f, "{c}"),
            AtomicExpr::Var(v) => write!(f, "{v}"),
            AtomicExpr::Type(e) => write!(f, "{e}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Call expression
//-------------------------------------------------------------------------

impl CallExpr {
    pub fn new(
        callee: CalleeExpr,
        call_opts: Vec<CallOption>,
        args: Vec<AtomicExpr>,
        typ: Type,
        loc: Option<Loc>,
    ) -> Self {
        CallExpr { callee, call_opts, args, typ, loc }
    }
}

impl Display for CallExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args = self
            .args
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{}({})", self.callee, args)
    }
}

//-------------------------------------------------------------------------
// Implementations for Call option
//-------------------------------------------------------------------------

impl CallOption {
    pub fn new(name: String, value: Expr, loc: Option<Loc>) -> Self {
        CallOption { name, value, loc }
    }
}

impl Display for CallOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

//-------------------------------------------------------------------------
// Implementations for Callee expression
//-------------------------------------------------------------------------

impl Display for CalleeExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalleeExpr::BuiltIn(n) => write!(f, "{n}"),
            CalleeExpr::ContractDef(c) => write!(f, "{}", c.name),
            CalleeExpr::FuncDef(func) => write!(f, "{}", func.name),
            CalleeExpr::StructDef(s) => write!(f, "{}", s.name),
            CalleeExpr::MemberExpr(e) => write!(f, "{e}"),
            CalleeExpr::NewExpr(e) => write!(f, "{e}"),
            CalleeExpr::TypeNameExpr(e) => write!(f, "{e}"),
        }
    }
}

impl From<&str> for CalleeExpr {
    fn from(name: &str) -> Self {
        CalleeExpr::BuiltIn(name.to_string())
    }
}

impl From<FuncDef> for CalleeExpr {
    fn from(value: FuncDef) -> Self {
        CalleeExpr::FuncDef(value)
    }
}

impl From<ContractDef> for CalleeExpr {
    fn from(value: ContractDef) -> Self {
        CalleeExpr::ContractDef(value)
    }
}

impl From<StructDef> for CalleeExpr {
    fn from(value: StructDef) -> Self {
        CalleeExpr::StructDef(value)
    }
}

impl From<MemberExpr> for CalleeExpr {
    fn from(value: MemberExpr) -> Self {
        CalleeExpr::MemberExpr(value)
    }
}

impl From<NewExpr> for CalleeExpr {
    fn from(value: NewExpr) -> Self {
        CalleeExpr::NewExpr(value)
    }
}

impl From<TypeNameExpr> for CalleeExpr {
    fn from(value: TypeNameExpr) -> Self {
        CalleeExpr::TypeNameExpr(value)
    }
}

impl TryFrom<Expr> for CalleeExpr {
    type Error = String;

    fn try_from(value: Expr) -> Result<Self, Self::Error> {
        match value {
            Expr::Member(expr) => Ok(CalleeExpr::from(expr)),
            _ => Err(format!("Implement CallableExpr::from for expression: {value}")),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Conditional expression
//-------------------------------------------------------------------------

impl ConditionalExpr {
    pub fn new(
        cond: AtomicExpr,
        true_br: AtomicExpr,
        false_br: AtomicExpr,
        typ: Type,
        loc: Option<Loc>,
    ) -> Self {
        ConditionalExpr {
            cond: Box::new(cond),
            true_br: Box::new(true_br),
            false_br: Box::new(false_br),
            typ,
            loc,
        }
    }
}

impl Display for ConditionalExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ? {} : {}", self.cond, self.true_br, self.false_br)
    }
}

//-------------------------------------------------------------------------
// Implementations for Index access expression
//-------------------------------------------------------------------------

impl IndexExpr {
    pub fn new(base: Variable, index: Option<AtomicExpr>, typ: Type, loc: Option<Loc>) -> Self {
        IndexExpr { base, index: index.map(Box::new), typ, loc }
    }
}

impl Display for IndexExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.index {
            Some(index) => write!(f, "{}[{}]", self.base, index),
            None => write!(f, "{}[]", self.base),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Inline array expression
//-------------------------------------------------------------------------

impl InlineArrayExpr {
    pub fn new(elems: Vec<AtomicExpr>, typ: Type, loc: Option<Loc>) -> Self {
        InlineArrayExpr { elems, typ, loc }
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
// Implementations for Member access expression
//-------------------------------------------------------------------------

impl MemberExpr {
    pub fn new(base: Box<Expr>, member_name: String, typ: Type, loc: Option<Loc>) -> Self {
        MemberExpr { base, member: member_name, typ, loc }
    }
}

impl Display for MemberExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.base, self.member)
    }
}

//-------------------------------------------------------------------------
// Implementations for New expression
//-------------------------------------------------------------------------

impl NewExpr {
    pub fn new(typ: Type, loc: Option<Loc>) -> Self {
        NewExpr { typ, loc }
    }
}

impl Display for NewExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "new {}", self.typ)
    }
}

//-------------------------------------------------------------------------
// Implementations for Slice expression
//-------------------------------------------------------------------------

impl SliceExpr {
    pub fn new(
        base: Variable,
        start: Option<AtomicExpr>,
        end: Option<AtomicExpr>,
        typ: Type,
        loc: Option<Loc>,
    ) -> Self {
        SliceExpr {
            base,
            start_index: start.map(Box::new),
            end_index: end.map(Box::new),
            typ,
            loc,
        }
    }
}

impl Display for SliceExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[", self.base).ok();
        if let Some(expr) = &self.start_index {
            write!(f, "{expr}").ok();
        }
        write!(f, ":").ok();
        if let Some(expr) = &self.end_index {
            write!(f, "{expr}").ok();
        }

        write!(f, "]")
    }
}

//-------------------------------------------------------------------------
// Implementations for Tuple expression
//-------------------------------------------------------------------------

impl TupleExpr {
    pub fn new(elems: Vec<Option<AtomicExpr>>, typ: Type, loc: Option<Loc>) -> Self {
        TupleExpr { elems, typ, loc }
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
// Implementations for Type name expression
//-------------------------------------------------------------------------

impl TypeNameExpr {
    pub fn new(typ: Type, loc: Option<Loc>) -> Self {
        TypeNameExpr { typ, loc }
    }
}

impl Display for TypeNameExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.typ {
            Type::Magic(MagicType::MetaType(typ)) => match typ.as_ref() {
                // Print `payable` instead of `address payable`
                Type::Address(address_typ) if address_typ.payable => {
                    write!(f, "payable")
                }
                _ => write!(f, "{typ}"),
            },
            _ => write!(f, "{}", self.typ),
        }
    }
}
