//! Integration tests for the Vyper parser and IR lowering pipeline.
//!
//! These tests exercise the full pipeline:
//! JSON AST string → internal AST → normalization → SIR Module

use vyper::ast::normalize;
use vyper::irgen;
use vyper::parser;

/// Helper: run the full pipeline from JSON AST string.
fn compile_json(json: &str, path: &str) -> mlir::sir::Module {
    let su = parser::parse_from_json(json, path).expect("parse_from_json failed");
    let norm = normalize::run_passes(&su);
    irgen::lower_source_unit(&norm).expect("lower_source_unit failed")
}

// ─── JSON AST fragments for testing ───────────────────────────

/// A minimal ERC-20-like contract JSON AST
const MINIMAL_TOKEN_JSON: &str = r#"{
    "ast_type": "Module",
    "body": [
        {
            "ast_type": "VariableDecl",
            "target": { "ast_type": "Name", "id": "totalSupply", "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 11 },
            "annotation": { "ast_type": "Name", "id": "uint256", "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 11 },
            "value": null,
            "is_constant": false,
            "is_immutable": false,
            "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 30
        },
        {
            "ast_type": "VariableDecl",
            "target": { "ast_type": "Name", "id": "balanceOf", "lineno": 2, "col_offset": 0, "end_lineno": 2, "end_col_offset": 9 },
            "annotation": {
                "ast_type": "Subscript",
                "value": { "ast_type": "Name", "id": "HashMap", "lineno": 2, "col_offset": 0, "end_lineno": 2, "end_col_offset": 7 },
                "slice": {
                    "ast_type": "Tuple",
                    "elements": [
                        { "ast_type": "Name", "id": "address", "lineno": 2, "col_offset": 0, "end_lineno": 2, "end_col_offset": 7 },
                        { "ast_type": "Name", "id": "uint256", "lineno": 2, "col_offset": 0, "end_lineno": 2, "end_col_offset": 7 }
                    ],
                    "lineno": 2, "col_offset": 0, "end_lineno": 2, "end_col_offset": 30
                },
                "lineno": 2, "col_offset": 0, "end_lineno": 2, "end_col_offset": 30
            },
            "value": null,
            "is_constant": false,
            "is_immutable": false,
            "lineno": 2, "col_offset": 0, "end_lineno": 2, "end_col_offset": 30
        },
        {
            "ast_type": "FunctionDef",
            "name": "__init__",
            "args": {
                "ast_type": "arguments",
                "args": [
                    {
                        "ast_type": "arg",
                        "arg": "_supply",
                        "annotation": { "ast_type": "Name", "id": "uint256", "lineno": 4, "col_offset": 0, "end_lineno": 4, "end_col_offset": 6 },
                        "lineno": 4, "col_offset": 0, "end_lineno": 4, "end_col_offset": 20
                    }
                ],
                "default": null
            },
            "returns": null,
            "decorator_list": [
                { "ast_type": "Name", "id": "deploy", "lineno": 3, "col_offset": 0, "end_lineno": 3, "end_col_offset": 10 },
                { "ast_type": "Name", "id": "external", "lineno": 3, "col_offset": 0, "end_lineno": 3, "end_col_offset": 10 }
            ],
            "body": [
                {
                    "ast_type": "Assign",
                    "target": {
                        "ast_type": "Attribute",
                        "value": { "ast_type": "Name", "id": "self", "lineno": 5, "col_offset": 0, "end_lineno": 5, "end_col_offset": 4 },
                        "attr": "totalSupply",
                        "lineno": 5, "col_offset": 0, "end_lineno": 5, "end_col_offset": 15
                    },
                    "value": { "ast_type": "Name", "id": "_supply", "lineno": 5, "col_offset": 18, "end_lineno": 5, "end_col_offset": 25 },
                    "lineno": 5, "col_offset": 0, "end_lineno": 5, "end_col_offset": 25
                },
                {
                    "ast_type": "Assign",
                    "target": {
                        "ast_type": "Subscript",
                        "value": {
                            "ast_type": "Attribute",
                            "value": { "ast_type": "Name", "id": "self", "lineno": 6, "col_offset": 0, "end_lineno": 6, "end_col_offset": 4 },
                            "attr": "balanceOf",
                            "lineno": 6, "col_offset": 0, "end_lineno": 6, "end_col_offset": 13
                        },
                        "slice": {
                            "ast_type": "Attribute",
                            "value": { "ast_type": "Name", "id": "msg", "lineno": 6, "col_offset": 14, "end_lineno": 6, "end_col_offset": 17 },
                            "attr": "sender",
                            "lineno": 6, "col_offset": 14, "end_lineno": 6, "end_col_offset": 24
                        },
                        "lineno": 6, "col_offset": 0, "end_lineno": 6, "end_col_offset": 25
                    },
                    "value": { "ast_type": "Name", "id": "_supply", "lineno": 6, "col_offset": 27, "end_lineno": 6, "end_col_offset": 34 },
                    "lineno": 6, "col_offset": 0, "end_lineno": 6, "end_col_offset": 34
                }
            ],
            "lineno": 4, "col_offset": 0, "end_lineno": 7, "end_col_offset": 0
        },
        {
            "ast_type": "FunctionDef",
            "name": "transfer",
            "args": {
                "ast_type": "arguments",
                "args": [
                    {
                        "ast_type": "arg",
                        "arg": "_to",
                        "annotation": { "ast_type": "Name", "id": "address", "lineno": 9, "col_offset": 0, "end_lineno": 9, "end_col_offset": 7 },
                        "lineno": 9, "col_offset": 0, "end_lineno": 9, "end_col_offset": 12
                    },
                    {
                        "ast_type": "arg",
                        "arg": "_amount",
                        "annotation": { "ast_type": "Name", "id": "uint256", "lineno": 9, "col_offset": 14, "end_lineno": 9, "end_col_offset": 21 },
                        "lineno": 9, "col_offset": 14, "end_lineno": 9, "end_col_offset": 29
                    }
                ],
                "default": null
            },
            "returns": null,
            "decorator_list": [
                { "ast_type": "Name", "id": "external", "lineno": 8, "col_offset": 0, "end_lineno": 8, "end_col_offset": 10 }
            ],
            "body": [
                {
                    "ast_type": "Assert",
                    "test": {
                        "ast_type": "Compare",
                        "left": {
                            "ast_type": "Subscript",
                            "value": {
                                "ast_type": "Attribute",
                                "value": { "ast_type": "Name", "id": "self", "lineno": 10, "col_offset": 0, "end_lineno": 10, "end_col_offset": 4 },
                                "attr": "balanceOf",
                                "lineno": 10, "col_offset": 0, "end_lineno": 10, "end_col_offset": 13
                            },
                            "slice": {
                                "ast_type": "Attribute",
                                "value": { "ast_type": "Name", "id": "msg", "lineno": 10, "col_offset": 14, "end_lineno": 10, "end_col_offset": 17 },
                                "attr": "sender",
                                "lineno": 10, "col_offset": 14, "end_lineno": 10, "end_col_offset": 24
                            },
                            "lineno": 10, "col_offset": 0, "end_lineno": 10, "end_col_offset": 25
                        },
                        "ops": [{ "ast_type": "GtE" }],
                        "comparators": [
                            { "ast_type": "Name", "id": "_amount", "lineno": 10, "col_offset": 28, "end_lineno": 10, "end_col_offset": 35 }
                        ],
                        "lineno": 10, "col_offset": 0, "end_lineno": 10, "end_col_offset": 35
                    },
                    "msg": { "ast_type": "Constant", "value": "Insufficient balance", "lineno": 10, "col_offset": 37, "end_lineno": 10, "end_col_offset": 59 },
                    "lineno": 10, "col_offset": 0, "end_lineno": 10, "end_col_offset": 59
                },
                {
                    "ast_type": "AugAssign",
                    "target": {
                        "ast_type": "Subscript",
                        "value": {
                            "ast_type": "Attribute",
                            "value": { "ast_type": "Name", "id": "self", "lineno": 11, "col_offset": 0, "end_lineno": 11, "end_col_offset": 4 },
                            "attr": "balanceOf",
                            "lineno": 11, "col_offset": 0, "end_lineno": 11, "end_col_offset": 13
                        },
                        "slice": {
                            "ast_type": "Attribute",
                            "value": { "ast_type": "Name", "id": "msg", "lineno": 11, "col_offset": 14, "end_lineno": 11, "end_col_offset": 17 },
                            "attr": "sender",
                            "lineno": 11, "col_offset": 14, "end_lineno": 11, "end_col_offset": 24
                        },
                        "lineno": 11, "col_offset": 0, "end_lineno": 11, "end_col_offset": 25
                    },
                    "op": { "ast_type": "Sub" },
                    "value": { "ast_type": "Name", "id": "_amount", "lineno": 11, "col_offset": 29, "end_lineno": 11, "end_col_offset": 36 },
                    "lineno": 11, "col_offset": 0, "end_lineno": 11, "end_col_offset": 36
                },
                {
                    "ast_type": "AugAssign",
                    "target": {
                        "ast_type": "Subscript",
                        "value": {
                            "ast_type": "Attribute",
                            "value": { "ast_type": "Name", "id": "self", "lineno": 12, "col_offset": 0, "end_lineno": 12, "end_col_offset": 4 },
                            "attr": "balanceOf",
                            "lineno": 12, "col_offset": 0, "end_lineno": 12, "end_col_offset": 13
                        },
                        "slice": { "ast_type": "Name", "id": "_to", "lineno": 12, "col_offset": 14, "end_lineno": 12, "end_col_offset": 17 },
                        "lineno": 12, "col_offset": 0, "end_lineno": 12, "end_col_offset": 18
                    },
                    "op": { "ast_type": "Add" },
                    "value": { "ast_type": "Name", "id": "_amount", "lineno": 12, "col_offset": 22, "end_lineno": 12, "end_col_offset": 29 },
                    "lineno": 12, "col_offset": 0, "end_lineno": 12, "end_col_offset": 29
                }
            ],
            "lineno": 9, "col_offset": 0, "end_lineno": 13, "end_col_offset": 0
        }
    ],
    "lineno": 1, "col_offset": 0, "end_lineno": 13, "end_col_offset": 0
}"#;

/// A vault-like contract with nonreentrant, events, and msg.sender/msg.value
const VAULT_JSON: &str = r#"{
    "ast_type": "Module",
    "body": [
        {
            "ast_type": "EventDef",
            "name": "Deposit",
            "body": [
                {
                    "ast_type": "AnnAssign",
                    "target": { "ast_type": "Name", "id": "sender", "lineno": 2, "col_offset": 0, "end_lineno": 2, "end_col_offset": 6 },
                    "annotation": { "ast_type": "Call", "func": { "ast_type": "Name", "id": "indexed", "lineno": 2, "col_offset": 0, "end_lineno": 2, "end_col_offset": 7 }, "args": [{ "ast_type": "Name", "id": "address", "lineno": 2, "col_offset": 8, "end_lineno": 2, "end_col_offset": 15 }], "keywords": [], "lineno": 2, "col_offset": 0, "end_lineno": 2, "end_col_offset": 16 },
                    "value": null,
                    "lineno": 2, "col_offset": 0, "end_lineno": 2, "end_col_offset": 16
                },
                {
                    "ast_type": "AnnAssign",
                    "target": { "ast_type": "Name", "id": "amount", "lineno": 3, "col_offset": 0, "end_lineno": 3, "end_col_offset": 6 },
                    "annotation": { "ast_type": "Name", "id": "uint256", "lineno": 3, "col_offset": 0, "end_lineno": 3, "end_col_offset": 7 },
                    "value": null,
                    "lineno": 3, "col_offset": 0, "end_lineno": 3, "end_col_offset": 7
                }
            ],
            "lineno": 1, "col_offset": 0, "end_lineno": 4, "end_col_offset": 0
        },
        {
            "ast_type": "VariableDecl",
            "target": { "ast_type": "Name", "id": "balances", "lineno": 5, "col_offset": 0, "end_lineno": 5, "end_col_offset": 8 },
            "annotation": {
                "ast_type": "Subscript",
                "value": { "ast_type": "Name", "id": "HashMap", "lineno": 5, "col_offset": 0, "end_lineno": 5, "end_col_offset": 7 },
                "slice": {
                    "ast_type": "Tuple",
                    "elements": [
                        { "ast_type": "Name", "id": "address", "lineno": 5, "col_offset": 0, "end_lineno": 5, "end_col_offset": 7 },
                        { "ast_type": "Name", "id": "uint256", "lineno": 5, "col_offset": 0, "end_lineno": 5, "end_col_offset": 7 }
                    ],
                    "lineno": 5, "col_offset": 0, "end_lineno": 5, "end_col_offset": 30
                },
                "lineno": 5, "col_offset": 0, "end_lineno": 5, "end_col_offset": 30
            },
            "value": null,
            "is_constant": false,
            "is_immutable": false,
            "lineno": 5, "col_offset": 0, "end_lineno": 5, "end_col_offset": 30
        },
        {
            "ast_type": "FunctionDef",
            "name": "deposit",
            "args": { "ast_type": "arguments", "args": [], "default": null },
            "returns": null,
            "decorator_list": [
                { "ast_type": "Name", "id": "external", "lineno": 6, "col_offset": 0, "end_lineno": 6, "end_col_offset": 10 },
                { "ast_type": "Name", "id": "payable", "lineno": 6, "col_offset": 0, "end_lineno": 6, "end_col_offset": 10 },
                { "ast_type": "Call", "func": { "ast_type": "Name", "id": "nonreentrant", "lineno": 6, "col_offset": 0, "end_lineno": 6, "end_col_offset": 12 }, "args": [{ "ast_type": "Constant", "value": "lock", "lineno": 6, "col_offset": 13, "end_lineno": 6, "end_col_offset": 19 }], "keywords": [], "lineno": 6, "col_offset": 0, "end_lineno": 6, "end_col_offset": 20 }
            ],
            "body": [
                {
                    "ast_type": "AugAssign",
                    "target": {
                        "ast_type": "Subscript",
                        "value": {
                            "ast_type": "Attribute",
                            "value": { "ast_type": "Name", "id": "self", "lineno": 7, "col_offset": 0, "end_lineno": 7, "end_col_offset": 4 },
                            "attr": "balances",
                            "lineno": 7, "col_offset": 0, "end_lineno": 7, "end_col_offset": 12
                        },
                        "slice": {
                            "ast_type": "Attribute",
                            "value": { "ast_type": "Name", "id": "msg", "lineno": 7, "col_offset": 13, "end_lineno": 7, "end_col_offset": 16 },
                            "attr": "sender",
                            "lineno": 7, "col_offset": 13, "end_lineno": 7, "end_col_offset": 23
                        },
                        "lineno": 7, "col_offset": 0, "end_lineno": 7, "end_col_offset": 24
                    },
                    "op": { "ast_type": "Add" },
                    "value": {
                        "ast_type": "Attribute",
                        "value": { "ast_type": "Name", "id": "msg", "lineno": 7, "col_offset": 28, "end_lineno": 7, "end_col_offset": 31 },
                        "attr": "value",
                        "lineno": 7, "col_offset": 28, "end_lineno": 7, "end_col_offset": 37
                    },
                    "lineno": 7, "col_offset": 0, "end_lineno": 7, "end_col_offset": 37
                },
                {
                    "ast_type": "Log",
                    "value": {
                        "ast_type": "Call",
                        "func": { "ast_type": "Name", "id": "Deposit", "lineno": 8, "col_offset": 4, "end_lineno": 8, "end_col_offset": 11 },
                        "args": [
                            { "ast_type": "Attribute", "value": { "ast_type": "Name", "id": "msg", "lineno": 8, "col_offset": 12, "end_lineno": 8, "end_col_offset": 15 }, "attr": "sender", "lineno": 8, "col_offset": 12, "end_lineno": 8, "end_col_offset": 22 },
                            { "ast_type": "Attribute", "value": { "ast_type": "Name", "id": "msg", "lineno": 8, "col_offset": 24, "end_lineno": 8, "end_col_offset": 27 }, "attr": "value", "lineno": 8, "col_offset": 24, "end_lineno": 8, "end_col_offset": 33 }
                        ],
                        "keywords": [],
                        "lineno": 8, "col_offset": 4, "end_lineno": 8, "end_col_offset": 34
                    },
                    "lineno": 8, "col_offset": 0, "end_lineno": 8, "end_col_offset": 34
                }
            ],
            "lineno": 7, "col_offset": 0, "end_lineno": 9, "end_col_offset": 0
        }
    ],
    "lineno": 1, "col_offset": 0, "end_lineno": 9, "end_col_offset": 0
}"#;

