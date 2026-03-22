pub fn configure_unit_test_env() {
    let _ = env_logger::builder().is_test(true).try_init();
}
