# Contributing to Verazt

Thanks for your interest in improving Verazt. Bug reports, false positive reports, new detectors, and fixes are all welcome.

## Reporting issues

When you report a false positive or a missed bug, include the smallest contract that reproduces it, the command you ran, and the output you got. A contract that compiles on its own is the fastest path to a fix.

## Building

Verazt builds with the nightly toolchain pinned in `rust-toolchain.toml`. Running the tests and the analyzer also needs `solc-select` (`make deps`) and, for Vyper, `vyper-select`; see the [README](README.md#installation).

```bash
cargo build
```

## Project layout

| Crate | Role |
| --- | --- |
| `frontend` | Parses Solidity and Vyper with their compilers and lowers them to SIR |
| `scirs` | The IRs: SIR (source level), CIR (core), and BIR (basic blocks in SSA form) |
| `analyzer` | Analysis passes, detectors, the pipeline, and report formatting |
| `bugs` | Bug types and dataset annotation parsing |
| `benchmark` | Measures detection accuracy against annotated datasets |
| `verazt` | The `verazt` binary |
| `common` | Shared utilities |

`verazt compile <file> --print-sir` prints the SIR of a contract, which helps when writing a detector (`--print-cir` and `--print-air` print the other IRs).

## Adding a detector

Most detectors are syntactic scan detectors that walk the SIR, like `crates/analyzer/src/detectors/sir/evm/function/tx_origin.rs`. To add one:

1. Add a variant to `DetectorId` in `crates/analyzer/src/detectors/base/id.rs`, with its kebab-case ID in `as_str`.
2. Create the detector under `crates/analyzer/src/detectors/sir/evm/`, in `module/`, `contract/`, or `function/` depending on the level it inspects. Define its `DetectorMeta` (description, recommendation, severity, confidence, SWC and CWE IDs) and implement `ScanDetector`: `level()` and the matching `check_module`, `check_contract`, or `check_function`. Use the `scirs::sir::utils::visit::Visit` trait to walk the IR.
3. Export it from that folder's `mod.rs`.
4. Register it in `register_all_detectors` in `crates/analyzer/src/detectors/base/registry.rs`. The registry tests list every `DetectorId` in `ALL_IDS` and `ordinal`: append the new ID to both (the exhaustive `ordinal` match does not compile until you do).
5. Add tests that run the detector on a contract with the bug and on one without it.

Detectors that need dataflow facts work on BIR instead: they implement `BugDetectionPass` directly and are registered with `registry.register`. See `crates/analyzer/src/detectors/bir/reentrancy.rs`.

## Testing

```bash
cargo test --no-fail-fast
```

`--no-fail-fast` runs every test target even when an earlier one fails.

The `frontend` crate generates one test per contract in the `libsolidity` and `smartbugs-curated` datasets:

```bash
cargo test -p frontend --test compile_libsolidity
cargo test -p frontend --test compile_smartbugs
```

To measure detection accuracy against the ground-truth annotations of a dataset:

```bash
cargo run -p benchmark -- --dataset solidity/smartbugs-curated
```

## Submitting changes

- Keep each pull request to one change, with tests for new behavior.
- Run `cargo build` and `cargo test` before opening it.
- Write commit messages as [Conventional Commits](https://www.conventionalcommits.org/), for example `fix(analyzer): skip unreachable blocks in the dataflow solver`.

By contributing, you agree that your contributions are licensed under the [Apache License 2.0](LICENSE).
