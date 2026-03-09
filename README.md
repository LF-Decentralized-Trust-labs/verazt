<div align="center">

# Verazt

Smart contract security toolkit.

</div>

---

# Prerequisites

## For Solidity smart contracts

- We use `solc-select` to switch between Solidity compiler versions.

```bash
pip3 install solc-select
```

## For Vyper smart contracts

- We use `vyper-select` to switch between Vyper compiler versions.

```bash
pip3 install vyper-select
```

# Usage

## Analyzing Solidity contracts

```bash
verazt analyze <path/to/contract.sol>
```

## Analyzing Vyper contracts

```bash
verazt analyze <path/to/contract.vy>
```

# Notes

- For Solidity: only support Solidity version >= 0.4.12
