use color_eyre::eyre::{self, eyre};
use std::panic::Location;

/// Create an error and capture the source code location raising it.
///
/// NOTE: `Location::caller()` needs to be called from a function, not directly
/// from a macro, to be able to capture the source code location of the caller.
#[track_caller]
pub fn create_error(error_msg: impl std::fmt::Display) -> eyre::Report {
    let loc = Location::caller();
    let msg = if cfg!(debug_assertions) {
        // If build in Debug mode, track source code location raising this error.
        format!(
            "{}, raised at file: {}:{}.",
            error_msg,
            loc.file(),
            loc.line()
        )
    } else {
        format!("{error_msg}")
    };
    eyre!(msg)
}

/// Create an error message which also capture source code location of the
/// caller.
#[macro_export]
macro_rules! error {
    ($msg:literal $(,)?) => {
        return core::error::create_error(format!($msg));
    };
    ($err:expr $(,)?) => {
        return core::error::create_error($err);
    };
    ($fmt:expr, $($arg:tt)*) => {
        return core::error::create_error(color_eyre::eyre::eyre!($fmt, $($arg)*));
    };
}

/// Report an error and exit the current function immediately, similar to the
/// `return` statement.
#[macro_export]
macro_rules! fail {
    ($msg:literal $(,)?) => {
        return Err(core::error::create_error(format!($msg)));
    };
    ($err:expr $(,)?) => {
        return Err(core::error::create_error($err));
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err(core::error::create_error(color_eyre::eyre::eyre!($fmt, $($arg)*)));
    };
}
