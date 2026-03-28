//! EVM dialect expressions with enum-of-structs pattern.

use crate::sir::exprs::Expr;
use crate::sir::types::Type;
use common::loc::Loc;
use std::fmt::{self, Display};

use super::types::EvmType;

// ═══════════════════════════════════════════════════════════════════
// Struct definitions for each EvmExpr variant
// ═══════════════════════════════════════════════════════════════════

/// `evm.msg_sender()` — `msg.sender`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmMsgSender {
    pub loc: Loc,
}

/// `evm.msg_value()` — `msg.value`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmMsgValue {
    pub loc: Loc,
}

/// `evm.timestamp()` — `block.timestamp`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmTimestamp {
    pub loc: Loc,
}

/// `evm.block_number()` — `block.number`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmBlockNumber {
    pub loc: Loc,
}

/// `evm.tx_origin()` — `tx.origin`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmTxOrigin {
    pub loc: Loc,
}

/// `evm.inline_asm` — opaque inline assembly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmInlineAsm {
    pub asm_text: String,
    pub loc: Loc,
}

/// `convert(x, T)` — Vyper's explicit type cast builtin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmConvert {
    pub expr: Box<Expr>,
    pub to: Type,
    pub loc: Loc,
}

/// `slice(x, start, len)` — byte slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmSlice {
    pub expr: Box<Expr>,
    pub start: Box<Expr>,
    pub length: Box<Expr>,
    pub loc: Loc,
}

/// `len(x)` — length of DynArray or Bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmLen {
    pub expr: Box<Expr>,
    pub loc: Loc,
}

/// `raw_call(target, data, value?, gas?)` — low-level call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmRawCall {
    pub target: Box<Expr>,
    pub data: Box<Expr>,
    pub value: Option<Box<Expr>>,
    pub gas: Option<Box<Expr>>,
    pub loc: Loc,
}

/// `send(target, value)` — Vyper's send() builtin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmSend {
    pub target: Box<Expr>,
    pub value: Box<Expr>,
    pub loc: Loc,
}

/// `self.balance` — contract's own balance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmSelfBalance {
    pub loc: Loc,
}

/// `empty(T)` — zero value of type T.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmEmpty {
    pub ty: Type,
    pub loc: Loc,
}

/// `concat(a, b, ...)` — byte/string concatenation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmConcat {
    pub exprs: Vec<Expr>,
    pub loc: Loc,
}

/// `evm.delegatecall(target, data)` — low-level delegatecall.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmDelegatecall {
    pub target: Box<Expr>,
    pub data: Box<Expr>,
    pub loc: Loc,
}

/// `evm.low_level_call(target, data, value?, gas?)` — `.call{value:…}(…)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmLowLevelCall {
    pub target: Box<Expr>,
    pub data: Box<Expr>,
    pub value: Option<Box<Expr>>,
    pub gas: Option<Box<Expr>>,
    pub loc: Loc,
}

/// `keccak256(data)` — Keccak-256 hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmKeccak256 {
    pub expr: Box<Expr>,
    pub loc: Loc,
}

/// `sha256(data)` — SHA-256 hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmSha256 {
    pub expr: Box<Expr>,
    pub loc: Loc,
}

/// `ripemd160(data)` — RIPEMD-160 hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmRipemd160 {
    pub expr: Box<Expr>,
    pub loc: Loc,
}

/// `ecrecover(hash, v, r, s)` — recover signer address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmEcrecover {
    pub hash: Box<Expr>,
    pub v: Box<Expr>,
    pub r: Box<Expr>,
    pub s: Box<Expr>,
    pub loc: Loc,
}

/// `addmod(x, y, k)` — (x + y) % k.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmAddmod {
    pub x: Box<Expr>,
    pub y: Box<Expr>,
    pub k: Box<Expr>,
    pub loc: Loc,
}

/// `mulmod(x, y, k)` — (x * y) % k.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmMulmod {
    pub x: Box<Expr>,
    pub y: Box<Expr>,
    pub k: Box<Expr>,
    pub loc: Loc,
}

/// `gasleft()` — remaining gas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmGasleft {
    pub loc: Loc,
}

/// `blockhash(blockNumber)` — hash of given block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmBlockhash {
    pub expr: Box<Expr>,
    pub loc: Loc,
}