// ═══════════════════════════════════════════════════════════════════
// Integration tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_full_pipeline_token_contract() {
    let module = compile_json(MINIMAL_TOKEN_JSON, "token.vy");

    // Check module ID
    assert_eq!(module.id, "token.vy");

    // Check module-level attrs
    assert!(
        module
            .attrs
            .iter()
            .any(|a| a.key == mlir::sir::attrs::sir_attrs::SOURCE_LANG
                && a.value == mlir::sir::AttrValue::String("vyper".to_string()))
    );

    // Should have exactly one contract decl
    assert_eq!(module.decls.len(), 1);
    let contract = match &module.decls[0] {
        mlir::sir::Decl::Contract(c) => c,
        _ => panic!("Expected Contract decl"),
    };
    assert_eq!(contract.name, "token");

    // Contract should have 4 members: 2 storage + 2 functions
    assert_eq!(contract.members.len(), 4);

    // Check storage vars
    let storage_members: Vec<&mlir::sir::StorageDecl> = contract
        .members
        .iter()
        .filter_map(|m| match m {
            mlir::sir::MemberDecl::Storage(s) => Some(s),
            _ => None,
        })
        .collect();
    assert_eq!(storage_members.len(), 2);
    assert_eq!(storage_members[0].name, "totalSupply");
    assert_eq!(storage_members[1].name, "balanceOf");

    // Check totalSupply type is uint256 (I256)
    assert_eq!(storage_members[0].ty, mlir::sir::Type::I256);

    // Check balanceOf type is Map(Address, I256)
    match &storage_members[1].ty {
        mlir::sir::Type::Map(k, v) => {
            assert_eq!(
                **k,
                mlir::sir::Type::Dialect(mlir::sir::DialectType::Evm(mlir::sir::dialect::evm::EvmType::Address))
            );
            assert_eq!(**v, mlir::sir::Type::I256);
        }
        other => panic!("Expected Map type, got: {other:?}"),
    }

    // Check functions
    let func_members: Vec<&mlir::sir::FunctionDecl> = contract
        .members
        .iter()
        .filter_map(|m| match m {
            mlir::sir::MemberDecl::Function(f) => Some(f),
            _ => None,
        })
        .collect();
    assert_eq!(func_members.len(), 2);

    // __init__ function (normalized: prefixed with contract name)
    let init_fn = func_members[0];
    assert_eq!(init_fn.name, "token____init__");
    assert_eq!(init_fn.params.len(), 1);
    // Parameter names are renamed by normalization (rename_vars pass)
    assert!(init_fn.params[0].name.contains("supply"));
    assert!(
        init_fn
            .attrs
            .iter()
            .any(|a| a.key == mlir::sir::attrs::evm_attrs::IS_CONSTRUCTOR)
    );
    assert!(
        init_fn
            .attrs
            .iter()
            .any(|a| a.key == mlir::sir::attrs::sir_attrs::VISIBILITY
                && a.value == mlir::sir::AttrValue::String("public".to_string()))
    );

    // transfer function (normalized: prefixed with contract name)
    let transfer_fn = func_members[1];
    assert_eq!(transfer_fn.name, "token__transfer");
    assert_eq!(transfer_fn.params.len(), 2);
    assert!(transfer_fn.params[0].name.contains("to"));
    assert!(transfer_fn.params[1].name.contains("amount"));

    // transfer has @external decorator → visibility=public
    assert!(
        transfer_fn
            .attrs
            .iter()
            .any(|a| a.key == mlir::sir::attrs::sir_attrs::VISIBILITY
                && a.value == mlir::sir::AttrValue::String("public".to_string()))
    );

    // transfer body should have 3 stmts: assert + 2 aug_assign
    let body = transfer_fn.body.as_ref().expect("should have body");
    assert_eq!(body.len(), 3);
    assert!(matches!(body[0], mlir::sir::Stmt::Assert(_)));
    assert!(matches!(body[1], mlir::sir::Stmt::AugAssign(_)));
    assert!(matches!(body[2], mlir::sir::Stmt::AugAssign(_)));
}

