pub fn run<I, T>(_args_iter: I)
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    println!("verify run not implemented yet");
}
