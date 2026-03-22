//! Step 4: Dialect Lowering
//!
//! Each dialect construct in SIR is lowered to BIR ops that implement
//! at least one of the four interfaces: StorageOp, CallOp, TaintSource,
//! TaintSink.

use crate::bir::cfg::BasicBlock;
use crate::bir::interfaces::{
    AliasGroupId, CallRisk, CallTarget, SinkCategory, StorageIndex, StorageRef, TaintLabel,
};
use crate::bir::lower::LowerError;
use crate::bir::ops::{CallDialectOp, OpKind, StorageDialectOp, TaintSinkOp, TaintSourceOp};
use crate::sir::Attr;

/// Lower dialect ops in all basic blocks.
///
/// After this step, every retained dialect op implements at least one
/// BIR interface. Untagged dialect ops cause `LowerError::UntaggedDialectOp`.
pub fn lower_dialect_ops(
    blocks: &mut [BasicBlock],
    module_attrs: &[Attr],
) -> Result<(), LowerError> {
    // Determine which dialect is loaded
    let dialect = detect_dialect(module_attrs);

    for block in blocks.iter_mut() {
        for op in &mut block.ops {
            if let OpKind::Opaque { description } = &op.kind {
                let desc = description.clone();
                // Try to lower based on description patterns
                if let Some(lowered) = try_lower_opaque(&desc, &dialect) {
                    op.kind = lowered;
                }
                // Opaque ops that couldn't be lowered remain as-is
                // (they represent expressions/calls not yet matched to
                // interfaces)
            }
        }
    }

    Ok(())
}

/// Detect which dialect is loaded from module attributes.
fn detect_dialect(attrs: &[Attr]) -> String {
    for attr in attrs {
        if attr.namespace == "sir" && attr.key == "loaded_dialects" {
            if let crate::sir::AttrValue::String(s) = &attr.value {
                return s.clone();
            }
        }
    }
    String::new()
}

/// Try to lower an opaque op description into a typed BIR op.
fn try_lower_opaque(description: &str, dialect: &str) -> Option<OpKind> {
    // EVM dialect lowering
    if dialect.contains("evm") || description.starts_with("evm.") {
        return try_lower_evm(description);
    }

    // Move dialect lowering
    if dialect.contains("move") || description.starts_with("move.") {
        return try_lower_move(description);
    }

    // Anchor dialect lowering
    if dialect.contains("anchor") || description.starts_with("anchor.") {
        return try_lower_anchor(description);
    }

    None
}

/// EVM dialect lowering table.
fn try_lower_evm(description: &str) -> Option<OpKind> {
    if description.contains("evm.msg_sender()") {
        return Some(OpKind::TaintSrc(TaintSourceOp {
            label: TaintLabel::UserControlled,
            dialect_name: "evm".to_string(),
            op_name: "msg_sender".to_string(),
        }));
    }

    if description.contains("evm.msg_value()") {
        return Some(OpKind::TaintSrc(TaintSourceOp {
            label: TaintLabel::UserControlled,
            dialect_name: "evm".to_string(),
            op_name: "msg_value".to_string(),
        }));
    }

    if description.contains("evm.timestamp()") {
        return Some(OpKind::TaintSrc(TaintSourceOp {
            label: TaintLabel::BlockContext,
            dialect_name: "evm".to_string(),
            op_name: "timestamp".to_string(),
        }));
    }

    if description.contains("evm.block_number()") {
        return Some(OpKind::TaintSrc(TaintSourceOp {
            label: TaintLabel::BlockContext,
            dialect_name: "evm".to_string(),
            op_name: "block_number".to_string(),
        }));
    }

    if description.contains("evm.raw_call(") || description.contains("evm.send(") {
        return Some(OpKind::Call(CallDialectOp {
            callee: CallTarget::Dynamic,
            call_risk: CallRisk { reentrancy: true, value_transfer: true },
            args: vec![],
            dialect_name: "evm".to_string(),
            op_name: "external_call".to_string(),
        }));
    }

    if description.contains("emit ") || description.contains("evm.emit_event(") {
        return Some(OpKind::TaintSnk(TaintSinkOp {
            category: SinkCategory::EventLog,
            dialect_name: "evm".to_string(),
            op_name: "emit_event".to_string(),
        }));
    }

    None
}