/// `abi.encode(args...)` — ABI-encode arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmAbiEncode {
    pub args: Vec<Expr>,
    pub loc: Loc,
}

/// `abi.encodePacked(args...)` — Packed ABI encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmAbiEncodePacked {
    pub args: Vec<Expr>,
    pub loc: Loc,
}

/// `abi.decode(data, (types...))` — ABI decode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmAbiDecode {
    pub data: Box<Expr>,
    pub types: Vec<Type>,
    pub loc: Loc,
}

/// `abi.encodeWithSelector(sel, args...)` — encode with function selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmAbiEncodeWithSelector {
    pub selector: Box<Expr>,
    pub args: Vec<Expr>,
    pub loc: Loc,
}

/// `abi.encodeWithSignature(sig, args...)` — encode with string signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmAbiEncodeWithSignature {
    pub signature: Box<Expr>,
    pub args: Vec<Expr>,
    pub loc: Loc,
}

/// `abi.encodeCall(func, args...)` — encode a function call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmAbiEncodeCall {
    pub func: Box<Expr>,
    pub args: Vec<Expr>,
    pub loc: Loc,
}

/// `addr.transfer(amount)` — transfer Ether, reverts on failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmTransfer {
    pub target: Box<Expr>,
    pub amount: Box<Expr>,
    pub loc: Loc,
}

/// `this` — reference to the current contract instance (as address).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmThis {
    pub loc: Loc,
}

/// `super` — reference to the parent contract for method dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmSuper {
    pub loc: Loc,
}

/// `msg.data` — complete calldata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmMsgData {
    pub loc: Loc,
}

/// `msg.sig` — first 4 bytes of calldata (function selector).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmMsgSig {
    pub loc: Loc,
}

/// `block.difficulty` / `block.prevrandao` — previous block's RANDAO.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmBlockDifficulty {
    pub loc: Loc,
}

/// `block.gaslimit` — block gas limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmBlockGaslimit {
    pub loc: Loc,
}

/// `block.coinbase` — current block miner's address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmBlockCoinbase {
    pub loc: Loc,
}

/// `block.chainid` — current chain ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmBlockChainid {
    pub loc: Loc,
}

/// `block.basefee` — current block's base fee.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmBlockBasefee {
    pub loc: Loc,
}

// ═══════════════════════════════════════════════════════════════════
// EvmExpr enum (enum-of-structs)
// ═══════════════════════════════════════════════════════════════════

/// EVM-specific expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvmExpr {
    MsgSender(EvmMsgSender),
    MsgValue(EvmMsgValue),
    Timestamp(EvmTimestamp),
    BlockNumber(EvmBlockNumber),
    TxOrigin(EvmTxOrigin),
    InlineAsm(EvmInlineAsm),
    Convert(EvmConvert),
    Slice(EvmSlice),
    Len(EvmLen),
    RawCall(EvmRawCall),
    Send(EvmSend),
    SelfBalance(EvmSelfBalance),
    Empty(EvmEmpty),
    Concat(EvmConcat),
    Delegatecall(EvmDelegatecall),
    LowLevelCall(EvmLowLevelCall),
    Keccak256(EvmKeccak256),
    Sha256(EvmSha256),
    Ripemd160(EvmRipemd160),
    Ecrecover(EvmEcrecover),
    Addmod(EvmAddmod),
    Mulmod(EvmMulmod),
    Gasleft(EvmGasleft),
    Blockhash(EvmBlockhash),
    AbiEncode(EvmAbiEncode),
    AbiEncodePacked(EvmAbiEncodePacked),
    AbiDecode(EvmAbiDecode),
    AbiEncodeWithSelector(EvmAbiEncodeWithSelector),
    AbiEncodeWithSignature(EvmAbiEncodeWithSignature),
    AbiEncodeCall(EvmAbiEncodeCall),
    Transfer(EvmTransfer),
    This(EvmThis),
    Super(EvmSuper),
    MsgData(EvmMsgData),
    MsgSig(EvmMsgSig),
    BlockDifficulty(EvmBlockDifficulty),
    BlockGaslimit(EvmBlockGaslimit),
    BlockCoinbase(EvmBlockCoinbase),
    BlockChainid(EvmBlockChainid),
    BlockBasefee(EvmBlockBasefee),
}

