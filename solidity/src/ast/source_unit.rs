use crate::ast::*;
use bat::PrettyPrinter;
use meta::Name;
use std::{
    collections::HashMap,
    fmt::{self, Display},
};

//-------------------------------------------------------------------------
// Data structures representing source units
//-------------------------------------------------------------------------

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SourceUnit {
    pub id: Option<isize>,
    pub path: String,
    pub elems: Vec<SourceUnitElem>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SourceUnitElem {
    Pragma(PragmaDir),
    Import(ImportDir),
    Using(UsingDir),
    Error(ErrorDef),
    Func(FuncDef),
    Var(VarDecl),
    Struct(StructDef),
    Enum(EnumDef),
    UserType(TypeDef),
    Contract(ContractDef),
}

//-------------------------------------------------------------------------
// Implementations for source unit
//-------------------------------------------------------------------------

impl SourceUnit {
    pub fn new(id: Option<isize>, path: String, elems: Vec<SourceUnitElem>) -> Self {
        SourceUnit { id, path, elems }
    }

    pub fn get_solidity_pragma_versions(&self) -> Vec<String> {
        let mut versions = vec![];
        for elem in self.elems.iter() {
            if let SourceUnitElem::Pragma(pragma) = elem {
                if let PragmaKind::Version(ver) = &pragma.kind {
                    versions.push(ver.clone())
                }
            }
        }
        versions
    }

    pub fn find_contract_def(&self, name: &Name) -> Option<&ContractDef> {
        for elem in self.elems.iter() {
            if let SourceUnitElem::Contract(contract) = elem {
                if &contract.name == name {
                    return Some(contract);
                }
            }
        }
        None
    }

    pub fn find_struct_def(&self, name: &Name) -> Option<&StructDef> {
        for elem in self.elems.iter() {
            if let SourceUnitElem::Struct(struct_) = elem {
                if &struct_.name == name {
                    return Some(struct_);
                }
            }
        }
        None
    }

    pub fn find_contract_def_by_base_name(&self, name: &Name) -> Option<&ContractDef> {
        for elem in self.elems.iter() {
            if let SourceUnitElem::Contract(contract) = elem {
                if contract.name.base == name.base {
                    return Some(contract);
                }
            }
        }
        None
    }

    pub fn find_imported_source_unit(&self, name: &Name) -> Option<&ContractDef> {
        for elem in self.elems.iter() {
            if let SourceUnitElem::Contract(contract) = elem {
                if contract.name.base == name.base {
                    return Some(contract);
                }
            }
        }
        None
    }

    pub fn construct_contract_map<'a>(&'a self) -> HashMap<Name, &'a ContractDef> {
        let mut contract_map: HashMap<Name, &'a ContractDef> = HashMap::new();
        for source_unit_elem in &self.elems {
            if let SourceUnitElem::Contract(contract) = source_unit_elem {
                contract_map.insert(contract.name.clone(), contract);
            }
        }
        contract_map
    }

    pub fn print_highlighted_code(&self) {
        let str = &format!("{self}");
        PrettyPrinter::new()
            .theme("Visual Studio Dark+")
            .line_numbers(true)
            .input_from_bytes(str.as_bytes())
            .language("solidity")
            .print()
            .unwrap_or_default();
        print!("");
    }
}

impl Display for SourceUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "// File: {}", &self.path).ok();
        match self.elems.is_empty() {
            true => write!(f, "<Empty source unit>"),
            false => {
                let elems = self
                    .elems
                    .iter()
                    .map(|elem| format!("{elem}"))
                    .collect::<Vec<String>>()
                    .join("\n\n");
                write!(f, "{elems}")
            }
        }
    }
}

//-------------------------------------------------
// Implementations for source unit element
//-------------------------------------------------

impl SourceUnitElem {
    pub fn get_name(&self) -> Option<Name> {
        match self {
            SourceUnitElem::Pragma(_) => None,
            SourceUnitElem::Import(_) => None,
            SourceUnitElem::Using(_) => None,
            SourceUnitElem::Error(e) => Some(e.name.clone()),
            SourceUnitElem::Func(f) => Some(f.name.clone()),
            SourceUnitElem::Var(v) => Some(v.name.clone()),
            SourceUnitElem::UserType(t) => Some(t.name.clone()),
            SourceUnitElem::Struct(s) => Some(s.name.clone()),
            SourceUnitElem::Enum(e) => Some(e.name.clone()),
            SourceUnitElem::Contract(c) => Some(c.name.clone()),
        }
    }
}

impl From<PragmaDir> for SourceUnitElem {
    fn from(p: PragmaDir) -> Self {
        SourceUnitElem::Pragma(p)
    }
}

impl From<ImportDir> for SourceUnitElem {
    fn from(i: ImportDir) -> Self {
        SourceUnitElem::Import(i)
    }
}

impl From<UsingDir> for SourceUnitElem {
    fn from(u: UsingDir) -> Self {
        SourceUnitElem::Using(u)
    }
}

impl From<ErrorDef> for SourceUnitElem {
    fn from(e: ErrorDef) -> Self {
        SourceUnitElem::Error(e)
    }
}

impl From<FuncDef> for SourceUnitElem {
    fn from(fd: FuncDef) -> Self {
        SourceUnitElem::Func(fd)
    }
}

impl From<VarDecl> for SourceUnitElem {
    fn from(v: VarDecl) -> Self {
        SourceUnitElem::Var(v)
    }
}

impl From<TypeDef> for SourceUnitElem {
    fn from(t: TypeDef) -> Self {
        SourceUnitElem::UserType(t)
    }
}

impl From<StructDef> for SourceUnitElem {
    fn from(s: StructDef) -> Self {
        SourceUnitElem::Struct(s)
    }
}

impl From<EnumDef> for SourceUnitElem {
    fn from(e: EnumDef) -> Self {
        SourceUnitElem::Enum(e)
    }
}

impl From<ContractDef> for SourceUnitElem {
    fn from(c: ContractDef) -> Self {
        SourceUnitElem::Contract(c)
    }
}

impl Display for SourceUnitElem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceUnitElem::Pragma(p) => write!(f, "{p}"),
            SourceUnitElem::Import(i) => write!(f, "{i}"),
            SourceUnitElem::Using(u) => write!(f, "{u}"),
            SourceUnitElem::Contract(c) => write!(f, "{c}"),
            SourceUnitElem::Func(fd) => write!(f, "{fd}"),
            SourceUnitElem::Var(v) => write!(f, "{v};"),
            SourceUnitElem::Struct(s) => write!(f, "{s}"),
            SourceUnitElem::Enum(e) => write!(f, "{e}"),
            SourceUnitElem::Error(e) => write!(f, "{e}"),
            SourceUnitElem::UserType(t) => write!(f, "{t}"),
        }
    }
}
