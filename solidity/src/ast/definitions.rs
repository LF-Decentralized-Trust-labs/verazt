use crate::{ast::*, version};
use extlib::{error::Result, fail, string::StringExt};
use meta::DataLoc;
use node_semver::Range;
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing definitions
//-------------------------------------------------------------------------

/// Contract definition.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ContractDef {
    pub id: Option<isize>,
    pub scope_id: Option<isize>,
    pub name: Name,
    pub kind: ContractKind,
    pub is_abstract: bool,
    pub base_contracts: Vec<BaseContract>,
    pub body: Vec<ContractElem>,
    pub loc: Option<Loc>,
}

/// Contract kind.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ContractKind {
    Contract,
    Interface,
    Library,
}

/// Base contract.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BaseContract {
    pub name: Name,
    pub args: Vec<Expr>,
    pub loc: Option<Loc>,
}

/// Contract element.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ContractElem {
    Using(UsingDir),
    Error(ErrorDef),
    Event(EventDef),
    Struct(StructDef),
    Enum(EnumDef),
    Type(TypeDef),
    Var(VarDecl),
    Func(FuncDef),
}

/// Enum definition.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct EnumDef {
    pub id: Option<isize>,
    pub scope_id: Option<isize>,
    pub name: Name,
    pub elems: Vec<String>,
    pub loc: Option<Loc>,
}

/// Error definition.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErrorDef {
    pub name: Name,
    pub params: Vec<VarDecl>,
    pub loc: Option<Loc>,
}

/// Event definition.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct EventDef {
    pub name: Name,
    pub is_anonymous: bool,
    pub params: Vec<VarDecl>,
    pub loc: Option<Loc>,
}

/// Function definition.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FuncDef {
    pub id: Option<isize>,
    pub scope_id: Option<isize>, // ID of the scope where the function is define.
    pub name: Name,
    pub kind: FuncKind,
    pub is_virtual: bool,
    pub visibility: FuncVis,
    pub mutability: FuncMut,
    pub modifier_invocs: Vec<CallExpr>,
    pub overriding: Overriding,
    pub params: Vec<VarDecl>,
    pub returns: Vec<VarDecl>,
    pub body: Option<Block>,
    pub loc: Option<Loc>,
    pub sol_ver: Option<node_semver::Range>,
}

/// Function kind.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum FuncKind {
    Constructor,
    Receive,
    Fallback,
    ContractFunc,
    FreeFunc,
    Modifier,
}

/// Struct definition.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct StructDef {
    pub id: Option<isize>,
    pub scope_id: Option<isize>,
    pub name: Name,
    pub fields: Vec<StructField>,
    pub loc: Option<Loc>,
}

/// Struct field.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct StructField {
    pub id: Option<isize>,
    pub name: String,
    pub typ: Type,
    pub loc: Option<Loc>,
}

/// User-defined type definition.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TypeDef {
    pub id: Option<isize>,
    pub scope_id: Option<isize>,
    pub name: Name,
    pub base_typ: Type,
    pub loc: Option<Loc>,
}

/// Variable declaration.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct VarDecl {
    pub id: Option<isize>,
    pub scope_id: Option<isize>,
    pub name: Name,
    pub typ: Type,
    pub value: Option<Expr>,
    pub mutability: VarMut,
    pub is_state_var: bool,
    pub overriding: Overriding,
    pub visibility: VarVis,
    pub data_loc: Option<DataLoc>,
    pub loc: Option<Loc>,
}

//-------------------------------------------------------------------------
// Implementation for Contract definition
//-------------------------------------------------------------------------

impl ContractDef {
    pub fn new(
        id: Option<isize>,
        scope_id: Option<isize>,
        name: Name,
        kind: ContractKind,
        is_abstract: bool,
        base_contracts: Vec<BaseContract>,
        body: Vec<ContractElem>,
        loc: Option<Loc>,
    ) -> Self {
        ContractDef { id, scope_id, name, kind, is_abstract, base_contracts, body, loc }
    }