impl Display for EvmExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvmExpr::MsgSender(_) => write!(f, "evm.msg_sender()"),
            EvmExpr::MsgValue(_) => write!(f, "evm.msg_value()"),
            EvmExpr::Timestamp(_) => write!(f, "evm.timestamp()"),
            EvmExpr::BlockNumber(_) => write!(f, "evm.block_number()"),
            EvmExpr::TxOrigin(_) => write!(f, "evm.tx_origin()"),
            EvmExpr::InlineAsm(e) => write!(f, "evm.inline_asm({})", e.asm_text),
            EvmExpr::Convert(e) => write!(f, "evm.convert({}, {})", e.expr, e.to),
            EvmExpr::Slice(e) => {
                write!(f, "evm.slice({}, {}, {})", e.expr, e.start, e.length)
            }
            EvmExpr::Len(e) => write!(f, "evm.len({})", e.expr),
            EvmExpr::RawCall(e) => {
                write!(f, "evm.raw_call({}, {}", e.target, e.data)?;
                if let Some(v) = &e.value {
                    write!(f, ", value={v}")?;
                }
                if let Some(g) = &e.gas {
                    write!(f, ", gas={g}")?;
                }
                write!(f, ")")
            }
            EvmExpr::Send(e) => {
                write!(f, "evm.send({}, {})", e.target, e.value)
            }
            EvmExpr::SelfBalance(_) => write!(f, "evm.self_balance()"),
            EvmExpr::Empty(e) => write!(f, "evm.empty({})", e.ty),
            EvmExpr::Concat(e) => {
                let es: Vec<_> = e.exprs.iter().map(|e| e.to_string()).collect();
                write!(f, "evm.concat({})", es.join(", "))
            }
            EvmExpr::Delegatecall(e) => {
                write!(f, "evm.delegatecall({}, {})", e.target, e.data)
            }
            EvmExpr::LowLevelCall(e) => {
                write!(f, "evm.low_level_call({}, {}", e.target, e.data)?;
                if let Some(v) = &e.value {
                    write!(f, ", value={v}")?;
                }
                if let Some(g) = &e.gas {
                    write!(f, ", gas={g}")?;
                }
                write!(f, ")")
            }
            EvmExpr::Keccak256(e) => write!(f, "evm.keccak256({})", e.expr),
            EvmExpr::Sha256(e) => write!(f, "evm.sha256({})", e.expr),
            EvmExpr::Ripemd160(e) => write!(f, "evm.ripemd160({})", e.expr),
            EvmExpr::Ecrecover(e) => {
                write!(f, "evm.ecrecover({}, {}, {}, {})", e.hash, e.v, e.r, e.s)
            }
            EvmExpr::Addmod(e) => write!(f, "evm.addmod({}, {}, {})", e.x, e.y, e.k),
            EvmExpr::Mulmod(e) => write!(f, "evm.mulmod({}, {}, {})", e.x, e.y, e.k),
            EvmExpr::Gasleft(_) => write!(f, "evm.gasleft()"),
            EvmExpr::Blockhash(e) => write!(f, "evm.blockhash({})", e.expr),
            EvmExpr::AbiEncode(e) => {
                let a: Vec<_> = e.args.iter().map(|e| e.to_string()).collect();
                write!(f, "evm.abi_encode({})", a.join(", "))
            }
            EvmExpr::AbiEncodePacked(e) => {
                let a: Vec<_> = e.args.iter().map(|e| e.to_string()).collect();
                write!(f, "evm.abi_encode_packed({})", a.join(", "))
            }
            EvmExpr::AbiDecode(e) => {
                let ts: Vec<_> = e.types.iter().map(|t| t.to_string()).collect();
                write!(f, "evm.abi_decode({}, ({}))", e.data, ts.join(", "))
            }
            EvmExpr::AbiEncodeWithSelector(e) => {
                let a: Vec<_> = e.args.iter().map(|e| e.to_string()).collect();
                write!(f, "evm.abi_encode_with_selector({}, {})", e.selector, a.join(", "))
            }
            EvmExpr::AbiEncodeWithSignature(e) => {
                let a: Vec<_> = e.args.iter().map(|e| e.to_string()).collect();
                write!(f, "evm.abi_encode_with_signature({}, {})", e.signature, a.join(", "))
            }
            EvmExpr::AbiEncodeCall(e) => {
                let a: Vec<_> = e.args.iter().map(|e| e.to_string()).collect();
                write!(f, "evm.abi_encode_call({}, {})", e.func, a.join(", "))
            }
            EvmExpr::Transfer(e) => write!(f, "evm.transfer({}, {})", e.target, e.amount),
            EvmExpr::This(_) => write!(f, "evm.this"),
            EvmExpr::Super(_) => write!(f, "evm.super"),
            EvmExpr::MsgData(_) => write!(f, "evm.msg_data()"),
            EvmExpr::MsgSig(_) => write!(f, "evm.msg_sig()"),
            EvmExpr::BlockDifficulty(_) => write!(f, "evm.block_difficulty()"),
            EvmExpr::BlockGaslimit(_) => write!(f, "evm.block_gaslimit()"),
            EvmExpr::BlockCoinbase(_) => write!(f, "evm.block_coinbase()"),
            EvmExpr::BlockChainid(_) => write!(f, "evm.block_chainid()"),
            EvmExpr::BlockBasefee(_) => write!(f, "evm.block_basefee()"),
        }
    }
}