#[test]
fn test_full_pipeline_vault_contract() {
    let module = compile_json(VAULT_JSON, "vault.vy");

    assert_eq!(module.id, "vault.vy");
    assert_eq!(module.decls.len(), 1);

    let contract = match &module.decls[0] {
        mlir::sir::Decl::Contract(c) => c,
        _ => panic!("Expected Contract decl"),
    };
    assert_eq!(contract.name, "vault");

    // Should have: EventDef (Dialect), storage, function
    assert_eq!(contract.members.len(), 3);

    // EventDef
    match &contract.members[0] {
        mlir::sir::MemberDecl::Dialect(mlir::sir::DialectMemberDecl::Evm(
            mlir::sir::dialect::evm::EvmMemberDecl::EventDef { name, params, indexed, anonymous },
        )) => {
            assert_eq!(name, "Deposit");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].0, "sender");
            assert_eq!(params[1].0, "amount");
            assert!(indexed[0]); // sender is indexed
            assert!(!indexed[1]); // amount is not indexed
            assert!(!anonymous);
        }
        other => panic!("Expected EventDef, got: {other:?}"),
    }

    // Storage
    match &contract.members[1] {
        mlir::sir::MemberDecl::Storage(s) => {
            assert_eq!(s.name, "balances");
        }
        other => panic!("Expected Storage, got: {other:?}"),
    }

    // deposit function
    match &contract.members[2] {
        mlir::sir::MemberDecl::Function(f) => {
            assert_eq!(f.name, "vault__deposit");
            assert_eq!(f.params.len(), 0);

            // Check decorators
            assert!(
                f.attrs
                    .iter()
                    .any(|a| a.key == mlir::sir::attrs::sir_attrs::VISIBILITY
                        && a.value == mlir::sir::AttrValue::String("public".to_string()))
            );
            assert!(
                f.attrs
                    .iter()
                    .any(|a| a.key == mlir::sir::attrs::evm_attrs::PAYABLE)
            );
            assert!(
                f.attrs
                    .iter()
                    .any(|a| a.key == mlir::sir::attrs::evm_attrs::NONREENTRANT)
            );

            // Body: 2 stmts (AugAssign, EmitEvent)
            let body = f.body.as_ref().expect("body");
            assert_eq!(body.len(), 2);
            assert!(matches!(body[0], mlir::sir::Stmt::AugAssign(_)));

            // EmitEvent
            match &body[1] {
                mlir::sir::Stmt::Dialect(mlir::sir::DialectStmt::Evm(
                    mlir::sir::dialect::evm::EvmStmt::EmitEvent { event, args, .. },
                )) => {
                    assert_eq!(event, "Deposit");
                    assert_eq!(args.len(), 2);
                    // First arg should be msg.sender → EvmExpr::MsgSender
                    assert!(matches!(
                        &args[0],
                        mlir::sir::Expr::Dialect(mlir::sir::DialectExpr::Evm(
                            mlir::sir::dialect::evm::EvmExpr::MsgSender
                        ))
                    ));
                    // Second arg should be msg.value → EvmExpr::MsgValue
                    assert!(matches!(
                        &args[1],
                        mlir::sir::Expr::Dialect(mlir::sir::DialectExpr::Evm(
                            mlir::sir::dialect::evm::EvmExpr::MsgValue
                        ))
                    ));
                }
                other => panic!("Expected EmitEvent, got: {other:?}"),
            }
        }
        other => panic!("Expected Function, got: {other:?}"),
    }
}

