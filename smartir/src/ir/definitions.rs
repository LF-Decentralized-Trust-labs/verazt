use crate::ir::*;
use core::stdext::string::StringExt;
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing AST definitions
//-------------------------------------------------------------------------

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ContractDef {
    pub name: String,
    pub kind: ContractKind,
    pub elems: Vec<ContractElem>,
    pub loc: Option<Loc>,
}

// Contract kinds in IR only contain Contract and Interface.
//
// IR should not contain Library since all library functions will be normalized
// to normal functions.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ContractKind {
    Contract,
    Interface,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ContractElem {
    EventDef(EventDef),
    ErrorDef(ErrorDef),
    StructDef(StructDef),
    EnumDef(EnumDef),
    VarDecl(VarDecl),
    FuncDef(FuncDef),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub elems: Vec<String>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ErrorDef {
    pub name: String,
    pub params: Vec<VarDecl>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct EventDef {
    pub name: String,
    pub anon: bool,
    pub params: Vec<VarDecl>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FuncDef {
    pub name: String,
    pub is_virtual: bool,
    pub params: Vec<VarDecl>,
    pub returns: Vec<VarDecl>,
    pub body: Option<Block>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructField>,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub typ: Type,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub typ: Type,
    pub value: Option<Expr>,
    pub is_state_var: bool,
    pub data_loc: Option<DataLoc>,
    pub loc: Option<Loc>,
}

//-------------------------------------------------------------------------
// Implementations for contract definition
//-------------------------------------------------------------------------

impl ContractDef {
    pub fn new(
        name: String,
        kind: ContractKind,
        elems: Vec<ContractElem>,
        loc: Option<Loc>,
    ) -> Self {
        ContractDef { name, kind, elems, loc }
    }
}

impl Display for ContractDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} ", self.kind, self.name).ok();
        let body = self
            .elems
            .iter()
            .map(|elem| format!("{elem}").indent(4))
            .collect::<Vec<String>>()
            .join("\n\n");
        write!(f, "{{\n{body}\n}}")
    }
}

//-------------------------------------------------------------------------
// Implementations for contract kind
//-------------------------------------------------------------------------

impl Display for ContractKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractKind::Contract => write!(f, "contract"),
            ContractKind::Interface => write!(f, "interface"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for contract element
//-------------------------------------------------------------------------

impl ContractElem {}

impl From<EventDef> for ContractElem {
    fn from(event: EventDef) -> Self {
        ContractElem::EventDef(event)
    }
}

impl From<ErrorDef> for ContractElem {
    fn from(error: ErrorDef) -> Self {
        ContractElem::ErrorDef(error)
    }
}

impl From<StructDef> for ContractElem {
    fn from(struct_: StructDef) -> Self {
        ContractElem::StructDef(struct_)
    }
}

impl From<EnumDef> for ContractElem {
    fn from(enum_: EnumDef) -> Self {
        ContractElem::EnumDef(enum_)
    }
}

impl From<VarDecl> for ContractElem {
    fn from(var: VarDecl) -> Self {
        ContractElem::VarDecl(var)
    }
}

impl From<FuncDef> for ContractElem {
    fn from(func: FuncDef) -> Self {
        ContractElem::FuncDef(func)
    }
}

impl Display for ContractElem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractElem::EnumDef(enum_) => write!(f, "{enum_}"),
            ContractElem::ErrorDef(error) => write!(f, "{error}"),
            ContractElem::EventDef(event) => write!(f, "{event}"),
            ContractElem::FuncDef(func) => write!(f, "{func}"),
            ContractElem::StructDef(struct_) => write!(f, "{struct_}"),
            ContractElem::VarDecl(var) => write!(f, "{var};"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for enum definition
//-------------------------------------------------------------------------

impl EnumDef {
    pub fn new(name: String, elems: Vec<String>, loc: Option<Loc>) -> Self {
        EnumDef { name, elems, loc }
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
// Implementations for error definition
//-------------------------------------------------------------------------

impl ErrorDef {
    pub fn new(name: String, params: Vec<VarDecl>, loc: Option<Loc>) -> Self {
        ErrorDef { name, params, loc }
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
// Implementations for event definition
//-------------------------------------------------------------------------

impl EventDef {
    pub fn new(name: String, anon: bool, params: Vec<VarDecl>, loc: Option<Loc>) -> Self {
        EventDef { name, anon, params, loc }
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
        match self.anon {
            true => write!(f, " anonymous;"),
            false => write!(f, ";"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Function definition
//-------------------------------------------------------------------------

impl FuncDef {
    pub fn new(
        name: String,
        body: Option<Block>,
        is_virtual: bool,
        params: Vec<VarDecl>,
        returns: Vec<VarDecl>,
        loc: Option<Loc>,
    ) -> Self {
        FuncDef { name, body, is_virtual, params, returns, loc }
    }
}

impl Display for FuncDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.name.is_empty() {
            write!(f, " {}", self.name).ok();
        }

        let params = self
            .params
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "({params})").ok();

        if !self.returns.is_empty() {
            let returns = self
                .returns
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            write!(f, " returns ({returns})").ok();
        }

        match &self.body {
            None => write!(f, ";"),
            Some(body) => writeln!(f, " {body}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for struct definition
//-------------------------------------------------------------------------

impl StructDef {
    pub fn new(name: String, fields: Vec<StructField>, loc: Option<Loc>) -> Self {
        StructDef { name, fields, loc }
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
// Implementations for struct field
//-------------------------------------------------------------------------

impl StructField {
    pub fn new(name: String, typ: Type, loc: Option<Loc>) -> Self {
        StructField { name, typ, loc }
    }
}

impl Display for StructField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.typ, self.name)
    }
}

//-------------------------------------------------------------------------
// Implementations for variable declaration
//-------------------------------------------------------------------------

impl VarDecl {
    pub fn new(
        name: String,
        typ: Type,
        value: Option<Expr>,
        state_var: bool,
        data_loc: Option<DataLoc>,
        loc: Option<Loc>,
    ) -> Self {
        VarDecl { name, typ, value, data_loc, is_state_var: state_var, loc }
    }
}

impl Display for VarDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.typ).ok();
        // Data location must be printed for some types
        let need_to_print_data_loc = match &self.typ {
            Type::Array(_) | Type::Struct(_) | Type::Mapping(_) | Type::String(_) => true,
            Type::Bytes(typ) => typ.length.is_none(),
            _ => false,
        };
        if self.typ.data_loc() != DataLoc::None && need_to_print_data_loc {
            if let Some(data_loc) = &self.data_loc {
                write!(f, " {data_loc}").ok();
            }
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