impl EvmExpr {
    /// Return type of this EVM dialect expression.
    pub fn typ(&self) -> Type {
        use crate::sir::dialect::DialectType;
        match self {
            EvmExpr::MsgSender(_) => Type::Dialect(DialectType::Evm(EvmType::Address)),
            EvmExpr::MsgValue(_) => Type::I256,
            EvmExpr::Timestamp(_) => Type::I256,
            EvmExpr::BlockNumber(_) => Type::I256,
            EvmExpr::TxOrigin(_) => Type::Dialect(DialectType::Evm(EvmType::Address)),
            EvmExpr::InlineAsm(_) => Type::None,
            EvmExpr::Convert(e) => e.to.clone(),
            EvmExpr::Slice(_) => Type::Bytes,
            EvmExpr::Len(_) => Type::I256,
            EvmExpr::RawCall(_) => Type::Bytes,
            EvmExpr::Send(_) => Type::Bool,
            EvmExpr::SelfBalance(_) => Type::I256,
            EvmExpr::Empty(e) => e.ty.clone(),
            EvmExpr::Concat(_) => Type::Bytes,
            EvmExpr::Delegatecall(_) => Type::Tuple(vec![Type::Bool, Type::Bytes]),
            EvmExpr::LowLevelCall(_) => Type::Tuple(vec![Type::Bool, Type::Bytes]),
            EvmExpr::Keccak256(_) => Type::FixedBytes(32),
            EvmExpr::Sha256(_) => Type::FixedBytes(32),
            EvmExpr::Ripemd160(_) => Type::FixedBytes(20),
            EvmExpr::Ecrecover(_) => Type::Dialect(DialectType::Evm(EvmType::Address)),
            EvmExpr::Addmod(_) | EvmExpr::Mulmod(_) => Type::I256,
            EvmExpr::Gasleft(_) => Type::I256,
            EvmExpr::Blockhash(_) => Type::FixedBytes(32),
            EvmExpr::AbiEncode(_)
            | EvmExpr::AbiEncodePacked(_)
            | EvmExpr::AbiEncodeWithSelector(_)
            | EvmExpr::AbiEncodeWithSignature(_)
            | EvmExpr::AbiEncodeCall(_) => Type::Bytes,
            EvmExpr::AbiDecode(e) => {
                if e.types.len() == 1 {
                    e.types[0].clone()
                } else {
                    Type::Tuple(e.types.clone())
                }
            }
            EvmExpr::Transfer(_) => Type::None,
            EvmExpr::This(_) => Type::Dialect(DialectType::Evm(EvmType::Address)),
            EvmExpr::Super(_) => Type::None,
            EvmExpr::MsgData(_) => Type::Bytes,
            EvmExpr::MsgSig(_) => Type::FixedBytes(4),
            EvmExpr::BlockDifficulty(_) => Type::I256,
            EvmExpr::BlockGaslimit(_) => Type::I256,
            EvmExpr::BlockCoinbase(_) => Type::Dialect(DialectType::Evm(EvmType::Address)),
            EvmExpr::BlockChainid(_) => Type::I256,
            EvmExpr::BlockBasefee(_) => Type::I256,
        }
    }
}
