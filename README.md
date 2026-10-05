# Verazt

Verazt is a static analyzer that finds security vulnerabilities and code quality issues in Solidity and Vyper smart contracts.

It lowers contracts into a family of intermediate representations and runs two kinds of detectors on them: fast syntactic detectors on the source-level IR, and dataflow detectors (such as cross-function reentrancy) on a control-flow-graph IR in SSA form. Findings can be printed as text, or exported as JSON, Markdown, or SARIF for code scanning tools.

## Installation

Verazt is written in Rust and builds with the nightly toolchain pinned in `rust-toolchain.toml`, which `rustup` selects automatically.

```bash
git clone https://github.com/taquangtrung/verazt.git
cd verazt
cargo install --path crates/verazt
```

Verazt calls the official compilers to parse contracts, and picks the compiler version each contract's pragma asks for through a version manager:

- Solidity needs [`solc-select`](https://github.com/crytic/solc-select). Run `make deps` to install it with `uv` or `pipx`.
- Vyper needs `vyper-select`, which is not published on PyPI and must be installed separately.

When no installed compiler satisfies a contract's pragma, Verazt offers to install one. Pass `--install-compiler` to install it without asking.

## Usage

Analyze one or more contracts:

```bash
verazt contracts/Token.sol
verazt contracts/Vault.vy
```

`verazt analyze <files>` is the same command spelled out. All files in one run must be in the same language, which is detected from the extension of the first file or set with `--language`.

Useful options:

```bash
# Report only medium severity and above
verazt contracts/Token.sol --min-severity medium

# Write a SARIF report for GitHub code scanning
verazt contracts/Token.sol --format sarif --output verazt.sarif

# Run only the selected detectors
verazt contracts/Token.sol --enable reentrancy,tx-origin
```

Run `verazt --help` for the full list of options.

### Example output

```text
🐛 Issue 2: Constant State Variable (Code Quality) [constant-state-var]

---> bug_sample.sol:6:5-21
     5 | contract BugSample {
>    6 |     uint256 uzero = 0;
    .. |     ^^^^^^^^^^^^^^^^^
     7 |     uint256 umax = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;

Description: State variable 'uzero' in 'BugSample' is never modified after initialization. Consider declaring it as 'constant' or 'immutable' to save gas.

Severity: Low
```

The process exits with status 1 when it finds a high or critical severity issue, or when part of the analysis fails, so it can gate a CI pipeline.

### Configuration

Generate a configuration file with the default settings, then pass it with `--config`:

```bash
verazt init-config verazt.toml
verazt contracts/Token.sol --config verazt.toml
```

The file selects the detectors, the output format, the minimum severity, and the number of worker threads. Command-line options override it.

## Detectors

| ID | Detects | Severity |
| --- | --- | --- |
| `reentrancy` | Reentrancy | Critical |
| `reentrancy-flow` | Reentrancy, found on the control-flow graph | Critical |
| `reentrancy-cross-function` | Cross-function reentrancy | High |
| `arithmetic-overflow` | Integer overflow and underflow | High |
| `bad-randomness` | Randomness from predictable sources | High |
| `delegatecall` | Dangerous `delegatecall` | High |
| `denial-of-service` | Denial of service | High |
| `missing-access-control` | Missing access control | High |
| `tx-origin` | Authentication with `tx.origin` | High |
| `uninitialized-storage` | Uninitialized storage | High |
| `centralization-risk` | Centralization risk | Medium |
| `front-running` | Front running | Medium |
| `unchecked-call` | Unchecked call return values | Medium |
| `visibility` | Visibility issues | Medium |
| `constant-state-var` | State variables that could be `constant` | Low |
| `dead-code` | Dead code | Low |
| `deprecated` | Deprecated features | Low |
| `floating-pragma` | Floating pragma | Low |
| `low-level-call` | Low-level calls, checked or not | Low |
| `shadowing` | Variable shadowing | Low |
| `short-address` | Short address attack | Low |
| `timestamp-dependence` | Timestamp dependence | Low |

`verazt list-detectors` prints this list with each detector's confidence, and `verazt show-detector <id>` prints its description, recommendation, and SWC and CWE references.

## Limitations

- Solidity support starts at version 0.4.12.
- Verazt is under active development: expect false positives, and check findings before acting on them.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to build, test, and add a detector.

## License

Verazt is licensed under the [Apache License 2.0](LICENSE).