    pub fn functions(&self) -> Vec<FuncDef> {
        self.body
            .iter()
            .filter_map(|elem| match elem {
                ContractElem::Func(func) => Some(func.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn get_user_defined_types(&self) -> Vec<Type> {
        self.body
            .iter()
            .filter_map(|elem| {
                let contract = Some(self.name.clone());
                match elem {
                    ContractElem::Struct(s) => {
                        let t = StructType::new(s.name.clone(), contract, DataLoc::None, false);
                        Some(Type::from(t))
                    }
                    ContractElem::Enum(e) => {
                        let t = EnumType::new(e.name.clone(), contract);
                        Some(Type::from(t))
                    }
                    ContractElem::Type(t) => {
                        let t = UserDefinedType::new(t.name.clone(), contract);
                        Some(Type::from(t))
                    }
                    ContractElem::Var(_) => todo!(),
                    ContractElem::Func(_) => todo!(),
                    _ => todo!(),
                }
            })
            .collect()
    }

    pub fn find_function_def(&self, func_name: &str) -> Option<&FuncDef> {
        for elem in self.body.iter() {
            if let ContractElem::Func(func) = elem
                && (func.name.original_name() == func_name
                    || (func_name == "fallback" && func.is_fallback_function()))
            {
                return Some(func);
            }
        }
        None
    }
}

impl Display for ContractDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_abstract {
            write!(f, "abstract ").ok();
        }
        write!(f, "{} {} ", self.kind, self.name).ok();
        if !self.base_contracts.is_empty() {
            let base_contracts = self
                .base_contracts
                .iter()
                .map(|base| format!("{}", base.name))
                .collect::<Vec<String>>()
                .join(", ");
            write!(f, "is {base_contracts} ").ok();
        }
        let body = self
            .body
            .iter()
            .map(|elem| format!("{elem}").indent(4))
            .collect::<Vec<String>>()
            .join("\n\n");
        write!(f, "{{\n{body}\n}}")
    }
}

//-------------------------------------------------------------------------
// Implementation for Contract Kind
//-------------------------------------------------------------------------

impl ContractKind {
    pub fn from_string(kind: &str) -> Result<Self> {
        match kind {
            "contract" => Ok(ContractKind::Contract),
            "interface" => Ok(ContractKind::Interface),
            "library" => Ok(ContractKind::Library),
            _ => fail!("Unknown contract kind: {}", kind),
        }
    }
}

impl Display for ContractKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractKind::Contract => write!(f, "contract"),
            ContractKind::Interface => write!(f, "interface"),
            ContractKind::Library => write!(f, "library"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Base Contract
//-------------------------------------------------------------------------

impl BaseContract {
    pub fn new(name: Name, args: Vec<Expr>, loc: Option<Loc>) -> Self {
        BaseContract { name, args, loc }
    }
}

impl Display for BaseContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.args.is_empty() {
            write!(f, "{}", self.name)
        } else {
            let args = self
                .args
                .iter()
                .map(|arg| format!("{arg}"))
                .collect::<Vec<String>>()
                .join(", ");
            write!(f, "{}({})", self.name, args)
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Contract elements
//-------------------------------------------------------------------------

impl ContractElem {}

impl From<UsingDir> for ContractElem {
    fn from(using: UsingDir) -> Self {
        ContractElem::Using(using)
    }
}

impl From<EventDef> for ContractElem {
    fn from(event: EventDef) -> Self {
        ContractElem::Event(event)
    }
}

impl From<ErrorDef> for ContractElem {
    fn from(error: ErrorDef) -> Self {
        ContractElem::Error(error)
    }
}

impl From<StructDef> for ContractElem {
    fn from(struct_: StructDef) -> Self {
        ContractElem::Struct(struct_)
    }
}

impl From<EnumDef> for ContractElem {
    fn from(enum_: EnumDef) -> Self {
        ContractElem::Enum(enum_)
    }
}

impl From<TypeDef> for ContractElem {
    fn from(typ: TypeDef) -> Self {
        ContractElem::Type(typ)
    }
}

impl From<VarDecl> for ContractElem {
    fn from(var: VarDecl) -> Self {
        ContractElem::Var(var)
    }
}

impl From<FuncDef> for ContractElem {
    fn from(func: FuncDef) -> Self {
        ContractElem::Func(func)
    }
}

impl Display for ContractElem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractElem::Enum(e) => write!(f, "{e}"),
            ContractElem::Error(e) => write!(f, "{e}"),
            ContractElem::Event(e) => write!(f, "{e}"),
            ContractElem::Func(x) => write!(f, "{x}"),
            ContractElem::Struct(s) => write!(f, "{s}"),
            ContractElem::Type(t) => write!(f, "{t}"),
            ContractElem::Using(u) => write!(f, "{u}"),
            ContractElem::Var(v) => write!(f, "{v};"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Enum definition
//-------------------------------------------------------------------------

impl EnumDef {
    pub fn new(
        id: Option<isize>,
        scope: Option<isize>,
        name: Name,
        elems: Vec<String>,
        loc: Option<Loc>,
    ) -> Self {
        EnumDef { id, scope_id: scope, name, elems, loc }
    }
}

impl Display for EnumDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "enum {} {{", self.name).ok();
        let elems = self
            .elems
            .iter()
            .map(|elem| elem.indent(4))
            .collect::<Vec<String>>()
            .join(",\n");
        write!(f, "{elems}\n}}")
    }
}

//-------------------------------------------------------------------------
// Implementation for Error definition
//-------------------------------------------------------------------------

impl ErrorDef {
    pub fn new(name: Name, params: Vec<VarDecl>, loc: Option<Loc>) -> Self {
        ErrorDef { name, params, loc }
    }

    pub fn get_type(&self) -> Type {
        let param_typs = self.params.iter().map(|p| p.typ.clone()).collect();
        FuncType::new(param_typs, vec![], FuncVis::None, FuncMut::None).into()
    }
}

impl Display for ErrorDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params = self
            .params
            .iter()
            .map(|p| p.name.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "error {} ({});", self.name, params)
    }
}

