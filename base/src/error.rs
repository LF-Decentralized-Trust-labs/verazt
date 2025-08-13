//! Wrapper library to provide `color_eyre::eyre` utilities.

use color_eyre::eyre::{self, eyre};
use core::fmt::{Debug, Display};
use std::panic::Location;

//-------------------------------------------------------------------------
// Wrapper type
//-------------------------------------------------------------------------

/// Wrapper to eyre::Result type
pub type Result<T, E = eyre::Report> = color_eyre::eyre::Result<T, E>;

pub type Report = color_eyre::eyre::Report;

//-------------------------------------------------------------------------
// Wrapper functions
//-------------------------------------------------------------------------

/// Helper function to create an error and capture the source code location raising it.
///
/// NOTE: `Location::caller()` needs to be called from a function, not directly
/// from a macro, to be able to capture the source code location of the caller.
#[track_caller]
pub fn create_error(error_msg: impl std::fmt::Display) -> eyre::Report {
    let loc = Location::caller();
    let msg = if cfg!(debug_assertions) {
        // If build in Debug mode, track source code location raising this error.
        format!(
            "{}\nRaised at file: {}:{}.",
            error_msg,
            loc.file(),
            loc.line()
        )
    } else {
        format!("{error_msg}")
    };
    eyre!(msg)
}

//-------------------------------------------------------------------------
// New macros
//-------------------------------------------------------------------------

/// Create an error message which also captures caller's source code location.
#[macro_export]
macro_rules! error {
    ($msg:literal $(,)?) => {
        return base::error::create_error(format!($msg));
    };
    ($err:expr $(,)?) => {
        return base::error::create_error($err);
    };
    ($fmt:expr, $($arg:tt)*) => {
        return base::error::create_error(color_eyre::eyre::eyre!($fmt, $($arg)*));
    };
}

/// Report an error and exit the current function immediately, similar to the
/// `return` statement.
#[macro_export]
macro_rules! fail {
    ($msg:literal $(,)?) => {
        return Err(base::error::create_error(format!($msg)));
    };
    ($err:expr $(,)?) => {
        return Err(base::error::create_error($err));
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err(base::error::create_error(color_eyre::eyre::eyre!($fmt, $($arg)*)));
    };
}

pub fn report_error<T>(error_msg: impl std::fmt::Display) -> eyre::Result<T, eyre::Report> {
    let report = create_error(error_msg);
    Err(report)
}


//-------------------------------------------------------------------------
// New utilities to handle option types of errors
//-------------------------------------------------------------------------

pub trait OptionExt<T> {
    fn ok_or_error<M>(self, message: M) -> Result<T>
    where
        M: Debug + Display + Send + Sync + 'static;
}

impl<T> OptionExt<T> for Option<T> {
    #[track_caller]
    fn ok_or_error<M>(self, message: M) -> Result<T>
    where
        M: Debug + Display + Send + Sync + 'static,
    {
        match self {
            Some(ok) => Ok(ok),
            None => Err(eyre::Report::msg(message)),
        }
    }
}

//-------------------------------------------------------------------------
// Public utilities
//-------------------------------------------------------------------------

/// Configure new error reporting mechanism
pub fn init() {
    let _ = color_eyre::install();
}
