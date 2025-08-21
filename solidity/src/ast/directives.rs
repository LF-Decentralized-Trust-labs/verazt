use crate::ast::*;
use meta::{Loc, Name, NamePath};
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing directives
//-------------------------------------------------------------------------

/// Import directive.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ImportDir {
    pub id: Option<isize>,
    pub kind: ImportKind,
}

/// Kind of import directive.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ImportKind {
    ImportSourceUnit(ImportSourceUnit),
    ImportSymbols(ImportSymbols),
}

/// Directive to import source unit.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ImportSourceUnit {
    pub path: String,          // Import path
    pub abs_path: String,      // Absolute path to the source unit
    pub alias: Option<String>, // Alias given to the source unit
    pub loc: Option<Loc>,
}

/// Directive to import symbols.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ImportSymbols {
    pub source_unit_path: String,
    pub source_unit_abs_path: String,
    pub imported_symbols: Vec<ImportSymbol>,
    pub loc: Option<Loc>,
}

/// Directive to import one symbol.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ImportSymbol {
    pub symbol_name: String,
    pub symbol_alias: Option<String>,
    pub loc: Option<Loc>,
}

/// Pragma directive.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PragmaDir {
    pub id: Option<isize>,
    pub kind: PragmaKind,
    pub loc: Option<Loc>,
}

/// Kind of pragma directive.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum PragmaKind {
    Version(String),
    AbiCoder(String),
    Experimental(String),
}