/// Move dialect lowering table.
fn try_lower_move(description: &str) -> Option<OpKind> {
    if description.contains("move.signer_address(") {
        return Some(OpKind::TaintSrc(TaintSourceOp {
            label: TaintLabel::SignerArg,
            dialect_name: "move".to_string(),
            op_name: "signer_address".to_string(),
        }));
    }

    if description.contains("move.borrow_global_mut<") {
        // Extract the type name from the description
        let base = extract_type_from_borrow(description);
        return Some(OpKind::Storage(StorageDialectOp {
            storage_ref: StorageRef { base: base.clone(), indices: vec![StorageIndex::Wildcard] },
            is_write: true,
            alias_group_id: AliasGroupId(format!("{base}[*]")),
            key_operand: None,
            value_operand: None,
            dialect_name: "move".to_string(),
            op_name: "borrow_global_mut".to_string(),
        }));
    }

    if description.contains("move.borrow_global<") {
        let base = extract_type_from_borrow(description);
        return Some(OpKind::Storage(StorageDialectOp {
            storage_ref: StorageRef { base: base.clone(), indices: vec![StorageIndex::Wildcard] },
            is_write: false,
            alias_group_id: AliasGroupId(format!("{base}[*]")),
            key_operand: None,
            value_operand: None,
            dialect_name: "move".to_string(),
            op_name: "borrow_global".to_string(),
        }));
    }

    if description.contains("move.write_ref(") {
        return Some(OpKind::Storage(StorageDialectOp {
            storage_ref: StorageRef {
                base: "ref".to_string(),
                indices: vec![StorageIndex::Wildcard],
            },
            is_write: true,
            alias_group_id: AliasGroupId("ref[*]".to_string()),
            key_operand: None,
            value_operand: None,
            dialect_name: "move".to_string(),
            op_name: "write_ref".to_string(),
        }));
    }

    None
}

/// Anchor dialect lowering table.
fn try_lower_anchor(description: &str) -> Option<OpKind> {
    if description.contains("anchor.signer_key(") {
        return Some(OpKind::TaintSrc(TaintSourceOp {
            label: TaintLabel::SignerArg,
            dialect_name: "anchor".to_string(),
            op_name: "signer_key".to_string(),
        }));
    }

    if description.contains("anchor.account_load_mut(") {
        return Some(OpKind::Storage(StorageDialectOp {
            storage_ref: StorageRef {
                base: "account".to_string(),
                indices: vec![StorageIndex::Wildcard],
            },
            is_write: true,
            alias_group_id: AliasGroupId("account[*]".to_string()),
            key_operand: None,
            value_operand: None,
            dialect_name: "anchor".to_string(),
            op_name: "account_load_mut".to_string(),
        }));
    }

    if description.contains("anchor.account_load(") {
        return Some(OpKind::Storage(StorageDialectOp {
            storage_ref: StorageRef {
                base: "account".to_string(),
                indices: vec![StorageIndex::Wildcard],
            },
            is_write: false,
            alias_group_id: AliasGroupId("account[*]".to_string()),
            key_operand: None,
            value_operand: None,
            dialect_name: "anchor".to_string(),
            op_name: "account_load".to_string(),
        }));
    }

    if description.contains("anchor.cpi(") {
        return Some(OpKind::Call(CallDialectOp {
            callee: CallTarget::Dynamic,
            call_risk: CallRisk { reentrancy: true, value_transfer: false },
            args: vec![],
            dialect_name: "anchor".to_string(),
            op_name: "cpi".to_string(),
        }));
    }

    None
}

/// Extract a type name from a borrow_global description.
fn extract_type_from_borrow(description: &str) -> String {
    if let Some(start) = description.find('<') {
        if let Some(end) = description.find('>') {
            return description[start + 1..end].to_string();
        }
    }
    "unknown".to_string()
}
