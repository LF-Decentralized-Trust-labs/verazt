use color_eyre::eyre::{self, bail, eyre, Error, Result};
use color_eyre::Report;
use std::panic::Location;

/// Wrapper of `eyre!` to capture caller location.
#[track_caller]
pub fn eyre_with_location(msg: impl std::fmt::Display) -> eyre::Report {
    let loc = Location::caller();
    eyre!(format!("{} at {}:{}", msg, loc.file(), loc.line()))
}

/// New macro to create an error report which also capture source code location
/// of the caller.
///
/// NOTE: the caller location cannot be tracked directly from the macro, but
/// need to be tracked by the wrapper function `eyre_with_location`
#[macro_export]
macro_rules! eyre_loc {
    ($msg:literal $(,)?) => {
        return core::error::eyre_with_location(format!($msg));
    };
    ($err:expr $(,)?) => {
        return core::error::eyre_with_location($err);
    };
    ($fmt:expr, $($arg:tt)*) => {
        return core::error::eyre_with_location(color_eyre::eyre::eyre!($fmt, $($arg)*));
    };
}

/// New macro to report an report which also captures source code location of the
/// caller.
///
/// NOTE: the caller location cannot be tracked directly from the macro, but
/// need to be tracked by the wrapper function `eyre_with_location`
#[macro_export]
macro_rules! bail_loc {
    ($msg:literal $(,)?) => {
        return Err(core::error::eyre_with_location(format!($msg)));
    };
    ($err:expr $(,)?) => {
        return Err(core::error::eyre_with_location($err));
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err(core::error::eyre_with_location(color_eyre::eyre::eyre!($fmt, $($arg)*)));
    };
}
