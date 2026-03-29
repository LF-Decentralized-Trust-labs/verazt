use common::error::{Result, create_error};
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Source code location
//-------------------------------------------------------------------------

/// Source code location — unified type alias.
pub type Loc = common::loc::Loc;

//-------------------------------------------------------------------------
// Data location
//-------------------------------------------------------------------------

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum DataLoc {
    Memory,   // In memory.
    Storage,  // In storage.
    Calldata, // In function call data
    None,     // No data location specified.
}

impl DataLoc {
    pub fn new(data_loc: &str) -> Result<Self> {
        match data_loc {
            "memory" => Ok(DataLoc::Memory),
            "storage" => Ok(DataLoc::Storage),
            "calldata" => Ok(DataLoc::Calldata),
            "default" => Ok(DataLoc::None),
            _ => Err(create_error(format!("Unknown data location: {data_loc}"))),
        }
    }

    /// Check if the data location is `Calldata`.
    #[must_use]
    pub fn is_calldata(&self) -> bool {
        matches!(self, Self::Calldata)
    }

    #[must_use]
    pub fn is_storage(&self) -> bool {
        matches!(self, Self::Storage)
    }

    #[must_use]
    pub fn is_memory(&self) -> bool {
        matches!(self, Self::Memory)
    }
}

impl Display for DataLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DataLoc::*;
        match self {
            Memory => write!(f, "memory"),
            Storage => write!(f, "storage"),
            Calldata => write!(f, "calldata"),
            None => Ok(()),
        }
    }
}
