use meta::{DataLoc, Name};
use num_bigint::BigInt;
use std::fmt::{self, Display};

//-------------------------------------------------------------------------
// Data structures representing data types
//-------------------------------------------------------------------------

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum Type {
    Bool,
    Int(IntType),
    Fixed(FixedType),
    String(StringType),
    Address(AddressType),
    Bytes(BytesType),
    Array(ArrayType),
    Slice(SliceType),
    Struct(StructType),
    Enum(EnumType),
    Module(String),
    Tuple(TupleType),
    Func(FunctionType),
    Mapping(MappingType),
    Contract(ContractType),
    Magic(MagicType),
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct AddressType {
    pub payable: bool,
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct ArrayType {
    pub base: Box<Type>,
    pub length: Option<BigInt>,
    pub data_loc: DataLoc,
    pub is_ptr: bool,
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct BytesType {
    pub length: Option<u8>, // The length in `fixed-bytes` type. `None` when it is just `bytes`.
    pub data_loc: DataLoc,
    pub is_ptr: bool, // Whether the struct is a pointer type.``
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct ContractType {
    pub name: Name,
    pub is_lib: bool,
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct EnumType {
    pub name: Name,
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct FixedType {
    pub is_signed: bool, // Whether the fixed point type is signed or unsigned
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct FunctionType {
    pub params: Vec<Box<Type>>,
    pub returns: Vec<Box<Type>>,
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct IntType {
    pub bitwidth: Option<u16>, // None means arbitrary precision.
    pub is_signed: bool,       // Whether the integer type is signed or unsigned
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct MappingType {
    pub key: Box<Type>,
    pub value: Box<Type>,
    pub data_loc: DataLoc,
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct SliceType {
    pub base: Box<Type>,
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct StringType {
    pub data_loc: DataLoc,
    pub is_ptr: bool, // Whether it is a pointer type.
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct StructType {
    pub name: Name,
    pub data_loc: DataLoc,
    pub is_ptr: bool, // Whether it is a pointer type.
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct TupleType {
    pub elems: Vec<Option<Box<Type>>>,
}

// TODO: make it more general for many smart contract languages
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum MagicType {
    BlockType,
    MessageType,
    TxnType,
    ABIType,
    MetaType(Box<Type>),
}

//-------------------------------------------------------------------------
// Implementations for Type
//-------------------------------------------------------------------------

impl Type {
    pub fn is_contract_type(&self) -> bool {
        matches!(self, Type::Contract(_))
    }

    pub fn is_func_type(&self) -> bool {
        matches!(self, Type::Func(_))
    }

    pub fn is_contract_library_type(&self) -> bool {
        match self {
            Type::Contract(t) => t.is_lib,
            _ => false,
        }
    }

    pub fn is_magic_meta_type(&self) -> bool {
        matches!(self, Type::Magic(MagicType::MetaType(_)))
    }

    pub fn is_magic_contract_type(&self) -> bool {
        match self {
            Type::Magic(MagicType::MetaType(t)) => t.is_contract_type(),
            _ => false,
        }
    }

    pub fn is_magic_contract_library_type(&self) -> bool {
        match self {
            Type::Magic(MagicType::MetaType(t)) => t.is_contract_library_type(),
            _ => false,
        }
    }

    pub fn name(&self) -> Option<Name> {
        match self {
            Type::Struct(t) => Some(t.name.clone()),
            Type::Enum(t) => Some(t.name.clone()),
            Type::Contract(t) => Some(t.name.clone()),
            Type::Magic(MagicType::MetaType(t)) => t.name(),
            _ => None,
        }
    }

    pub fn update_name(&mut self, name: Name) {
        match self {
            Type::Struct(t) => t.name = name,
            Type::Enum(t) => t.name = name,
            Type::Contract(t) => t.name = name,
            Type::Magic(MagicType::MetaType(t)) => t.as_mut().update_name(name),
            _ => {}
        }
    }

    pub fn data_loc(&self) -> DataLoc {
        match self {
            Type::Array(t) => t.data_loc,
            Type::String(t) => t.data_loc,
            Type::Bytes(t) => t.data_loc,
            Type::Struct(t) => t.data_loc,
            Type::Mapping(t) => t.data_loc,
            _ => DataLoc::None,
        }
    }

    pub fn set_data_loc(&mut self, data_loc: DataLoc) {
        match self {
            Type::Array(t) => t.data_loc = data_loc,
            Type::String(t) => t.data_loc = data_loc,
            Type::Bytes(t) => t.data_loc = data_loc,
            Type::Struct(t) => t.data_loc = data_loc,
            Type::Mapping(t) => t.data_loc = data_loc,
            _ => {}
        }
    }
}

impl From<IntType> for Type {
    fn from(t: IntType) -> Self {
        Type::Int(t)
    }
}

impl From<FixedType> for Type {
    fn from(t: FixedType) -> Self {
        Type::Fixed(t)
    }
}

impl From<StringType> for Type {
    fn from(t: StringType) -> Self {
        Type::String(t)
    }
}

impl From<BytesType> for Type {
    fn from(t: BytesType) -> Self {
        Type::Bytes(t)
    }
}

impl From<AddressType> for Type {
    fn from(t: AddressType) -> Self {
        Type::Address(t)
    }
}

impl From<StructType> for Type {
    fn from(t: StructType) -> Self {
        Type::Struct(t)
    }
}

impl From<EnumType> for Type {
    fn from(t: EnumType) -> Self {
        Type::Enum(t)
    }
}

impl From<ContractType> for Type {
    fn from(t: ContractType) -> Self {
        Type::Contract(t)
    }
}

impl From<TupleType> for Type {
    fn from(t: TupleType) -> Self {
        Type::Tuple(t)
    }
}

impl From<MappingType> for Type {
    fn from(t: MappingType) -> Self {
        Type::Mapping(t)
    }
}

impl From<ArrayType> for Type {
    fn from(t: ArrayType) -> Self {
        Type::Array(t)
    }
}

impl From<SliceType> for Type {
    fn from(t: SliceType) -> Self {
        Type::Slice(t)
    }
}

impl From<FunctionType> for Type {
    fn from(t: FunctionType) -> Self {
        Type::Func(t)
    }
}

impl From<MagicType> for Type {
    fn from(t: MagicType) -> Self {
        Type::Magic(t)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Bool => write!(f, "bool"),
            Type::String(t) => write!(f, "{t}"),
            Type::Int(t) => write!(f, "{t}"),
            Type::Fixed(t) => write!(f, "{t}"),
            Type::Bytes(t) => write!(f, "{t}"),
            Type::Address(t) => write!(f, "{t}"),
            Type::Array(t) => write!(f, "{t}"),
            Type::Slice(t) => write!(f, "{t}"),
            Type::Struct(t) => write!(f, "{t}"),
            Type::Enum(t) => write!(f, "{t}"),
            Type::Module(t) => write!(f, "{t}"),
            Type::Contract(t) => write!(f, "{t}"),
            Type::Tuple(t) => write!(f, "{t}"),
            Type::Func(t) => write!(f, "{t}"),
            Type::Mapping(t) => write!(f, "{t}"),
            Type::Magic(t) => write!(f, "{t}"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Address type
//-------------------------------------------------------------------------

impl AddressType {
    pub fn new(payable: bool) -> Self {
        AddressType { payable }
    }

    pub fn serialize(&self) -> String {
        match &self.payable {
            false => "address".to_string(),
            true => "address_payable".to_string(),
        }
    }
}

impl Display for AddressType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.payable {
            false => write!(f, "address"),
            true => write!(f, "address payable"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Array type
//-------------------------------------------------------------------------

impl ArrayType {
    pub fn new(base: Type, length: Option<BigInt>, data_loc: DataLoc, is_ptr: bool) -> Self {
        ArrayType { base: Box::new(base), length, data_loc, is_ptr }
    }
}

impl Display for ArrayType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.length {
            None => write!(f, "{}[]", self.base).ok(),
            Some(len) => write!(f, "{}[{}]", self.base, len).ok(),
        };

        if self.data_loc != DataLoc::None {
            write!(f, " {}", self.data_loc).ok();
        }

        Ok(())
    }
}

//-------------------------------------------------------------------------
// Implementations for Bytes type
//-------------------------------------------------------------------------

impl BytesType {
    pub fn new(length: Option<u8>, data_loc: DataLoc, is_ptr: bool) -> Self {
        BytesType { length, data_loc, is_ptr }
    }

    pub fn serialize(&self) -> String {
        let res = match self.length {
            Some(num) => format!("bytes{num}"),
            None => "bytes".to_string(),
        };
        match self.data_loc {
            DataLoc::None => res,
            _ => format!("{res}_{}", self.data_loc),
        }
    }
}

impl Display for BytesType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.length {
            Some(num) => write!(f, "bytes{num}").ok(),
            None => write!(f, "bytes").ok(),
        };

        if self.data_loc != DataLoc::None {
            write!(f, " {}", self.data_loc).ok();
        }

        Ok(())
    }
}

//-------------------------------------------------------------------------
// Implementations for Contract type
//-------------------------------------------------------------------------

impl ContractType {
    pub fn new(name: Name, is_lib: bool) -> Self {
        ContractType { name, is_lib }
    }

    pub fn serialize(&self) -> String {
        format!("{}", self.name)
    }
}

impl Display for ContractType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

//-------------------------------------------------------------------------
// Implementations for Enum type
//-------------------------------------------------------------------------

impl EnumType {
    pub fn new(name: Name) -> Self {
        EnumType { name }
    }
}

impl Display for EnumType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

//-------------------------------------------------------------------------
// Implementations for Fixed type
//-------------------------------------------------------------------------

impl FixedType {
    pub fn new(signed: bool) -> Self {
        FixedType { is_signed: signed }
    }

    pub fn serialize(&self) -> String {
        match self.is_signed {
            true => "fixed".to_string(),
            false => "ufixed".to_string(),
        }
    }
}

impl Display for FixedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.is_signed {
            true => write!(f, "fixed"),
            false => write!(f, "ufixed"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Function type
//-------------------------------------------------------------------------

impl FunctionType {
    pub fn new(params: Vec<Type>, returns: Vec<Type>) -> Self {
        let params = params.into_iter().map(Box::new).collect();
        let returns = returns.into_iter().map(Box::new).collect();
        Self { params, returns }
    }
}

impl Display for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params = self
            .params
            .iter()
            .map(|t| format!("{t}"))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "function ({params})").ok();

        let returns = self
            .returns
            .iter()
            .map(|t| format!("{t}"))
            .collect::<Vec<_>>()
            .join(", ");
        if !self.returns.is_empty() {
            write!(f, " returns ({returns})").ok();
        }

        Ok(())
    }
}

//-------------------------------------------------------------------------
// Implementations for Int type
//-------------------------------------------------------------------------

impl IntType {
    pub fn new(bitwidth: Option<u16>, signed: bool) -> Self {
        IntType { bitwidth, is_signed: signed }
    }

    pub fn serialize(&self) -> String {
        let bw = match &self.bitwidth {
            None => "".to_string(),
            Some(bw) => format!("{bw}"),
        };
        match self.is_signed {
            true => format!("int{bw}"),
            false => format!("uint{bw}"),
        }
    }
}

impl Display for IntType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bw = match &self.bitwidth {
            None => "".to_string(),
            Some(bw) => format!("{bw}"),
        };
        match self.is_signed {
            true => write!(f, "int{bw}"),
            false => write!(f, "uint{bw}"),
        }
    }
}

//-------------------------------------------------------------------------
// Magic type
//-------------------------------------------------------------------------

impl MagicType {
    pub fn new_block_type() -> Self {
        MagicType::BlockType
    }

    pub fn new_message_type() -> Self {
        MagicType::MessageType
    }

    pub fn new_transaction_type() -> Self {
        MagicType::TxnType
    }

    pub fn new_abi_type() -> Self {
        MagicType::ABIType
    }

    pub fn new_meta_type(base_type: Type) -> Self {
        MagicType::MetaType(Box::new(base_type))
    }
}

impl Display for MagicType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MagicType::BlockType => write!(f, "block"),
            MagicType::MessageType => write!(f, "msg"),
            MagicType::TxnType => write!(f, "tx"),
            MagicType::ABIType => write!(f, "abi"),
            MagicType::MetaType(typ) => write!(f, "type({typ})"),
        }
    }
}

//-------------------------------------------------------------------------
// Implementations for Mapping type
//-------------------------------------------------------------------------

impl MappingType {
    pub fn new(key: Type, value: Type, data_loc: DataLoc) -> Self {
        MappingType { key: Box::new(key), value: Box::new(value), data_loc }
    }
}

impl Display for MappingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mapping({} => {})", self.key, self.value).ok();

        if self.data_loc != DataLoc::None {
            write!(f, " {}", self.data_loc).ok();
        }

        Ok(())
    }
}

//-------------------------------------------------------------------------
// Slice type
//-------------------------------------------------------------------------

impl SliceType {
    pub fn new(base: Type) -> Self {
        SliceType { base: Box::new(base) }
    }

    pub fn serialize(&self) -> String {
        format!("{}", self.base)
    }
}

impl Display for SliceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.base)
    }
}

//-------------------------------------------------------------------------
// Implementations for String type
//-------------------------------------------------------------------------

impl StringType {
    pub fn new(data_loc: DataLoc, is_ptr: bool) -> Self {
        StringType { data_loc, is_ptr }
    }
}

impl Display for StringType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "string").ok();

        if self.data_loc != DataLoc::None {
            write!(f, " {}", self.data_loc).ok();
        }

        Ok(())
    }
}

//-------------------------------------------------------------------------
// Implementations for Struct type
//-------------------------------------------------------------------------

impl StructType {
    pub fn new(name: Name, data_loc: DataLoc, is_ptr: bool) -> Self {
        StructType { name, data_loc, is_ptr }
    }
}

impl Display for StructType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name).ok();

        if self.data_loc != DataLoc::None {
            write!(f, " {}", self.data_loc).ok();
        }

        Ok(())
    }
}

//-------------------------------------------------------------------------
// Implementations for Tuple type
//-------------------------------------------------------------------------

impl TupleType {
    pub fn new(elems: Vec<Option<Box<Type>>>) -> Self {
        TupleType { elems }
    }
}

impl Display for TupleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let elems = self
            .elems
            .iter()
            .map(|e| match e {
                None => "".to_string(),
                Some(typ) => typ.to_string(),
            })
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "tuple({elems})")
    }
}
