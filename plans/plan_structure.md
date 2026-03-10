# Crates Folder Structure

## Workspace Layout

```
verazt/
├── Cargo.toml                          # Workspace root
├── crates/
│   ├── analysis/                       # Static analysis frameworks & passes
│   ├── bugs/                           # Bug datasets & definitions
│   ├── common/                         # Shared utilities (errors, strings)
│   ├── frontend/                       # Language frontends (Solidity, Vyper)
│   ├── mlir/                           # IR definitions (SIR, AIR dialects)
│   ├── scanner/                        # Bug detector pipeline & CLI
│   ├── verazt/                         # Main binary entry point
│   └── verifier/                       # Formal verification CLI
└── examples/
    └── solana/
        ├── token/
        ├── vault/
        └── vault_buggy/
```

---

## Crate Details

### `analysis`
Static analysis frameworks and analysis passes.

```
analysis/src/
├── lib.rs
├── context.rs
├── context_new.rs
├── frameworks/
│   ├── mod.rs
│   ├── cfa/                            # Control flow analysis
│   │   ├── mod.rs
│   │   ├── callgraph.rs
│   │   ├── domtree.rs
│   │   ├── loops.rs
│   │   └── reachability.rs
│   └── dfa/                            # Data flow analysis
│       ├── mod.rs
│       ├── annotated_cfg.rs
│       ├── lattice.rs
│       ├── solver.rs
│       ├── utils.rs
│       ├── var.rs
│       └── analyses/
│           ├── mod.rs
│           ├── def_use.rs
│           ├── liveness.rs
│           ├── reaching_defs.rs
│           ├── state_mutation.rs
│           └── taint.rs
├── passes/
│   ├── mod.rs
│   ├── base/
│   │   ├── mod.rs
│   │   ├── meta.rs
│   │   └── traits.rs
│   ├── air/                            # AIR-level passes
│   │   ├── mod.rs
│   │   ├── def_use.rs
│   │   ├── dominance.rs
│   │   ├── icfg.rs
│   │   ├── interval.rs
│   │   ├── taint.rs
│   │   └── taint_propagation.rs
│   ├── sir/                            # SIR-level passes
│   │   ├── mod.rs
│   │   ├── cfg_pass.rs
│   │   └── write_set.rs
│   └── vir/
│       └── mod.rs
└── pipeline/
    ├── mod.rs
    ├── dependency.rs
    ├── executor.rs
    ├── manager.rs
    └── scheduler.rs
```

**Dependencies:** `mlir`, `log`, `num-traits`, `petgraph`, `rayon`, `thiserror`

---

### `bugs`
Bug dataset definitions and SmartBugs integration.

```
bugs/src/
├── lib.rs
├── bug.rs
├── swc.rs
└── datasets/
    ├── mod.rs
    └── smartbugs.rs
```

**Dependencies:** `frontend`, `serde`, `regex`, `walkdir`

---

### `common`
Shared error types and string utilities.

```
common/src/
├── lib.rs
├── error.rs
└── string.rs
```

**Dependencies:** `color-eyre`

---

### `frontend`
Language frontends: AST definitions, parsers, and AST→SIR lowering for Solidity and Vyper.

```
frontend/src/
├── lib.rs
├── solidity/
│   ├── mod.rs
│   ├── ast/
│   │   ├── mod.rs
│   │   ├── block.rs
│   │   ├── builtins.rs
│   │   ├── defs.rs
│   │   ├── dirs.rs
│   │   ├── exprs.rs
│   │   ├── ident.rs
│   │   ├── lits.rs
│   │   ├── loc.rs
│   │   ├── name.rs
│   │   ├── source_unit.rs
│   │   ├── stmts.rs
│   │   ├── types.rs
│   │   ├── utils/
│   │   │   ├── mod.rs
│   │   │   ├── compare.rs
│   │   │   ├── export.rs
│   │   │   ├── fold.rs
│   │   │   ├── map.rs
│   │   │   ├── normalize.rs
│   │   │   ├── syntactic_comparer.rs
│   │   │   ├── version.rs
│   │   │   └── visit.rs
│   │   └── yul/
│   │       ├── mod.rs
│   │       ├── block.rs
│   │       ├── defs.rs
│   │       ├── exprs.rs
│   │       ├── ident.rs
│   │       ├── lits.rs
│   │       ├── sections.rs
│   │       ├── source_unit.rs
│   │       ├── stmts.rs
│   │       ├── types.rs
│   │       └── utils/
│   │           ├── mod.rs
│   │           ├── fold.rs
│   │           ├── map.rs
│   │           └── visit.rs
│   ├── lower/
│   │   ├── mod.rs
│   │   ├── lower.rs
│   │   └── normalize/                  # AST normalize passes (internal to lowering)
│   │       ├── mod.rs
│   │       ├── elim_func_modifier.rs
│   │       ├── elim_import_directives.rs
│   │       ├── elim_named_args.rs
│   │       ├── elim_using_directives.rs
│   │       ├── flatten_expr.rs
│   │       ├── flatten_name_index.rs
│   │       ├── merge_pragmas.rs
│   │       ├── rename_callees.rs
│   │       ├── rename_contracts.rs
│   │       ├── rename_defs.rs
│   │       ├── rename_vars.rs
│   │       ├── resolve_inheritance.rs
│   │       ├── substitution.rs
│   │       ├── unroll_unary_tuple.rs
│   │       └── utils.rs
│   └── parser/
│       ├── mod.rs
│       ├── json_ast_parser/
│       │   ├── mod.rs
│       │   └── ast_parser.rs
│       ├── type_parser/
│       │   ├── mod.rs
│       │   └── type_parser.rs
│       ├── version_parser/
│       │   ├── mod.rs
│       │   └── version_parser.rs
│       └── yul_parser/
│           ├── mod.rs
│           ├── keywords.rs
│           └── parser.rs
└── vyper/
    ├── mod.rs
    ├── ast/
    │   ├── mod.rs
    │   ├── defs.rs
    │   ├── exprs.rs
    │   ├── loc.rs
    │   ├── source_unit.rs
    │   ├── stmts.rs
    │   └── types.rs
    ├── lower/
    │   ├── mod.rs
    │   ├── lower.rs
    │   └── normalize/                  # AST normalize passes (internal to lowering)
    │       ├── mod.rs
    │       ├── flatten_expr.rs
    │       ├── rename_defs.rs
    │       └── rename_vars.rs
    └── parser/
        ├── mod.rs
        └── json_ast_parser/
            ├── mod.rs
            └── ast_parser.rs
```

