<div align="center">

# Verazt

Smart contract security toolkit.

</div>

---

# Prerequisites

## Install solc-select

We use `solc-select` to switch between Solidity compiler versions.

```bash
pip3 install solc-select
```

Then install the required versions or all of them:

```bash
# Install specific version
solc-select install 0.8.22

# Or install all versions
solc-select install all
```

## Install vyper-select

We use `vyper-select` to manage and switch between Vyper compiler versions,
analogous to `solc-select` for Solidity.

```bash
pip install vyper-select
```

Install the required Vyper version(s):

```bash
# Install a specific version
vyper-select install 0.3.10

# Or install all available versions
vyper-select install all
```

Verify the installation:

```bash
vyper-select versions
vyper --version
```

# Usage

## Analyzing Solidity contracts

```bash
smarthunt path/to/contract.sol
```

## Analyzing Vyper contracts

```bash
smarthunt path/to/contract.vy
```

The language is detected automatically from the file extension (`.vy` → Vyper,
`.sol` → Solidity). Use `--language` to override:

```bash
smarthunt --language vyper path/to/contract.vy
```

## Common options

```bash
# JSON output
smarthunt contract.vy --format json --output report.json

# Run detectors in parallel
smarthunt contract.vy --parallel

# Enable or disable specific detectors
smarthunt contract.vy --enable reentrancy,unchecked-call
smarthunt contract.vy --disable dead-code

# List all available detectors
smarthunt list-detectors
```

> **Note:** In the initial release, only DFA-based detectors run on Vyper
> contracts. AST-level pattern detectors are Solidity-only and are silently
> skipped for `.vy` input. See `examples/vyper/` for sample contracts.

# Notes

- Only support Solidity version >= 0.4.12
- Only support Vyper version >= 0.2.0
