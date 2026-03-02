# Plan: Download Smart Contract Bug Benchmarks

This document outlines the plan to download existing smart contract bug benchmarks with **source code annotations** for use as test cases.

## Overview

**Goal**: Download Solidity contracts where bugs are annotated directly in the source code, suitable for testing the verazt static analyzer.

**Requirement**: Only datasets with:
- Solidity source files (.sol)
- Inline annotations or metadata marking vulnerability locations
- Clear vulnerability type classifications

---

## 1. Benchmarks with Source Code Annotations

### 1.1 Tier 1: Inline Annotations in Source Code

| Repository | Annotation Format | Vulnerability Types | Size | URL |
|------------|-------------------|---------------------|------|-----|
| **smartbugs-curated** | Inline markers (`<yes> <report> TYPE`) + JSON metadata with line numbers | DASP taxonomy (reentrancy, access control, arithmetic, etc.) | ~143 contracts | https://github.com/smartbugs/smartbugs-curated |
| **not-so-smart-contracts** | Code comments + README descriptions per vulnerability category | Common patterns (reentrancy, denial of service, etc.) | ~30 examples | https://github.com/crytic/not-so-smart-contracts |

**Details:**
- **smartbugs-curated**: Best for systematic testing. Contains `vulnerabilities.json` with exact line numbers and inline markers in comments.
- **not-so-smart-contracts**: Educational examples with explanations. Good for understanding vulnerability patterns.

### 1.2 Tier 2: Metadata-Based Annotations

| Repository | Annotation Format | Vulnerability Types | Size | URL |
|------------|-------------------|---------------------|------|-----|
| **SolidiFI-benchmark** | Injection logs mapping bugs to locations | 7 types: reentrancy, timestamp dependency, unhandled exceptions, unchecked send, TOD, integer overflow/underflow, tx.origin | 9,369 bugs | https://github.com/DependableSystemsLab/SolidiFI-benchmark |
| **ScaBench** | JSON with vulnerability metadata and code references | Real-world audit findings | 555 vulns in 31 projects | https://github.com/scabench-org/scabench |

**Details:**
- **SolidiFI-benchmark**: Systematically injected bugs with logs. Requires parsing injection logs to map bugs to source locations.
- **ScaBench**: Real-world vulnerabilities with JSON metadata. Good for realistic test cases.

### 1.3 Tier 3: Challenge-Based (Optional)

| Repository | Annotation Format | Use Case | URL |
|------------|-------------------|----------|-----|
| **damn-vulnerable-defi** | Challenge format with vulnerable contracts | DeFi-specific vulnerabilities | https://github.com/tinchoabbate/damn-vulnerable-defi |
| **ethernaut** | Wargame challenges | Educational smart contract vulnerabilities | https://github.com/OpenZeppelin/ethernaut |
| **DeFiHackLabs** | Exploit POCs with Foundry tests | Real-world DeFi exploits | https://github.com/SunWeb3Sec/DeFiHackLabs |

**Details:**
- Challenge-based benchmarks are less systematic but contain interesting real-world scenarios
- Good for integration testing and edge cases

---

## 2. Folder Structure

```
benchmarks/
├── README.md                        # Overview and annotation formats
├── raw/                             # Raw downloaded benchmarks
│   ├── smartbugs-curated/          # Tier 1: Best annotated
│   ├── not-so-smart-contracts/     # Tier 1: Educational examples
│   ├── solidifi-benchmark/         # Tier 2: Injection logs
│   ├── scabench/                   # Tier 2: Real-world audits
│   ├── damn-vulnerable-defi/       # Tier 3: Challenges (optional)
│   ├── ethernaut/                  # Tier 3: Wargame (optional)
│   └── defi-hack-labs/             # Tier 3: Exploit POCs (optional)
├── processed/                       # Processed test cases (generated)
│   ├── by-vulnerability/           # Organized by vulnerability type
│   │   ├── reentrancy/
│   │   ├── access-control/
│   │   ├── arithmetic/
│   │   └── ...
│   └── metadata.json               # Unified vulnerability metadata
└── scripts/
    ├── download.sh                 # Download all benchmarks
    ├── parse_smartbugs.py          # Extract smartbugs annotations
    ├── parse_solidifi.py           # Parse SolidiFI injection logs
    ├── parse_scabench.py           # Extract ScaBench metadata
    ├── generate_testcases.py       # Generate unified test cases
    └── verify.py                   # Verify downloads and parsing
```