#[test]
fn test_module_display_output() {
    let module = compile_json(MINIMAL_TOKEN_JSON, "token.vy");
    let display = format!("{module}");
    // Module Display should produce non-empty output
    assert!(!display.is_empty());
    // Should contain the contract name
    assert!(display.contains("token"));
}

#[test]
fn test_empty_module() {
    let json = r#"{
        "ast_type": "Module",
        "body": [],
        "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 0
    }"#;
    let module = compile_json(json, "empty.vy");
    assert_eq!(module.id, "empty.vy");
    assert_eq!(module.decls.len(), 1);
    match &module.decls[0] {
        mlir::sir::Decl::Contract(c) => {
            assert_eq!(c.name, "empty");
            assert!(c.members.is_empty());
        }
        _ => panic!("Expected Contract"),
    }
}

#[test]
fn test_normalization_applied() {
    // Ensure the normalization pass works through the pipeline.
    // The normalization should run without errors even if it
    // doesn't visibly change simple contracts.
    let json = r#"{
        "ast_type": "Module",
        "body": [
            {
                "ast_type": "FunctionDef",
                "name": "foo",
                "args": { "ast_type": "arguments", "args": [], "default": null },
                "returns": null,
                "decorator_list": [{ "ast_type": "Name", "id": "external", "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 10 }],
                "body": [
                    {
                        "ast_type": "AnnAssign",
                        "target": { "ast_type": "Name", "id": "x", "lineno": 2, "col_offset": 0, "end_lineno": 2, "end_col_offset": 1 },
                        "annotation": { "ast_type": "Name", "id": "uint256", "lineno": 2, "col_offset": 0, "end_lineno": 2, "end_col_offset": 7 },
                        "value": { "ast_type": "Constant", "value": 42, "lineno": 2, "col_offset": 12, "end_lineno": 2, "end_col_offset": 14 },
                        "lineno": 2, "col_offset": 0, "end_lineno": 2, "end_col_offset": 14
                    },
                    {
                        "ast_type": "Return",
                        "value": { "ast_type": "Name", "id": "x", "lineno": 3, "col_offset": 7, "end_lineno": 3, "end_col_offset": 8 },
                        "lineno": 3, "col_offset": 0, "end_lineno": 3, "end_col_offset": 8
                    }
                ],
                "lineno": 1, "col_offset": 0, "end_lineno": 4, "end_col_offset": 0
            }
        ],
        "lineno": 1, "col_offset": 0, "end_lineno": 4, "end_col_offset": 0
    }"#;
    let module = compile_json(json, "norm_test.vy");
    assert_eq!(module.decls.len(), 1);

    let contract = match &module.decls[0] {
        mlir::sir::Decl::Contract(c) => c,
        _ => panic!("Expected Contract"),
    };
    assert_eq!(contract.members.len(), 1);

    match &contract.members[0] {
        mlir::sir::MemberDecl::Function(f) => {
            assert_eq!(f.name, "norm_test__foo");
            let body = f.body.as_ref().unwrap();
            assert_eq!(body.len(), 2); // local var + return
            assert!(matches!(body[0], mlir::sir::Stmt::LocalVar(_)));
            assert!(matches!(body[1], mlir::sir::Stmt::Return(_)));
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_if_statement_lowering() {
    let json = r#"{
        "ast_type": "Module",
        "body": [
            {
                "ast_type": "FunctionDef",
                "name": "check",
                "args": {
                    "ast_type": "arguments",
                    "args": [
                        {
                            "ast_type": "arg",
                            "arg": "x",
                            "annotation": { "ast_type": "Name", "id": "uint256", "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 6 },
                            "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 10
                        }
                    ],
                    "default": null
                },
                "returns": { "ast_type": "Name", "id": "bool", "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 4 },
                "decorator_list": [{ "ast_type": "Name", "id": "external", "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 10 }],
                "body": [
                    {
                        "ast_type": "If",
                        "test": {
                            "ast_type": "Compare",
                            "left": { "ast_type": "Name", "id": "x", "lineno": 2, "col_offset": 3, "end_lineno": 2, "end_col_offset": 4 },
                            "ops": [{ "ast_type": "Gt" }],
                            "comparators": [{ "ast_type": "Constant", "value": 0, "lineno": 2, "col_offset": 7, "end_lineno": 2, "end_col_offset": 8 }],
                            "lineno": 2, "col_offset": 3, "end_lineno": 2, "end_col_offset": 8
                        },
                        "body": [
                            {
                                "ast_type": "Return",
                                "value": { "ast_type": "Constant", "value": true, "lineno": 3, "col_offset": 11, "end_lineno": 3, "end_col_offset": 15 },
                                "lineno": 3, "col_offset": 4, "end_lineno": 3, "end_col_offset": 15
                            }
                        ],
                        "orelse": [
                            {
                                "ast_type": "Return",
                                "value": { "ast_type": "Constant", "value": false, "lineno": 5, "col_offset": 11, "end_lineno": 5, "end_col_offset": 16 },
                                "lineno": 5, "col_offset": 4, "end_lineno": 5, "end_col_offset": 16
                            }
                        ],
                        "lineno": 2, "col_offset": 0, "end_lineno": 6, "end_col_offset": 0
                    }
                ],
                "lineno": 1, "col_offset": 0, "end_lineno": 6, "end_col_offset": 0
            }
        ],
        "lineno": 1, "col_offset": 0, "end_lineno": 6, "end_col_offset": 0
    }"#;
    let module = compile_json(json, "check.vy");
    let contract = match &module.decls[0] {
        mlir::sir::Decl::Contract(c) => c,
        _ => panic!("Expected Contract"),
    };

    let func = match &contract.members[0] {
        mlir::sir::MemberDecl::Function(f) => f,
        _ => panic!("Expected Function"),
    };
    assert_eq!(func.name, "check__check");
    assert_eq!(func.returns.len(), 1);
    assert_eq!(func.returns[0], mlir::sir::Type::Bool);

    let body = func.body.as_ref().unwrap();
    assert_eq!(body.len(), 1);
    match &body[0] {
        mlir::sir::Stmt::If(if_stmt) => {
            assert_eq!(if_stmt.then_body.len(), 1);
            assert!(matches!(if_stmt.then_body[0], mlir::sir::Stmt::Return(_)));
            let else_body = if_stmt.else_body.as_ref().expect("else_body");
            assert_eq!(else_body.len(), 1);
            assert!(matches!(else_body[0], mlir::sir::Stmt::Return(_)));
        }
        _ => panic!("Expected If statement"),
    }
}

#[test]
fn test_for_range_lowering() {
    let json = r#"{
        "ast_type": "Module",
        "body": [
            {
                "ast_type": "FunctionDef",
                "name": "loop_fn",
                "args": { "ast_type": "arguments", "args": [], "default": null },
                "returns": null,
                "decorator_list": [{ "ast_type": "Name", "id": "internal", "lineno": 1, "col_offset": 0, "end_lineno": 1, "end_col_offset": 10 }],
                "body": [
                    {
                        "ast_type": "For",
                        "target": { "ast_type": "Name", "id": "i", "lineno": 2, "col_offset": 4, "end_lineno": 2, "end_col_offset": 5 },
                        "iter": {
                            "ast_type": "Call",
                            "func": { "ast_type": "Name", "id": "range", "lineno": 2, "col_offset": 9, "end_lineno": 2, "end_col_offset": 14 },
                            "args": [{ "ast_type": "Constant", "value": 10, "lineno": 2, "col_offset": 15, "end_lineno": 2, "end_col_offset": 17 }],
                            "keywords": [],
                            "lineno": 2, "col_offset": 9, "end_lineno": 2, "end_col_offset": 18
                        },
                        "body": [
                            {
                                "ast_type": "Pass",
                                "lineno": 3, "col_offset": 4, "end_lineno": 3, "end_col_offset": 8
                            }
                        ],
                        "lineno": 2, "col_offset": 0, "end_lineno": 4, "end_col_offset": 0
                    }
                ],
                "lineno": 1, "col_offset": 0, "end_lineno": 4, "end_col_offset": 0
            }
        ],
        "lineno": 1, "col_offset": 0, "end_lineno": 4, "end_col_offset": 0
    }"#;
    let module = compile_json(json, "loop.vy");
    let contract = match &module.decls[0] {
        mlir::sir::Decl::Contract(c) => c,
        _ => panic!("Expected Contract"),
    };

    let func = match &contract.members[0] {
        mlir::sir::MemberDecl::Function(f) => f,
        _ => panic!("Expected Function"),
    };

    let body = func.body.as_ref().unwrap();
    assert_eq!(body.len(), 1);
    assert!(matches!(body[0], mlir::sir::Stmt::For(_)));
}