//-------------------------------------------------------------------------
// Implementation for Event definition
//-------------------------------------------------------------------------

impl EventDef {
    pub fn new(name: Name, anon: bool, params: Vec<VarDecl>, loc: Option<Loc>) -> Self {
        EventDef { name, is_anonymous: anon, params, loc }
    }
}

impl Display for EventDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params = self
            .params
            .iter()
            .map(|p| p.name.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        write!(f, "event {} ({})", self.name, params).ok();

        match self.is_anonymous {
            true => write!(f, " anonymous;"),
            false => write!(f, ";"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Function definition
//-------------------------------------------------------------------------

impl FuncDef {
    pub fn new(
        id: Option<isize>,
        scope: Option<isize>,
        name: Name,
        kind: FuncKind,
        body: Option<Block>,
        is_virtual: bool,
        visibility: FuncVis,
        mutability: FuncMut,
        params: Vec<VarDecl>,
        modifiers: Vec<CallExpr>,
        overriding: Overriding,
        returns: Vec<VarDecl>,
        loc: Option<Loc>,
        sol_ver: Option<Range>,
    ) -> Self {
        FuncDef {
            id,
            scope_id: scope,
            name,
            body,
            kind,
            is_virtual,
            visibility,
            mutability,
            params,
            modifier_invocs: modifiers,
            overriding,
            returns,
            loc,
            sol_ver,
        }
    }

    pub fn is_fallback_function(&self) -> bool {
        self.kind.eq(&FuncKind::Fallback)
    }

    pub fn is_receive_function(&self) -> bool {
        self.kind.eq(&FuncKind::Receive)
    }

    pub fn is_free_function(&self) -> bool {
        self.kind.eq(&FuncKind::FreeFunc)
    }

    pub fn typ(&self) -> Type {
        let params = self.params.iter().map(|p| p.typ.clone()).collect();
        let returns = self.returns.iter().map(|p| p.typ.clone()).collect();
        FuncType::new(params, returns, self.visibility.clone(), self.mutability.clone()).into()
    }

    pub fn set_naming_index(&mut self, index: Option<usize>) {
        self.name.index = index
    }
}

impl Display for FuncDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(range) = &self.sol_ver {
            if version::check_range_constraint(range, ">=0.6.0") {
                write!(f, "{}", self.kind).ok();
            } else if self.is_fallback_function() || !self.name.is_empty() {
                write!(f, "function").ok();
            } else {
                write!(f, "{}", self.kind).ok();
            }
        } else {
            write!(f, "{}", self.kind).ok();
        }

        if !self.name.is_empty() {
            write!(f, " {}", self.name).ok();
        }

        let params = self
            .params
            .iter()
            .map(|p| p.name.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "({params})").ok();

        // Do not print visibility for free function
        if self.kind != FuncKind::FreeFunc {
            let visibility = format!("{}", self.visibility);
            if !visibility.is_empty() {
                write!(f, " {visibility}").ok();
            }
        }

        let mutability = format!("{}", self.mutability);
        if !mutability.is_empty() {
            write!(f, " {mutability}").ok();
        }

        if self.is_virtual {
            write!(f, " virtual").ok();
        }

        if !self.modifier_invocs.is_empty() {
            let minvocs = self
                .modifier_invocs
                .iter()
                .map(|modifier| format!("{modifier}"))
                .collect::<Vec<String>>()
                .join(" ");
            write!(f, " {minvocs}").ok();
        }

        let overriding = format!("{}", self.overriding);
        if !overriding.is_empty() {
            write!(f, " {overriding}").ok();
        }

        if !self.returns.is_empty() {
            let returns = self
                .returns
                .iter()
                .map(|v| format!("{v}"))
                .collect::<Vec<String>>()
                .join(", ");
            write!(f, " returns ({returns})").ok();
        }

        match &self.body {
            None => write!(f, ";"),
            Some(body) => write!(f, " {body}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Function Kind
//-------------------------------------------------------------------------

impl FuncKind {
    pub fn new(kind: &str) -> Result<Self> {
        match kind {
            "constructor" => Ok(FuncKind::Constructor),
            "receive" => Ok(FuncKind::Receive),
            "fallback" => Ok(FuncKind::Fallback),
            "function" => Ok(FuncKind::ContractFunc),
            "freeFunction" => Ok(FuncKind::FreeFunc),
            "modifier" => Ok(FuncKind::Modifier),
            _ => fail!("Unknown function kind: {kind}"),
        }
    }
}

impl Display for FuncKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FuncKind::Constructor => write!(f, "constructor"),
            FuncKind::Receive => write!(f, "receive"),
            FuncKind::Fallback => write!(f, "fallback"),
            FuncKind::ContractFunc => write!(f, "function"),
            FuncKind::FreeFunc => write!(f, "function"),
            FuncKind::Modifier => write!(f, "modifier"),
        }
    }
}

//-------------------------------------------------------------------------
//  Implementation for Struct definition
//-------------------------------------------------------------------------

impl StructDef {
    pub fn new(
        id: Option<isize>,
        scope: Option<isize>,
        name: Name,
        fields: Vec<StructField>,
        loc: Option<Loc>,
    ) -> Self {
        StructDef { id, scope_id: scope, name, fields, loc }
    }
}

impl Display for StructDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "struct {} {{", self.name).ok();
        let fields = self
            .fields
            .iter()
            .map(|field| format!("{field};").indent(4))
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "{fields}\n}}")
    }
}

