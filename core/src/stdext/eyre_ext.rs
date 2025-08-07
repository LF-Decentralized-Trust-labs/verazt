use color_eyre::eyre::{self, bail, eyre, Error, Result};
use color_eyre::Report;
use std::panic::Location;

#[track_caller]
pub fn eyre_with_location(msg: impl std::fmt::Display) -> eyre::Report {
    let loc = Location::caller();
    eyre!(format!("{} at {}:{}", msg, loc.file(), loc.line()))
}

