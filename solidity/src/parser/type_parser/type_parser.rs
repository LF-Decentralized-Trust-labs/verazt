//! Module for parsing data type.

use crate::ast::*;
use crate::ast::{DataLoc, Name};
use extlib::{error::Result, fail};
use num_bigint::BigInt;
use pest::{
    Parser,
    iterators::{Pair, Pairs},
};
use pest_derive::Parser;

struct ArrayDim {
    pub length: Option<BigInt>,
    pub data_loc: DataLoc,
    pub is_ptr: bool,
}

impl ArrayDim {
    pub fn new(length: Option<BigInt>, dloc: DataLoc, is_ptr: bool) -> Self {
        ArrayDim { length, data_loc: dloc, is_ptr }
    }
}

/// Data structure representing a parse tree a data type.
///
/// This data structure is automatically derived by [`Pest`] parser.
#[derive(Parser)]
#[grammar = "parser/type_parser/type_grammar.pest"]
struct TypeParser;

impl TypeParser {
    //-------------------------------------------
    // Parsing data type stream.
    //-------------------------------------------

    /// Parse an input string stream of a data type.
    fn parse_data_type_stream(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Data type stream not found", pair),
        };
        match p.as_rule() {
            Rule::data_type => Self::parse_data_type(p),
            _ => error("Failed to parse data type", p),
        }
    }

    //-------------------------------------------
    // Parsing data type
    //-------------------------------------------

    fn parse_data_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Failed to parse data type", pair),
        };
        match p.as_rule() {
            Rule::slice_type => Self::parse_slice_type(p),
            Rule::array_type => Self::parse_array_type(p),
            Rule::non_array_type => Self::parse_non_array_type(p),
            _ => error("Need to parse data type", pair),
        }
    }

    //-------------------------------------------
    // Parsing non-array type
    //-------------------------------------------

    fn parse_non_array_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Failed to parse data type", pair),
        };
        match p.as_rule() {
            Rule::function_type => Self::parse_function_type(p),
            Rule::mapping_type => Self::parse_mapping_type(p),
            Rule::struct_type => Self::parse_struct_type(p),
            Rule::elementary_type => Self::parse_elementary_type(p),
            Rule::tuple_type => Self::parse_tuple_type(p),
            Rule::magic_type => Self::parse_magic_type(p),
            Rule::type_name => Self::parse_type_name(p),
            _ => error("Need to parse data type", pair),
        }
    }

    //-------------------------------------------
    // Parsing elementary type
    //-------------------------------------------

    fn parse_elementary_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Failed to parse data type", pair),
        };
        match p.as_rule() {
            Rule::int_type => Self::parse_int_type(p),
            Rule::uint_type => Self::parse_uint_type(p),
            Rule::rational_const_type => Self::parse_rational_const_type(p),
            Rule::bool_type => Ok(Type::Bool),
            Rule::enum_type => Self::parse_enum_type(p),
            Rule::module_type => Self::parse_module_type(p),
            Rule::address_type => Self::parse_address_type(p),
            Rule::bytes_type => Self::parse_bytes_type(p),
            Rule::string_type => Self::parse_string_type(p),
            Rule::contract_type => Self::parse_contract_type(p),
            _ => error("parse_elementary_type err", pair),
        }
    }

    //-------------------------------------------
    // Parsing tuple type
    //-------------------------------------------

    fn parse_tuple_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let elem_types = match inner_pairs.peek() {
            None => vec![],
            Some(p) => match p.as_rule() {
                Rule::tuple_param_types => {
                    inner_pairs.next();
                    Self::parse_tuple_param_types(p)?
                }
                _ => vec![],
            },
        };
        Ok(TupleType::new(elem_types).into())
    }

    fn parse_tuple_param_types(pair: Pair<Rule>) -> Result<Vec<Option<Box<Type>>>> {
        let mut inner_pairs = pair.clone().into_inner();
        if inner_pairs.peek().is_none() {
            return Ok(vec![]);
        }
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Failed to parse tuple param types", pair),
        };
        let fst_param = match p.as_rule() {
            Rule::tuple_param_type => Some(Box::new(Self::parse_tuple_parameter_type(p)?)),
            _ => None,
        };
        let mut other_params = match inner_pairs.peek() {
            Some(rule) => {
                inner_pairs.next();
                Self::parse_tuple_other_params_types(rule)?
            }
            None => vec![],
        };
        let mut params = vec![fst_param];
        params.append(&mut other_params);
        Ok(params)
    }

    fn parse_tuple_other_params_types(pair: Pair<Rule>) -> Result<Vec<Option<Box<Type>>>> {
        let mut inner_pairs = pair.clone().into_inner();
        if inner_pairs.peek().is_none() {
            return Ok(vec![]);
        }
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("parse_tuple_param_type err", pair),
        };
        let fst_param = match p.as_rule() {
            Rule::tuple_param_type => Some(Box::new(Self::parse_tuple_parameter_type(p)?)),
            _ => None,
        };
        let mut other_params = match inner_pairs.peek() {
            Some(p) => {
                inner_pairs.next();
                Self::parse_tuple_other_params_types(p)?
            }
            None => vec![],
        };
        let mut params = vec![fst_param];
        params.append(&mut other_params);
        Ok(params)
    }

    fn parse_tuple_parameter_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("parse_tuple_parameter_type failed", pair),
        };
        match p.as_rule() {
            Rule::function_type => Self::parse_function_type(p),
            Rule::mapping_type => Self::parse_mapping_type(p),
            Rule::array_type => Self::parse_array_type(p),
            Rule::struct_type => Self::parse_struct_type(p),
            Rule::tuple_type => Self::parse_tuple_type(p),
            Rule::magic_type => Self::parse_magic_type(p),
            Rule::elementary_type => Self::parse_elementary_type(p),
            Rule::type_name => Self::parse_type_name(p),
            _ => error("parse_tuple_parameter_type failed", pair),
        }
    }

    //-------------------------------------------
    // Parsing mapping type
    //-------------------------------------------

    fn parse_mapping_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let key = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Mapping key not found", pair),
        };
        let value = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Mapping value not found:", pair),
        };
        let key_typ = match key.as_rule() {
            Rule::non_array_type => Self::parse_non_array_type(key),
            Rule::array_type => Self::parse_array_type(key),
            _ => return error("Failed to parse mapping key type", key),
        };
        let value_typ = match value.as_rule() {
            Rule::non_array_type => Self::parse_non_array_type(value),
            Rule::array_type => Self::parse_array_type(value),
            _ => return error("Failed to parse mapping value type", value),
        };
        let dloc = Self::peek_data_loc(&mut inner_pairs)?;
        match (key_typ, value_typ) {
            (Ok(key), Ok(value)) => Ok(MappingType::new(key, value, dloc).into()),
            _ => error("Failed to parse mapping type", pair),
        }
    }

    //-------------------------------------------
    // Parsing array type
    //-------------------------------------------

    fn parse_array_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let typ = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Failed to parse array type", pair),
        };
        let mut base = match typ.as_rule() {
            Rule::non_array_type => Self::parse_non_array_type(typ)?,
            _ => return error("Failed to parse array base type", typ),
        };
        let dims_rule = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Array dimension not found", pair),
        };
        let dims = match dims_rule.as_rule() {
            Rule::array_dimension => Self::parse_dimensions(dims_rule)?,
            _ => return error("Array dimension not found", pair),
        };
        for dim in dims {
            base = ArrayType::new(base, dim.length, dim.data_loc, dim.is_ptr).into();
        }
        Ok(base)
    }

    fn parse_dimensions(pair: Pair<Rule>) -> Result<Vec<ArrayDim>> {
        let mut inner_pairs = pair.into_inner();
        let length = match inner_pairs.peek() {
            None => None,
            Some(p_length) => match p_length.as_str().parse() {
                Ok(length) => {
                    inner_pairs.next();
                    Some(length)
                }
                Err(_) => None,
            },
        };
        let dloc = Self::peek_data_loc(&mut inner_pairs)?;
        let is_ptr = match Self::peek_pointer_info(&inner_pairs) {
            Some(is_ptr) => {
                inner_pairs.next();
                is_ptr
            }
            None => false,
        };
        let dim = ArrayDim::new(length, dloc, is_ptr);
        match inner_pairs.next() {
            Some(p) => match p.as_rule() {
                Rule::array_dimension => {
                    let mut others = Self::parse_dimensions(p)?;
                    let mut dims = vec![dim];
                    dims.append(&mut others);
                    Ok(dims)
                }
                _ => error("Failed to parse dimension", p),
            },
            None => Ok(vec![dim]),
        }
    }

    //-------------------------------------------
    // Parsing slice type
    //-------------------------------------------

    fn parse_slice_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let typ = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Failed to parse mapping type", pair),
        };
        let base = match typ.as_rule() {
            Rule::array_type => Self::parse_array_type(typ)?,
            Rule::non_array_type => Self::parse_non_array_type(typ)?,
            _ => return error("Failed to parse array type", pair),
        };
        Ok(SliceType::new(base).into())
    }

    //-------------------------------------------
    // Parsing function type
    //-------------------------------------------

    fn parse_function_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        if let Some(p) = inner_pairs.peek() {
            if let Rule::function_name = p.as_rule() {
                inner_pairs.next();
            }
        }
        let params = match inner_pairs.peek() {
            None => vec![],
            Some(p) => match p.as_rule() {
                Rule::function_param_types => {
                    inner_pairs.next();
                    Self::parse_function_param_types(p)?
                }
                _ => vec![],
            },
        };
        let fmut = match Self::peek_function_mutability(&inner_pairs) {
            Ok(Some(fmut)) => {
                // Advance the token pari if peeking successfully
                inner_pairs.next();
                fmut
            }
            _ => FuncMut::None,
        };
        let fvis = match inner_pairs.peek() {
            Some(p) => match p.as_rule() {
                Rule::function_visibility => {
                    // Advance the token if peeking successfully
                    inner_pairs.next();
                    Self::parse_function_visibility(p)?
                }
                _ => FuncVis::None,
            },
            None => FuncVis::None,
        };
        let returns = match inner_pairs.peek() {
            Some(p) => match p.as_rule() {
                Rule::function_return_types => {
                    inner_pairs.next();
                    Self::parse_function_return_type(p)?
                }
                _ => return error("Unknown function return type", p),
            },
            None => vec![],
        };
        Ok(FuncType::new(params, returns, fvis, fmut).into())
    }

    fn parse_function_other_params_types(pair: Pair<Rule>) -> Result<Vec<Type>> {
        let mut inner_pairs = pair.clone().into_inner();
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Func param type not found", pair),
        };
        let fst_param = match p.as_rule() {
            Rule::function_param_type => Self::parse_function_parameter_type(p)?,
            _ => return error("Failed to parse func param type", p),
        };
        let mut other_params = match inner_pairs.peek() {
            Some(p) => {
                inner_pairs.next();
                Self::parse_function_other_params_types(p)?
            }
            None => vec![],
        };
        let mut params = vec![fst_param];
        params.append(&mut other_params);
        Ok(params)
    }

    fn parse_function_parameter_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Func param type not found", pair),
        };
        match p.as_rule() {
            Rule::function_type => Self::parse_function_type(p),
            Rule::mapping_type => Self::parse_mapping_type(p),
            Rule::array_type => Self::parse_array_type(p),
            Rule::struct_type => Self::parse_struct_type(p),
            Rule::tuple_type => Self::parse_tuple_type(p),
            Rule::magic_type => Self::parse_magic_type(p),
            Rule::elementary_type => Self::parse_elementary_type(p),
            Rule::type_name => Self::parse_type_name(p),
            _ => error("Failed to parse func param type", p),
        }
    }

    fn parse_function_param_types(pair: Pair<Rule>) -> Result<Vec<Type>> {
        let mut inner_pairs = pair.clone().into_inner();
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Func params types not found", pair),
        };
        let fst_param = match p.as_rule() {
            Rule::function_param_type => Self::parse_function_parameter_type(p)?,
            _ => return error("Failed to parse func params types", p),
        };
        let mut other_params = match inner_pairs.peek() {
            Some(rule) => {
                inner_pairs.next();
                Self::parse_function_other_params_types(rule)?
            }
            None => vec![],
        };
        let mut params = vec![fst_param];
        params.append(&mut other_params);
        Ok(params)
    }

    fn parse_function_return_type(pair: Pair<Rule>) -> Result<Vec<Type>> {
        let mut inner_pairs = pair.clone().into_inner();
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Failed to parse func_parameters_type", pair),
        };
        match p.as_rule() {
            Rule::function_param_types => Self::parse_function_param_types(p),
            _ => error("Failed to parse func_parameters_type", p),
        }
    }

    fn peek_function_mutability(pairs: &Pairs<Rule>) -> Result<Option<FuncMut>> {
        match pairs.peek() {
            Some(p) => match p.as_rule() {
                Rule::function_mutability => match p.as_str() {
                    "constant" => Ok(Some(FuncMut::Constant)),
                    "payable" => Ok(Some(FuncMut::Payable)),
                    "pure" => Ok(Some(FuncMut::Pure)),
                    "view" => Ok(Some(FuncMut::View)),
                    _ => error("Unknown function mutability", p),
                },
                _ => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn parse_function_visibility(pair: Pair<Rule>) -> Result<FuncVis> {
        let mut inner_pairs = pair.clone().into_inner();
        match inner_pairs.next() {
            Some(p) => match p.as_str() {
                "internal" => Ok(FuncVis::Internal),
                "external" => Ok(FuncVis::External),
                "private" => Ok(FuncVis::Private),
                "public" => Ok(FuncVis::Public),
                _ => error("Unknown visibility", p),
            },
            None => error("Failed to parse function visibility", pair),
        }
    }

    //-------------------------------------------
    // Parsing struct type
    //-------------------------------------------

    fn parse_struct_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Failed to parse struct type", pair),
        };
        let (scope, name) = match p.as_rule() {
            Rule::struct_name => {
                let mut p_inners = p.clone().into_inner();
                let mut scope = None;
                // Parse contract scope name, e.g., `Contract.StructName`, if existing.
                if p_inners.len() > 1 {
                    scope = match p_inners.next() {
                        Some(p_scope) => match p_scope.clone().into_inner().next() {
                            Some(scope) => Some(Name::from(scope.as_str())),
                            None => return error("Struct scope not found", p_scope),
                        },
                        None => return error("Struct scope not found", p),
                    };
                }
                // Parse struct name
                let name: Name = match p_inners.next() {
                    Some(p_name) => p_name.as_str().into(),
                    None => return error("Struct name not found", p),
                };
                (scope, name)
            }
            _ => return error("Failed to parse struct name", p),
        };
        let dloc = Self::peek_data_loc(&mut inner_pairs)?;
        let is_ptr = match Self::peek_pointer_info(&inner_pairs) {
            Some(is_ptr) => {
                inner_pairs.next();
                is_ptr
            }
            None => false,
        };

        Ok(StructType::new(name, scope, dloc, is_ptr).into())
    }

    //-------------------------------------------
    // Parsing enum type
    //-------------------------------------------

    fn parse_enum_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Failed to parse enum type", pair),
        };
        let (scope, name) = match p.as_rule() {
            Rule::enum_name => {
                let mut p_inners = p.clone().into_inner();
                let mut scope: Option<Name> = None;
                // Parse contract scope name, e.g., `Contract.EnumName`, if existing.
                if p_inners.len() > 1 {
                    scope = match p_inners.next() {
                        Some(p_scope) => match p_scope.clone().into_inner().next() {
                            Some(scope) => Some(Name::from(scope.as_str())),
                            None => return error("Enum scope name not found", p_scope),
                        },
                        None => return error("Enum scope name not found", p),
                    };
                }
                // Parse enum name
                let name: Name = match p_inners.next() {
                    Some(p_name) => p_name.as_str().into(),
                    None => return error("Enum name not found", p),
                };
                (scope, name)
            }
            _ => return error("Failed to parse enum name", p),
        };
        Ok(EnumType::new(name, scope).into())
    }

    //-------------------------------------------
    // Parsing type names
    //-------------------------------------------

    fn parse_type_name(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let mut scope: Option<Name> = None;
        // Parse contract scope name, e.g., `Contract.TypeName`, if existing.
        if inner_pairs.len() > 1 {
            scope = match inner_pairs.next() {
                Some(p_scope) => match p_scope.clone().into_inner().next() {
                    Some(scope) => Some(Name::from(scope.as_str())),
                    None => return error("Type name scope not found", p_scope),
                },
                None => return error("Type name scope not found", pair),
            };
        }
        // Parse type name
        let type_name: Name = match inner_pairs.next() {
            Some(p_name) => p_name.as_str().into(),
            None => return error("Enum name not found", pair),
        };
        Ok(UserDefinedType::new(type_name, scope).into())
    }

    //-------------------------------------------
    // Parsing module type
    //-------------------------------------------

    fn parse_module_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Failed to parse module type", pair),
        };
        let name = match p.as_rule() {
            Rule::module_name => p.as_str().to_string(),
            _ => return error("Failed to parse module name", p),
        };
        Ok(Type::Module(name))
    }

    //-------------------------------------------
    // Parsing contract types
    //-------------------------------------------

    fn parse_contract_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let is_contract_typ = match inner_pairs.next() {
            Some(p) => match p.as_str() {
                "contract" => true,
                "library" => false,
                _ => return error("Unknown contract type", p),
            },
            None => return error("Failed to parse contract type", pair),
        };
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Failed to parse contract type", pair),
        };
        let name: Name = match p.as_rule() {
            Rule::contract_name => {
                let mut p_inners = p.clone().into_inner();
                match p_inners.next() {
                    Some(p_name) => p_name.as_str().to_string().into(),
                    None => return error("Failed to parse contract name", p),
                }
            }
            _ => return error("Failed to parse contract name", p),
        };
        Ok(ContractType::new(name, !is_contract_typ, None).into())
    }

    //-------------------------------------------
    // Parsing magic type
    //-------------------------------------------

    fn parse_magic_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Failed to parse magic type", pair),
        };
        let typ = match p.as_rule() {
            Rule::block_type => MagicType::new_block_type(),
            Rule::message_type => MagicType::new_message_type(),
            Rule::transaction_type => MagicType::new_transaction_type(),
            Rule::abi_type => MagicType::new_abi_type(),
            Rule::meta_type => MagicType::new_meta_type(Self::parse_meta_type(p)?),
            _ => return error("Need to parse magic type", p),
        };
        Ok(typ.into())
    }

    fn parse_meta_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Failed to parse meta type", pair),
        };
        match p.as_rule() {
            Rule::array_type => Self::parse_array_type(p),
            Rule::non_array_type => Self::parse_non_array_type(p),
            _ => error("Failed to parse meta type", p),
        }
    }

    //-------------------------------------------
    // Parsing pointer type
    //-------------------------------------------

    /// Peek data location from a list of pairs. This action will not move the
    /// token pair iterator.
    fn peek_pointer_info(pairs: &Pairs<Rule>) -> Option<bool> {
        match pairs.peek() {
            None => None,
            Some(p2) => match p2.as_rule() {
                Rule::pointer_type => {
                    let is_ptr = matches!(p2.as_str(), "pointer");
                    Some(is_ptr)
                }
                _ => None,
            },
        }
    }

    //-------------------------------------------
    // Parsing integer types
    //-------------------------------------------

    fn parse_int_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Failed to parse int type", pair),
        };
        match p.as_rule() {
            Rule::int_var_type => {
                let mut p_inners = p.into_inner();
                let bitwidth: u16 = match p_inners.next() {
                    None => 256, // default bitwidth
                    Some(p_bw) => match p_bw.as_str().parse() {
                        Ok(bw) => bw,
                        Err(_) => return error("Invalid bitwith", p_bw),
                    },
                };
                Ok(IntType::new(Some(bitwidth), true).into())
            }
            Rule::int_const_type => Ok(IntType::new(None, true).into()),
            _ => error("Failed to parse int type", p),
        }
    }

    //-------------------------------------------
    // Parsing address types
    //-------------------------------------------

    fn parse_address_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        match inner_pairs.next() {
            Some(_) => Ok(AddressType::new(true).into()),
            None => Ok(AddressType::new(false).into()),
        }
    }

    fn parse_uint_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        let p = match inner_pairs.next() {
            Some(p) => p,
            None => return error("Failed to parse uint type", pair),
        };
        match p.as_rule() {
            Rule::uint_var_type => {
                let mut p_inners = p.into_inner();
                let bitwidth: u16 = match p_inners.next() {
                    None => 256, // default bitwidth
                    Some(p_bw) => match p_bw.as_str().parse() {
                        Ok(bw) => bw,
                        Err(_) => return error("Invalid bitwith", p_bw),
                    },
                };
                Ok(IntType::new(Some(bitwidth), false).into())
            }
            Rule::uint_const_type => Ok(IntType::new(None, false).into()),
            _ => error("Failed to parse uint type", p),
        }
    }

    //-------------------------------------------
    // Parsing string type
    //-------------------------------------------

    fn parse_string_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inners: Pairs<Rule> = pair.clone().into_inner();
        let p = match inners.next() {
            Some(p) => p,
            None => return error("Failed to parse string type", pair),
        };
        match p.as_rule() {
            Rule::string_var_type => {
                let mut inners2: Pairs<Rule> = p.into_inner();
                let dloc = Self::peek_data_loc(&mut inners2)?;
                let is_ptr = match Self::peek_pointer_info(&inners2) {
                    Some(is_ptr) => {
                        inners2.next(); // Advance the token pair if peeking successfully
                        is_ptr
                    }
                    None => false,
                };
                Ok(StringType::new(dloc, is_ptr).into())
            }
            Rule::string_const_type => Ok(StringType::new(DataLoc::None, false).into()),
            _ => error("Failed to parse string type", p),
        }
    }

    //-------------------------------------------
    // Parsing bytes type
    //-------------------------------------------

    fn parse_bytes_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();

        let length = match inner_pairs.next() {
            Some(pair_len) => match pair_len.as_str().parse::<u8>() {
                Ok(len) => {
                    inner_pairs.next();
                    Some(len)
                }
                Err(_) => None,
            },
            None => None,
        };
        let data_loc = Self::peek_data_loc(&mut inner_pairs)?;
        let is_ptr = match Self::peek_pointer_info(&inner_pairs) {
            Some(is_pointer) => {
                inner_pairs.next();
                is_pointer
            }
            None => false,
        };

        Ok(BytesType::new(length, data_loc, is_ptr).into())
    }

    //-------------------------------------------
    // Parsing rational constant types
    //-------------------------------------------

    fn parse_rational_const_type(pair: Pair<Rule>) -> Result<Type> {
        let mut inner_pairs = pair.clone().into_inner();
        match inner_pairs.next() {
            Some(_) => Ok(FixedType::new(true).into()),
            None => error("parse_rational_const_type failed", pair),
        }
    }

    //-------------------------------------------
    // Parsing data location
    //-------------------------------------------

    /// Peek a data location at the beginning of a list of pairs. This action
    /// will not move the token pair iterator.
    fn peek_data_loc(pairs: &mut Pairs<Rule>) -> Result<DataLoc> {
        let dloc = match pairs.peek() {
            None => DataLoc::None,
            Some(p) => match p.as_rule() {
                Rule::data_location => match p.as_str() {
                    "storage" => DataLoc::Storage,
                    "memory" => DataLoc::Memory,
                    "calldata" => DataLoc::Calldata,
                    "default" => DataLoc::None,
                    _ => return error("Unknown data location", p),
                },
                _ => DataLoc::None,
            },
        };
        if dloc != DataLoc::None {
            pairs.next(); // Advance the token pair if peeking successfully
        }
        Ok(dloc)
    }
}

fn error<T>(msg: &str, token: Pair<Rule>) -> Result<T> {
    fail!("Type parser: {}: {}\nToken: {}", msg, token.as_str(), token)
}

pub fn parse_data_type(type_str: &str) -> Result<Type> {
    // debug!("== PARSE DATA TYPE: {}", type_string);
    let mut pairs = match TypeParser::parse(Rule::data_type_stream, type_str) {
        Ok(pairs) => pairs,
        Err(err) => fail!("Error while parsing: {}\n\nError log: {}", type_str, err),
    };

    // Skip `SOI` token of Pest
    match pairs.next() {
        Some(next_pairs) => TypeParser::parse_data_type_stream(next_pairs),
        None => fail!("Error while parsing: {}\n\nPairs: {}", type_str, pairs),
    }
}
