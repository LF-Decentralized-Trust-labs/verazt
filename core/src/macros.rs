#[macro_export]
macro_rules! eyre_loc {
    ($msg:literal $(,)?) => {
        return core::stdext::eyre_ext::eyre_with_location(format!($msg));
    };
    ($err:expr $(,)?) => {
        return core::stdext::eyre_ext::eyre_with_location($err);
    };
    ($fmt:expr, $($arg:tt)*) => {
        return core::stdext::eyre_ext::eyre_with_location(color_eyre::eyre::eyre!($fmt, $($arg)*));
    };
}
