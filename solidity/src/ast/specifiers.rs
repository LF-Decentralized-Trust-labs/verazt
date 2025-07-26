use crate::ast::*;
use color_eyre::eyre::{bail, Result};
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing all specifiers
//-------------------------------------------------------------------------

/// Function and variable overriding.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Overriding {
    None,
    All,             // Override all parent contracts
    Some(Vec<Name>), // List of contract names that is overridden
}

/// Function visibility.
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum FuncVis {
    Internal,
    External,
    Private,
    Public,
    None, // No visibility specification
}

/// Variable visibility.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum VarVis {
    Internal,
    Private,
    Public,
    None, // No visibility specification
}

/// Function mutability.
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum FuncMut {
    Payable,
    NonPayable,
    Pure,
    View,
    Constant,
    None, // No mutability specification.
}

/// Variable mutability.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum VarMut {
    Constant,
    Immutable,
    Mutable,
    None, // No mutability specification.
}

//-------------------------------------------------------------------------
// Implementation for Overriding
//-------------------------------------------------------------------------

impl Overriding {
    pub fn is_all(&self) -> bool {
        matches!(self, Overriding::All)
    }

    pub fn is_some(&self) -> bool {
        matches!(self, Overriding::Some(_))
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Overriding::None)
    }
}

impl Display for Overriding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Overriding::None => write!(f, "").ok(),
            Overriding::All => write!(f, "override").ok(),
            Overriding::Some(names) => {
                let names: Vec<String> = names.iter().map(|n| n.to_string()).collect();
                write!(f, "override({})", names.join(", ")).ok()
            }
        };
        Ok(())
    }
}

//-------------------------------------------------------------------------
// Implementation for Function Visibility
//-------------------------------------------------------------------------

impl FuncVis {
    pub fn new(visibility: &str) -> Self {
        match visibility {
            "internal" => Self::Internal,
            "external" => Self::External,
            "public" => Self::Public,
            "private" => Self::Private,
            _ => panic!("Unknown visibility: {visibility}"),
        }
    }
}

impl Display for FuncVis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // By default, do not print `internal`.
            FuncVis::Internal => write!(f, "internal"),
            FuncVis::External => write!(f, "external"),
            FuncVis::Private => write!(f, "private"),
            FuncVis::Public => write!(f, "public"),
            FuncVis::None => write!(f, ""),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Variable Visibility
//-------------------------------------------------------------------------

impl VarVis {
    pub fn new(visibility: &str) -> Self {
        match visibility {
            "internal" => Self::Internal,
            "public" => Self::Public,
            "private" => Self::Private,
            _ => panic!("Unknown visibility: {visibility}"),
        }
    }
}

impl Display for VarVis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // By default, do not print `internal`.
            VarVis::Internal => write!(f, ""),
            VarVis::Private => write!(f, "private"),
            VarVis::Public => write!(f, "public"),
            VarVis::None => write!(f, ""),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Function mutability
//-------------------------------------------------------------------------

impl FuncMut {
    pub fn new(mutability: &str) -> Result<Self> {
        match mutability {
            "payable" => Ok(FuncMut::Payable),
            "nonpayable" => Ok(FuncMut::NonPayable),
            "pure" => Ok(FuncMut::Pure),
            "view" => Ok(FuncMut::View),
            _ => bail!("Unknown state mutability: {}", mutability),
        }
    }

    pub fn is_view(&self) -> bool {
        matches!(self, Self::View)
    }

    pub fn is_pure(&self) -> bool {
        matches!(self, Self::Pure)
    }

    pub fn is_payable(&self) -> bool {
        matches!(self, Self::Payable)
    }

    pub fn is_non_payable(&self) -> bool {
        matches!(self, Self::NonPayable)
    }
}

impl Display for FuncMut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FuncMut::Constant => write!(f, "constant"),
            FuncMut::NonPayable => write!(f, ""),
            FuncMut::Payable => write!(f, "payable"),
            FuncMut::Pure => write!(f, "pure"),
            FuncMut::View => write!(f, "view"),
            FuncMut::None => write!(f, ""),
        }
    }
}

//-------------------------------------------------------------------------
// Implementation for Variable mutability
//-------------------------------------------------------------------------

impl VarMut {
    pub fn new(mutability: &str) -> Result<Self> {
        match mutability {
            "mutable" => Ok(VarMut::Mutable),
            "immutable" => Ok(VarMut::Immutable),
            "constant" => Ok(VarMut::Constant),
            _ => bail!("Unknown mutability: {}", mutability),
        }
    }

    pub fn is_mutable(&self) -> bool {
        matches!(self, Self::Mutable)
    }

    pub fn is_immutable(&self) -> bool {
        matches!(self, Self::Immutable)
    }
}

impl Display for VarMut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VarMut::Constant => write!(f, "constant"),
            VarMut::Immutable => write!(f, "immutable"),
            VarMut::Mutable => write!(f, "mutable"),
            VarMut::None => write!(f, ""),
        }
    }
}