//-------------------------------------------------------------------------
// Implementation for Struct field
//-------------------------------------------------------------------------

impl StructField {
    pub fn new(id: Option<isize>, name: String, typ: Type, loc: Option<Loc>) -> Self {
        StructField { id, name, typ, loc }
    }
}

impl Display for StructField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.typ, self.name)
    }
}

//-------------------------------------------------------------------------
// Implementation for User-defined type definition
//-------------------------------------------------------------------------

impl TypeDef {
    pub fn new(
        id: Option<isize>,
        scope: Option<isize>,
        name: Name,
        base: Type,
        loc: Option<Loc>,
    ) -> Self {
        TypeDef { id, scope_id: scope, name, base_typ: base, loc }
    }
}

impl Display for TypeDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type {} is {};", self.name, self.base_typ)
    }
}

//-------------------------------------------------------------------------
// Implementation for Variable declaration
//-------------------------------------------------------------------------

impl VarDecl {
    pub fn new(
        id: Option<isize>,
        scope: Option<isize>,
        name: Name,
        typ: Type,
        value: Option<Expr>,
        mutability: VarMut,
        is_state_var: bool,
        visibility: VarVis,
        data_loc: Option<DataLoc>,
        overriding: Overriding,
        loc: Option<Loc>,
    ) -> Self {
        VarDecl {
            id,
            scope_id: scope,
            name,
            typ,
            value,
            data_loc,
            mutability,
            is_state_var,
            visibility,
            overriding,
            loc,
        }
    }

    pub fn set_naming_index(&mut self, index: Option<usize>) {
        self.name.index = index
    }
}

impl Display for VarDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.typ).ok();

        // Decide if data location of the variable declaration needs to be printed.
        let need_to_print_data_loc = match &self.typ {
            Type::Array(_) | Type::Struct(_) | Type::Mapping(_) | Type::String(_) => true,
            Type::Bytes(typ) => !self.is_state_var && typ.length.is_none(),
            _ => false,
        };
        if self.typ.data_loc() != DataLoc::None && need_to_print_data_loc {
            if let Some(data_loc) = &self.data_loc {
                write!(f, " {data_loc}").ok();
            }
        }

        let visibility = format!("{}", self.visibility);
        if !visibility.is_empty() {
            write!(f, " {visibility}").ok();
        }

        if !matches!(&self.mutability, VarMut::Mutable) {
            write!(f, " {}", self.mutability).ok();
        }

        let overriding = format!("{}", self.overriding);
        if !overriding.is_empty() {
            write!(f, " {overriding}").ok();
        }

        if !self.name.is_empty() {
            write!(f, " {}", self.name).ok();
        }

        if let Some(value) = &self.value {
            write!(f, " = {value}").ok();
        }

        Ok(())
    }
}