**Dependencies:** `mlir`, `common`, `clap`, `serde`, `serde_json`, `petgraph`, `pest`, `pest_derive`, `rayon` (optional), and more

---

### `mlir`
IR dialect definitions: SIR (Smart contract IR), CIR (Canonical IR), and AIR (Analysis IR).

```
mlir/src/
├── lib.rs
├── sir/                                # Smart contract IR
│   ├── mod.rs
│   ├── attrs.rs
│   ├── cfg.rs
│   ├── defs.rs
│   ├── exprs.rs
│   ├── lits.rs
│   ├── loc.rs
│   ├── module.rs
│   ├── spec.rs
│   ├── stmts.rs
│   ├── types.rs
│   ├── dialect/
│   │   ├── mod.rs
│   │   ├── anchor.rs
│   │   ├── evm.rs
│   │   ├── move_lang.rs
│   │   └── spec_dialect.rs
│   └── utils/
│       ├── mod.rs
│       ├── export.rs
│       ├── fold.rs
│       ├── map.rs
│       ├── printer.rs
│       ├── query.rs
│       └── visit.rs
├── cir/                                # Canonical IR (normalized, structured)
│   ├── mod.rs
│   ├── defs.rs
│   ├── exprs.rs
│   ├── module.rs
│   ├── stmts.rs
│   └── lower/                          # SIR → CIR lowering
│       └── mod.rs
└── air/                                # Analysis IR
    ├── mod.rs
    ├── alias.rs
    ├── call_graph.rs
    ├── cfg.rs
    ├── interfaces.rs
    ├── module.rs
    ├── ops.rs
    ├── pdg.rs
    ├── summary.rs
    ├── taint.rs
    └── lower/                          # CIR → AIR lowering
        ├── mod.rs
        ├── cfg.rs
        ├── dialect_lower.rs
        ├── icfg.rs
        └── ssa.rs
```

**Dependencies:** `common`, `indexmap`, `serde`, `num-bigint`, `num-traits`, `rust_decimal`, `thiserror`, `bat`, `color-eyre`

---

### `scanner`
Bug detection pipeline: detectors, pattern matching engine, and output formatting.

```
scanner/src/
├── lib.rs
├── artifacts.rs
├── cli.rs
├── config.rs
├── detector/
│   ├── mod.rs
│   ├── id.rs
│   └── traits.rs
├── detectors/
│   ├── mod.rs
│   ├── access_control.rs
│   ├── acquires_mismatch.rs
│   ├── arithmetic.rs
│   ├── cei_violation.rs
│   ├── centralization_risk.rs
│   ├── constant_state_var.rs
│   ├── dead_code.rs
│   ├── delegatecall.rs
│   ├── deprecated.rs
│   ├── floating_pragma.rs
│   ├── low_level_call.rs
│   ├── missing_access_control.rs
│   ├── missing_modifies.rs
│   ├── missing_pda_constraint.rs
│   ├── reentrancy.rs
│   ├── shadowing.rs
│   ├── sir_missing_access_control.rs
│   ├── timestamp_dependence.rs
│   ├── tx_origin.rs
│   ├── tx_origin_auth.rs
│   ├── unchecked_arithmetic.rs
│   ├── unchecked_call.rs
│   ├── uninitialized.rs
│   └── visibility.rs
├── engines/
│   ├── mod.rs
│   ├── datalog/
│   │   └── mod.rs
│   └── pattern/
│       ├── mod.rs
│       ├── builder.rs
│       ├── composite.rs
│       ├── core.rs
│       ├── matcher.rs
│       └── primitives.rs
├── output/
│   ├── mod.rs
│   ├── formatter.rs
│   ├── json.rs
│   ├── markdown.rs
│   └── sarif.rs
└── pipeline/
    ├── mod.rs
    ├── engine.rs
    └── registry.rs
```

**Dependencies:** `analysis`, `bugs`, `frontend`, `mlir`, `common`, `clap`, `serde`, `serde_json`, `rayon`, `toml`, `chrono`, `thiserror`, `log`

---

### `verazt`
Main binary crate. Entry point for the CLI tool.

```
verazt/src/
├── main.rs
└── compile.rs
```

**Dependencies:** `scanner`, `verifier`, `frontend`, `mlir`, `common`, `clap`

---

### `verifier`
Formal verification CLI (early stage).

```
verifier/src/
├── lib.rs
└── cli.rs
```

**Dependencies:** `analysis`, `clap`, `clap-verbosity-flag`

---

## Dependency Graph

```
verazt (bin)
├── scanner
│   ├── analysis
│   │   └── mlir
│   │       └── common
│   ├── bugs
│   │   └── frontend
│   │       ├── mlir
│   │       └── common
│   ├── frontend
│   └── mlir
├── verifier
│   └── analysis
├── frontend
├── mlir
└── common
```