---

## 3. Implementation Steps

### Step 1: Create Directory Structure
```bash
mkdir -p benchmarks/{raw,processed/by-vulnerability,scripts}
```

### Step 2: Download Benchmarks
Create `scripts/download.sh`:
```bash
#!/bin/bash
set -e

BENCHMARK_DIR="benchmarks/raw"
mkdir -p "$BENCHMARK_DIR"
cd "$BENCHMARK_DIR"

echo "Downloading Tier 1 benchmarks (inline annotations)..."
git clone --depth 1 https://github.com/smartbugs/smartbugs-curated.git
git clone --depth 1 https://github.com/crytic/not-so-smart-contracts.git

echo "Downloading Tier 2 benchmarks (metadata annotations)..."
git clone --depth 1 https://github.com/DependableSystemsLab/SolidiFI-benchmark.git solidifi-benchmark
git clone --depth 1 https://github.com/scabench-org/scabench.git

echo "Downloading Tier 3 benchmarks (optional challenges)..."
git clone --depth 1 https://github.com/tinchoabbate/damn-vulnerable-defi.git
git clone --depth 1 https://github.com/OpenZeppelin/ethernaut.git
git clone --depth 1 https://github.com/SunWeb3Sec/DeFiHackLabs.git defi-hack-labs

echo "Download complete!"
```

### Step 3: Parse SmartBugs Annotations
Create `scripts/parse_smartbugs.py`:
```python
"""Extract vulnerability annotations from smartbugs-curated."""
import json
import os
from pathlib import Path

def parse_smartbugs():
    """Parse vulnerabilities.json and extract annotated contracts."""
    base_path = Path("benchmarks/raw/smartbugs-curated")
    vuln_file = base_path / "vulnerabilities.json"

    with open(vuln_file) as f:
        vulns = json.load(f)

    test_cases = []
    for contract_path, metadata in vulns.items():
        sol_file = base_path / contract_path
        if sol_file.exists():
            test_cases.append({
                "file": str(sol_file),
                "vulnerabilities": metadata.get("vulnerabilities", []),
                "lines": metadata.get("lines", {}),
                "source": "smartbugs-curated"
            })

    return test_cases
```

### Step 4: Parse SolidiFI Injection Logs
Create `scripts/parse_solidifi.py`:
```python
"""Parse SolidiFI injection logs to map bugs to source locations."""
import json
from pathlib import Path

def parse_solidifi():
    """Parse injection logs from SolidiFI-benchmark."""
    base_path = Path("benchmarks/raw/solidifi-benchmark")

    # Parse injection logs (format TBD - needs investigation)
    # Maps buggy contracts to original + injection metadata
    test_cases = []

    for bug_type in ["Reentrancy", "Unhandled-Exceptions", etc.]:
        bug_dir = base_path / "buggy_contracts" / bug_type
        if bug_dir.exists():
            for sol_file in bug_dir.glob("*.sol"):
                # Extract injection metadata from filename or log
                test_cases.append({
                    "file": str(sol_file),
                    "vulnerability_type": bug_type,
                    "source": "solidifi-benchmark"
                })

    return test_cases
```

### Step 5: Generate Unified Test Cases
Create `scripts/generate_testcases.py`:
```python
"""Generate unified test case format for verazt."""
import json
from parse_smartbugs import parse_smartbugs
from parse_solidifi import parse_solidifi

def generate_testcases():
    """Combine all benchmarks into unified format."""
    all_cases = []

    # Parse all sources
    all_cases.extend(parse_smartbugs())
    all_cases.extend(parse_solidifi())

    # Organize by vulnerability type
    by_vuln = {}
    for case in all_cases:
        vuln_type = case.get("vulnerability_type", "unknown")
        if vuln_type not in by_vuln:
            by_vuln[vuln_type] = []
        by_vuln[vuln_type].append(case)

    # Save metadata
    with open("benchmarks/processed/metadata.json", "w") as f:
        json.dump({
            "total_cases": len(all_cases),
            "by_vulnerability": {k: len(v) for k, v in by_vuln.items()},
            "sources": list(set(c["source"] for c in all_cases))
        }, f, indent=2)

    return all_cases
```