/// Using directive.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UsingDir {
    pub id: Option<isize>,
    pub kind: UsingKind,
    pub target_type: Option<Type>,
    pub is_global: bool,
    pub loc: Option<Loc>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum UsingKind {
    UsingLib(UsingLib),
    UsingFunc(Vec<UsingFunc>),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UsingLib {
    pub lib_name: Name,
    pub lib_path: NamePath, // Where the library is defined.
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UsingFunc {
    pub func_name: Name,
    pub func_path: NamePath, // Where the function is defined.
    pub alias_operator: Option<String>,
}

//-------------------------------------------------------------------------
// Implementations for import directive.
//-------------------------------------------------------------------------

impl ImportDir {
    pub fn new(id: Option<isize>, kind: ImportKind) -> Self {
        ImportDir { id, kind }
    }

    pub fn new_import_path(id: Option<isize>, import: ImportSourceUnit) -> Self {
        let kind = ImportKind::ImportSourceUnit(import);
        ImportDir::new(id, kind)
    }

    pub fn new_import_symbols(id: Option<isize>, import: ImportSymbols) -> Self {
        let kind = ImportKind::ImportSymbols(import);
        ImportDir::new(id, kind)
    }

    pub fn get_import_path(&self) -> String {
        match &self.kind {
            ImportKind::ImportSourceUnit(import_sunit) => import_sunit.abs_path.to_string(),
            ImportKind::ImportSymbols(import_symbols) => {
                import_symbols.source_unit_abs_path.to_string()
            }
        }
    }
}

impl Display for ImportDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ImportKind::ImportSourceUnit(import) => write!(f, "{import}"),
            ImportKind::ImportSymbols(import) => write!(f, "{import}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for import kind.
//-------------------------------------------------------------------------

impl From<ImportSourceUnit> for ImportKind {
    fn from(import: ImportSourceUnit) -> Self {
        ImportKind::ImportSourceUnit(import)
    }
}

impl From<ImportSymbols> for ImportKind {
    fn from(import: ImportSymbols) -> Self {
        ImportKind::ImportSymbols(import)
    }
}

//-------------------------------------------------------------------------
// Implementations for import source unit directive.
//-------------------------------------------------------------------------

impl ImportSourceUnit {
    pub fn new(abs_path: String, path: String, alias: Option<String>, loc: Option<Loc>) -> Self {
        ImportSourceUnit { abs_path, path, alias, loc }
    }
}

impl Display for ImportSourceUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "import \"{}\"", self.path).ok();

        match &self.alias {
            None => write!(f, ";"),
            Some(name) => write!(f, " as {name};"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for directive to import symbols.
//-------------------------------------------------------------------------

impl ImportSymbols {
    pub fn new(
        abs_path: String,
        file_path: String,
        symbols: Vec<ImportSymbol>,
        loc: Option<Loc>,
    ) -> Self {
        ImportSymbols {
            source_unit_abs_path: abs_path,
            source_unit_path: file_path,
            imported_symbols: symbols,
            loc,
        }
    }
}

impl Display for ImportSymbols {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbols = self
            .imported_symbols
            .iter()
            .map(|s| format!("{s}"))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "import {{ {} }} from \"{}\";", symbols, self.source_unit_path,)
    }
}

//-------------------------------------------------------------------------
// Implementations for directive to import one symbol.
//-------------------------------------------------------------------------

impl ImportSymbol {
    pub fn new(name: String, alias: Option<String>, loc: Option<Loc>) -> Self {
        ImportSymbol { symbol_name: name, symbol_alias: alias, loc }
    }
}

impl Display for ImportSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.symbol_alias {
            None => write!(f, "{}", self.symbol_name),
            Some(alias) => write!(f, "{} as {}", self.symbol_name, alias),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Pragma directive
//-------------------------------------------------------------------------

impl PragmaDir {
    pub fn new(id: Option<isize>, kind: PragmaKind, loc: Option<Loc>) -> Self {
        PragmaDir { id, kind, loc }
    }
}

impl Display for PragmaDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pragma {};", self.kind)
    }
}

//-------------------------------------------------------------------------
// Implementation for Pragma kind
//-------------------------------------------------------------------------

impl PragmaKind {
    pub fn new_version(version: String) -> Self {
        PragmaKind::Version(version)
    }

    pub fn new_abi_coder(string: String) -> Self {
        PragmaKind::AbiCoder(string)
    }

    pub fn new_experimental(string: String) -> Self {
        PragmaKind::Experimental(string)
    }
}

impl Display for PragmaKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PragmaKind::Version(v) => write!(f, "solidity {v}"),
            PragmaKind::AbiCoder(v) => write!(f, "abicoder {v}"),
            PragmaKind::Experimental(v) => write!(f, "experimental {v}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Using directive
//-------------------------------------------------------------------------

impl UsingDir {
    pub fn new(
        id: Option<isize>,
        kind: UsingKind,
        typ: Option<Type>,
        global: bool,
        loc: Option<Loc>,
    ) -> Self {
        UsingDir { id, kind, target_type: typ, is_global: global, loc }
    }
}

impl Display for UsingDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "using {} for ", self.kind).ok();
        match &self.target_type {
            Some(typ) => write!(f, "{typ}").ok(),
            None => write!(f, "*").ok(),
        };
        match self.is_global {
            true => write!(f, " global;"),
            false => write!(f, ";"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Using directive Kind
//-------------------------------------------------------------------------

impl Display for UsingKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UsingKind::UsingLib(ulib) => write!(f, "{ulib}"),
            UsingKind::UsingFunc(ufuncs) => {
                let ufuncs = ufuncs
                    .iter()
                    .map(|u| u.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{{{ufuncs}}}")
            }
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for directive to use library.
//-------------------------------------------------------------------------

impl UsingLib {
    pub fn new(name_path: &str) -> Self {
        let names: Vec<Name> = name_path.split('.').map(Name::from).collect();
        match &names[..] {
            [] => panic!("Construct UsingLibDir: empty name path"),
            [ns @ .., n] => UsingLib { lib_name: n.clone(), lib_path: NamePath::new(ns.to_vec()) },
        }
    }
}

impl Display for UsingLib {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let scope = format!("{}", self.lib_path);
        if scope.is_empty() {
            write!(f, "{}", self.lib_name)
        } else {
            write!(f, "{}.{}", scope, self.lib_name)
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for directive to use function.
//-------------------------------------------------------------------------

impl UsingFunc {
    pub fn new(name_path: &str, op: Option<&str>) -> Self {
        let names: Vec<Name> = name_path.split('.').map(Name::from).collect();
        match &names[..] {
            [] => panic!("Construct UsingLibDir: empty name path"),
            [ns @ .., n] => UsingFunc {
                func_name: n.clone(),
                func_path: NamePath::new(ns.to_vec()),
                alias_operator: op.map(String::from),
            },
        }
    }
}

impl Display for UsingFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let scope = format!("{}", self.func_path);
        match scope.is_empty() {
            true => write!(f, "{}", self.func_name).ok(),
            false => write!(f, "{}.{}", scope, self.func_name).ok(),
        };
        if let Some(op) = &self.alias_operator {
            write!(f, " as {op}").ok();
        }
        Ok(())
    }
}
