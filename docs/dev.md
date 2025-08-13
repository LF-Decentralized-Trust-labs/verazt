# Dev Notes

- Print logging information using the `env_logger` crate, controlled by the environment variable `RUST_LOG`:

  ``` sh
  RUST_LOG=info <command>
  RUST_LOG=debug <command>
  ```

  See usage at: https://docs.rs/env_logger/latest/env_logger/