### Step 6: Verification
Create `scripts/verify.py`:
```python
"""Verify downloads and parsing completeness."""
from pathlib import Path

def verify():
    """Check all benchmarks downloaded and parsed correctly."""
    checks = {
        "smartbugs-curated": Path("benchmarks/raw/smartbugs-curated/vulnerabilities.json"),
        "not-so-smart-contracts": Path("benchmarks/raw/not-so-smart-contracts/README.md"),
        "solidifi-benchmark": Path("benchmarks/raw/solidifi-benchmark/README.md"),
    }

    for name, path in checks.items():
        status = "✓" if path.exists() else "✗"
        print(f"{status} {name}: {path}")
```

---

## 4. Understanding Annotation Formats

### SmartBugs Curated Format
**File**: `vulnerabilities.json`
```json
{
  "contracts/reentrancy/simple_dao.sol": {
    "vulnerabilities": ["reentrancy"],
    "lines": {
      "reentrancy": [12, 13, 14]
    }
  }
}
```

**Inline markers** in source code:
```solidity
// <yes> <report> REENTRANCY
function withdraw(uint amount) public {
    if (balances[msg.sender] >= amount) {
        msg.sender.call.value(amount)();  // Vulnerable line
        balances[msg.sender] -= amount;
    }
}
```

### SolidiFI Format
- Buggy contracts in `buggy_contracts/<BUG_TYPE>/` folders
- Injection logs track where bugs were injected (needs investigation of log format)
- 7 bug types in separate directories

### Not-So-Smart-Contracts Format
- Organized by vulnerability pattern in folders
- Each folder has README.md explaining the vulnerability
- Example contracts with commented explanations

---

## 5. Dependencies

```bash
# Required
git  # For cloning repositories

# Optional (for parsing scripts)
pip install pathlib  # Usually included in Python 3.4+
```

---

## 6. Estimated Storage

| Benchmark | Estimated Size |
|-----------|----------------|
| smartbugs-curated | ~10 MB |
| not-so-smart-contracts | ~5 MB |
| solidifi-benchmark | ~100 MB |
| scabench | ~50 MB |
| Tier 3 (optional) | ~500 MB |
| **Total (Tier 1+2)** | **~165 MB** |
| **Total (with Tier 3)** | **~665 MB** |

---

## 7. Next Steps After Download

1. **Run parsing scripts** to extract vulnerability metadata
2. **Organize test cases** by vulnerability type in `processed/by-vulnerability/`
3. **Generate test harness** for verazt to run against benchmarks
4. **Create expected results** file mapping each test case to expected findings
5. **Integrate with verazt CI/CD** for regression testing

---

## 8. Notes

- Use `--depth 1` for git clones to minimize download size (no history needed)
- Start with Tier 1 benchmarks (smartbugs-curated, not-so-smart-contracts) as they have the clearest annotations
- Tier 2 benchmarks require more processing but provide larger test sets
- Tier 3 benchmarks are optional but useful for real-world scenarios
- Consider creating `.gitignore` entry for `benchmarks/raw/` if not tracking in version control

---

## 9. Sources

### Research Papers & Documentation
- [SmartBugs: A Dataset of Vulnerable Solidity Smart Contracts](https://smartbugs.github.io/)
- [An Empirical Analysis of Vulnerability Detection Tools for Solidity Smart Contracts Using Line Level Manually Annotated Vulnerabilities](https://arxiv.org/html/2505.15756v1)
- [How Effective are Smart Contract Analysis Tools](https://arxiv.org/pdf/2005.11613) (SolidiFI paper)

### Benchmark Repositories
- [SmartBugs Curated](https://github.com/smartbugs/smartbugs-curated) - Tier 1
- [Not So Smart Contracts](https://github.com/crytic/not-so-smart-contracts) - Tier 1
- [SolidiFI Benchmark](https://github.com/DependableSystemsLab/SolidiFI-benchmark) - Tier 2
- [ScaBench](https://github.com/scabench-org/scabench) - Tier 2
- [Damn Vulnerable DeFi](https://www.damnvulnerabledefi.xyz/) - Tier 3
- [Ethernaut](https://github.com/OpenZeppelin/ethernaut) - Tier 3
- [DeFiHackLabs](https://github.com/SunWeb3Sec/DeFiHackLabs) - Tier 3

### Additional Resources
- [Awesome Smart Contract Datasets](https://github.com/acorn421/awesome-smart-contract-datasets) - Curated list
- [Building Secure Contracts by Trail of Bits](https://github.com/crytic/building-secure-contracts) - Security guidelines
