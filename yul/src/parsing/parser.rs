//! Module for parsing Yul intermediate code.

use crate::{
    ast::*,
    parsing::keywords::{YUL_KEYWORDS, YUL_RESERVED_NAMES},
};
use Either::{Left, Right};
use color_eyre::eyre::{Result, bail};
use either::Either;
use pest::{Parser, iterators::Pair};
use pest_derive::Parser;
use regex::Regex;
use std::fs;

/// Data structure representing a parse tree of Yul intermediate code.
///
/// This data structure is automatically derived by [`Pest`] parser.
#[derive(Parser)]
#[grammar = "parsing/yul_grammar.pest"]
struct YulParser;

impl YulParser {
    //-------------------------------------------------
    // Source unit
    //-------------------------------------------------

    fn parse_source_unit(pair: Pair<Rule>) -> Result<SourceUnit> {
        assert_eq!(pair.as_rule(), Rule::source_unit);

        let mut objects: Vec<Object> = vec![];
        for p in pair.into_inner() {
            let l = Self::parse_location(&p);
            match p.as_rule() {
                Rule::object => objects.push(Self::parse_object(p)?),
                Rule::COMMENT => {
                    let _comment = Self::parse_comment(p);
                }
                Rule::EOI => {}
                r => bail!("Parsing rule: {:?}.\n\nLocation: {}", r, l),
            }
        }

        if objects.is_empty() {
            bail!("Empty source unit!")
        } else if objects.len() > 1 {
            bail!("Source unit cannot accept more than 1 outermost object!")
        } else {
            Ok(SourceUnit::new(objects[0].clone()))
        }
    }

    //-------------------------------------------------
    // Object
    //-------------------------------------------------

    fn parse_object(pair: Pair<Rule>) -> Result<Object> {
        assert_eq!(pair.as_rule(), Rule::object);

        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.into_inner();

        // Get object name
        let name = match pair_iter.next() {
            Some(p) => p.as_str().to_string(),
            None => {
                bail!("Object name not found!\n\nLocation: {}", loc)
            }
        };

        // Get object code
        let code = match pair_iter.next() {
            Some(p) => Self::parse_code(p)?,
            None => {
                bail!("Object code not found!\n\nLocation: {}", loc)
            }
        };
        let mut children: Vec<Either<Object, Data>> = vec![];
        for p in pair_iter {
            let l = Self::parse_location(&p);
            match p.as_rule() {
                Rule::object => children.push(Left(Self::parse_object(p)?)),
                Rule::data => children.push(Right(Self::parse_data(p)?)),
                Rule::COMMENT => {}
                r => bail!("Parsing rule: {:?}.\n\nLocation: {}", r, l),
            }
        }

        Ok(Object::new(name, code, children))
    }

    fn parse_code(pair: Pair<Rule>) -> Result<Code> {
        assert_eq!(pair.as_rule(), Rule::code);

        let loc = Self::parse_location(&pair);
        let pair_iter = pair.into_inner();

        let mut blocks: Vec<Block> = vec![];
        for p in pair_iter {
            let l = Self::parse_location(&p);
            match p.as_rule() {
                Rule::block => blocks.push(Self::parse_block(p)?),
                Rule::COMMENT => {}
                r => {
                    bail!("Parsing rule: {:?}\n\nLocation{}", r, l)
                }
            }
        }

        if blocks.is_empty() {
            bail!("Code block not found!\n\nLocation: {}", loc);
        } else if blocks.len() > 1 {
            bail!("Code section must contain only 1 block!\n\nLocation:{}", loc);
        } else {
            Ok(Code::new(blocks[0].clone()))
        }
    }

    fn parse_data(pair: Pair<Rule>) -> Result<Data> {
        assert_eq!(pair.as_rule(), Rule::data);

        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.into_inner();

        let name = match pair_iter.next() {
            Some(p) => StringLit::new(p.as_str()),
            None => bail!("Data not found!\n\nLocation: {}", loc),
        };

        match pair_iter.next() {
            Some(p) => {
                let l = Self::parse_location(&p);
                let content = match p.as_rule() {
                    Rule::hex_literal => Left(HexLit::new(p.as_str())),
                    Rule::string_literal => Right(StringLit::new(p.as_str())),
                    _ => bail!("Parsing data: {}\n\nLocation: {}", p, l),
                };
                Ok(Data::new(name, content))
            }
            None => bail!("Data not found!\n\nLocation: {}", loc),
        }
    }

    //-------------------------------------------------
    // Block
    //-------------------------------------------------

    fn parse_block(pair: Pair<Rule>) -> Result<Block> {
        assert_eq!(pair.as_rule(), Rule::block);

        let mut stmts: Vec<Stmt> = vec![];
        for p in pair.into_inner() {
            let l = Self::parse_location(&p);
            match p.as_rule() {
                Rule::statement => stmts.push(Self::parse_stmt(p)?),
                Rule::COMMENT => {}
                r => bail!("Parsing rule: {:?}\n\nLocation: {}", r, l),
            }
        }

        Ok(Block::new(stmts))
    }

    //-------------------------------------------------
    // Definitions
    //-------------------------------------------------

    fn parse_var_decl(pair: Pair<Rule>) -> Result<VarDecl> {
        assert_eq!(pair.as_rule(), Rule::variable_declaration);

        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.into_inner();

        // Parse the LHS
        let vars = match pair_iter.next() {
            Some(p) => Self::parse_typed_ident_list(p)?,
            None => bail!("Declared variables not found!\n\nLocation: {}", loc),
        };

        // Parser the RHS
        let value = match pair_iter.next() {
            Some(p) => Some(Self::parse_expr(p)?),
            None => None,
        };

        Ok(VarDecl::new(vars, value))
    }

    fn parse_func_def(pair: Pair<Rule>) -> Result<FuncDef> {
        assert_eq!(pair.as_rule(), Rule::function_definition);

        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.into_inner();

        // Parse function name
        let name = match pair_iter.next() {
            Some(p) => p.as_str(),
            None => {
                bail!("Function name not found!\n\nLocation: {}", loc)
            }
        };

        if YUL_KEYWORDS.contains(&name) {
            bail!("Function name is a keyword: {}!\n\nLocation: {}", name, loc);
        }

        // Parse function parameters, returned values, and body
        let mut parameters = vec![];
        let mut returns = vec![];

        let p1 = match pair_iter.next() {
            Some(p) => p,
            None => {
                bail!("Function parameters or body not found!\n\nLocation: {}", loc)
            }
        };
        let body = match p1.as_rule() {
            Rule::typed_identifier_list => {
                parameters = Self::parse_typed_ident_list(p1)?;
                let p2 = match pair_iter.next() {
                    Some(p) => p,
                    None => {
                        bail!("Function returns/body not found!\n\nLocation: {}", loc)
                    }
                };
                match p2.as_rule() {
                    Rule::RETURN_SEPARATOR => {
                        returns = match pair_iter.next() {
                            Some(p) => Self::parse_typed_ident_list(p)?,
                            None => {
                                bail!("Function returns not found!\n\nLocation: {}", loc)
                            }
                        };

                        match pair_iter.next() {
                            Some(p) => Self::parse_block(p)?,
                            None => {
                                bail!("Function body not found!\n\nLocation: {}", loc)
                            }
                        }
                    }
                    _ => Self::parse_block(p2)?,
                }
            }

            Rule::RETURN_SEPARATOR => {
                returns = match pair_iter.next() {
                    Some(p) => Self::parse_typed_ident_list(p)?,
                    None => bail!("Function returns not found!\n\nLocation: {}", loc),
                };

                match pair_iter.next() {
                    Some(p) => Self::parse_block(p)?,
                    None => {
                        bail!("Function body not found!\n\nLocation: {}", loc)
                    }
                }
            }

            _ => Self::parse_block(p1)?,
        };

        Ok(FuncDef::new(name, parameters, returns, body))
    }

    //-------------------------------------------------
    // Statements
    //-------------------------------------------------

    fn parse_stmt(pair: Pair<Rule>) -> Result<Stmt> {
        assert_eq!(pair.as_rule(), Rule::statement);

        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.into_inner();

        let p = match pair_iter.next() {
            Some(p) => p,
            None => bail!("Statement not found!\n\nLocation: {}", loc),
        };

        let l = Self::parse_location(&p);
        match p.as_rule() {
            Rule::block => Ok(Stmt::Block(Self::parse_block(p)?)),
            Rule::function_definition => Ok(Self::parse_func_def(p)?.into()),
            Rule::variable_declaration => Ok(Self::parse_var_decl(p)?.into()),
            Rule::assign_statement => Ok(Self::parse_assign_stmt(p)?.into()),
            Rule::expression => Ok(Self::parse_expr(p)?.into()),
            Rule::switch_statement => Ok(Self::parse_switch_stmt(p)?.into()),
            Rule::if_statement => Ok(Self::parse_if_stmt(p)?.into()),
            Rule::for_statement => Ok(Self::parse_for_stmt(p)?.into()),
            Rule::break_statement => Ok(Stmt::Break),
            Rule::continue_statement => Ok(Stmt::Continue),
            Rule::leave_statement => Ok(Stmt::Leave),
            _ => {
                bail!("Parsing statement: {}\n\nLocation: {}", p, l)
            }
        }
    }

    fn parse_assign_stmt(pair: Pair<Rule>) -> Result<AssignStmt> {
        assert_eq!(pair.as_rule(), Rule::assign_statement);

        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.into_inner();

        // Parse LHS
        let vars = match pair_iter.next() {
            Some(p) => Self::parse_ident_list(p)?,
            None => bail!("Assigned variables not found!\n\nLocation: {}", loc),
        };

        // parse RSH
        let value = match pair_iter.next() {
            Some(p) => Self::parse_expr(p)?,
            None => {
                bail!("Assigned value not found!\n\nLocation: {}", loc)
            }
        };

        Ok(AssignStmt::new(vars, value))
    }

    fn parse_if_stmt(pair: Pair<Rule>) -> Result<IfStmt> {
        assert_eq!(pair.as_rule(), Rule::if_statement);

        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.into_inner();

        // Parse condition
        let cond = match pair_iter.next() {
            Some(p) => Self::parse_expr(p)?,
            None => {
                bail!("Condition of `if` statement not found!\n\nLocation: {}", loc)
            }
        };

        // Parse body
        let body = match pair_iter.next() {
            Some(p) => Self::parse_block(p)?,
            None => {
                bail!("Body of `if` statement not found!\n\nLocation: {}", loc)
            }
        };

        Ok(IfStmt::new(cond, body))
    }

    fn parse_for_stmt(pair: Pair<Rule>) -> Result<ForStmt> {
        assert_eq!(pair.as_rule(), Rule::for_statement);

        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.into_inner();

        let pre_blk = match pair_iter.next() {
            Some(p) => Self::parse_block(p)?,
            None => {
                bail!("Pre-block of `for` statement not found!\n\nLocation: {}", loc)
            }
        };

        let cond = match pair_iter.next() {
            Some(p) => Self::parse_expr(p)?,
            None => {
                bail!("Condition of `for` statement not found!\n\nLocation: {}", loc)
            }
        };

        let post_blk = match pair_iter.next() {
            Some(p) => Self::parse_block(p)?,
            None => {
                bail!("Post-block of `for` statement not found!\n\nLocation: {}", loc)
            }
        };

        let body = match pair_iter.next() {
            Some(p) => Self::parse_block(p)?,
            None => {
                bail!("Body of `for` statement not found!\n\nLocation: {}", loc)
            }
        };

        Ok(ForStmt::new(pre_blk, cond, post_blk, body))
    }

    fn parse_switch_stmt(pair: Pair<Rule>) -> Result<SwitchStmt> {
        assert_eq!(pair.as_rule(), Rule::switch_statement);

        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.into_inner();

        // Parse the switch expression
        let expression = match pair_iter.next() {
            Some(p) => Self::parse_expr(p)?,
            None => bail!("Switch expression not found!\n\nLocation: {}", loc),
        };

        // Parse all switch cases
        let mut switch_values: Vec<SwitchValue> = vec![];
        let mut switch_defaults: Vec<SwitchDefault> = vec![];
        for p in pair_iter {
            let l = Self::parse_location(&p);
            match p.as_rule() {
                Rule::switch_value => switch_values.push(Self::parse_switch_value(p)?),
                Rule::switch_default => switch_defaults.push(Self::parse_switch_default(p)?),
                r => bail!("Parsing rule: {:?}\n\nLocation: {}", r, l),
            }
        }

        let switch_default = match switch_defaults.len() {
            0 => None,
            1 => Some(switch_defaults[0].clone()),
            _ => {
                bail!("Switch statement allows only at most 1 default case.\n\nLocation: {}", loc)
            }
        };

        Ok(SwitchStmt::new(expression, switch_values, switch_default))
    }

    fn parse_switch_value(pair: Pair<Rule>) -> Result<SwitchValue> {
        assert_eq!(pair.as_rule(), Rule::switch_value);

        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.into_inner();

        let literal = match pair_iter.next() {
            Some(p) => Self::parse_literal(p)?,
            None => bail!("Value of a switch value not found!\n\nLocation: {}", loc),
        };

        let body = match pair_iter.next() {
            Some(p) => Self::parse_block(p)?,
            None => bail!("Body of a switch value not found!\n\nLocation: {}", loc),
        };

        Ok(SwitchValue::new(literal, body))
    }

    fn parse_switch_default(pair: Pair<Rule>) -> Result<SwitchDefault> {
        assert_eq!(pair.as_rule(), Rule::switch_default);

        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.into_inner();

        let body = match pair_iter.next() {
            Some(p) => Self::parse_block(p)?,
            None => bail!("Switch default not found!\n\nLocation: {}", loc),
        };

        Ok(SwitchDefault::new(body))
    }

    //-------------------------------------------------
    // Expressions
    //-------------------------------------------------

    fn parse_expr(pair: Pair<Rule>) -> Result<Expr> {
        assert_eq!(pair.as_rule(), Rule::expression);

        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.into_inner();

        let p = match pair_iter.next() {
            Some(p) => p,
            None => {
                bail!("Expression not found!\n\nLocation: {}", loc)
            }
        };
        let l = Self::parse_location(&p);

        match p.as_rule() {
            Rule::function_call => Ok(Self::parse_func_call(p)?.into()),
            Rule::member_expr => Ok(Self::parse_member_expr(p)?.into()),
            Rule::identifier => Ok(Self::parse_ident(p)?.into()),
            Rule::literal => Ok(Self::parse_literal(p)?.into()),
            r => bail!("Parsing expression rule: {:?}\n\nLocation: {}", r, l),
        }
    }

    fn parse_func_call(pair: Pair<Rule>) -> Result<CallExpr> {
        assert_eq!(pair.as_rule(), Rule::function_call);
        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.into_inner();
        let func = match pair_iter.next() {
            Some(p) => Self::parse_ident(p)?,
            None => bail!("Function callee not fouund!\n\nLocation: {}", loc),
        };
        let mut args = vec![];
        for p in pair_iter {
            let l = Self::parse_location(&p);
            match p.as_rule() {
                Rule::expression => args.push(Self::parse_expr(p)?),
                r => bail!("Parsing argument rule: {:?}\n\nLocation: {}", r, l),
            }
        }
        Ok(CallExpr::new(func, args))
    }

    fn parse_member_expr(pair: Pair<Rule>) -> Result<MemberExpr> {
        assert_eq!(pair.as_rule(), Rule::member_expr);
        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.into_inner();
        let base: Name = match pair_iter.next() {
            Some(p) => Self::parse_ident(p)?.name,
            None => bail!("Base of member access not found!\n\nLocation: {}", loc),
        };
        let member: Name = match pair_iter.next() {
            Some(p) => Self::parse_ident(p)?.name,
            None => bail!("Member of member access not found!\n\nLocation: {}", loc),
        };
        Ok(MemberExpr::new(base, member, Some(loc)))
    }

    //-------------------------------------------------
    // Identifiers
    //-------------------------------------------------

    fn parse_typed_ident_list(pair: Pair<Rule>) -> Result<Vec<Identifier>> {
        assert_eq!(pair.as_rule(), Rule::typed_identifier_list);

        let mut idents: Vec<Identifier> = vec![];
        for p in pair.into_inner() {
            let l = Self::parse_location(&p);
            match p.as_rule() {
                Rule::typed_identifier => {
                    idents.push(Self::parse_typed_ident(p)?);
                }
                r => bail!("Parsing identifier rule: {:?}\n\nLocation: {}", r, l),
            }
        }
        Ok(idents)
    }

    fn parse_typed_ident(pair: Pair<Rule>) -> Result<Identifier> {
        assert_eq!(pair.as_rule(), Rule::typed_identifier);
        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.into_inner();
        let name = match pair_iter.next() {
            Some(p) => Self::parse_ident(p)?.name,
            None => bail!("Indentifer name not found!\n\nLocation: {}", loc),
        };
        let typ = match pair_iter.next() {
            None => Type::Unkn,
            Some(p) => Self::parse_data_type(p)?,
        };
        Ok(Identifier::new(name, typ, Some(loc)))
    }

    fn parse_ident_list(pair: Pair<Rule>) -> Result<Vec<Identifier>> {
        assert_eq!(pair.as_rule(), Rule::identifier_list);
        let mut idents: Vec<Identifier> = vec![];
        for p in pair.into_inner() {
            let l = Self::parse_location(&p);
            match p.as_rule() {
                Rule::identifier => {
                    idents.push(Self::parse_ident(p)?);
                }
                _ => {
                    bail!("Parsing identifier: {}\n\nLcoation: {}", p, l)
                }
            }
        }
        Ok(idents)
    }

    fn parse_ident(pair: Pair<Rule>) -> Result<Identifier> {
        assert_eq!(pair.as_rule(), Rule::identifier);
        let loc = Self::parse_location(&pair);
        let ident = pair.as_str();
        if YUL_RESERVED_NAMES.contains(&ident) {
            bail!("Identifier is a reserved name: {}!\n\nLocation: {}", ident, loc);
        }
        Ok(Identifier::new(Name::from(ident), Type::Unkn, Some(loc)))
    }

    //-------------------------------------------------
    // Data type
    //-------------------------------------------------

    fn parse_data_type(pair: Pair<Rule>) -> Result<Type> {
        assert_eq!(pair.as_rule(), Rule::data_type);
        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.into_inner();
        let p = match pair_iter.next() {
            Some(p) => p,
            None => {
                bail!("Data type not found!\n\nLocation: {}", loc)
            }
        };
        let l = Self::parse_location(&p);

        match p.as_rule() {
            Rule::bool_type => Ok(Type::Bool),
            Rule::string_type => Ok(Type::String),
            Rule::int_type => {
                let type_name = p.as_str();
                let regex = match Regex::new(r"(\d+)") {
                    Ok(regex) => regex,
                    Err(_) => bail!("Invalid regex!"),
                };
                let bw = match regex.captures(type_name) {
                    Some(capture) => match capture.get(1) {
                        Some(m) => match m.as_str().parse::<usize>() {
                            Ok(bw) => bw,
                            Err(_) => bail!("Invalid bitwidth: {}\n\nLocation: {}", p, l),
                        },
                        None => 256,
                    },
                    None => {
                        bail!("Invalid type: {}\n\nLocation: {}", p, l)
                    }
                };
                let signed = type_name.starts_with("int");
                Ok(Type::Int(IntType::new(bw, signed)))
            }
            _ => bail!("Parsing type: {}\n\nLocation: {}", p, l),
        }
    }

    //-------------------------------------------------
    // Literals
    //-------------------------------------------------

    fn parse_literal(pair: Pair<Rule>) -> Result<Lit> {
        assert_eq!(pair.as_rule(), Rule::literal);

        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.into_inner();

        let p = match pair_iter.next() {
            Some(p) => p,
            None => bail!("Literal not found!\n\nLocation: {}", loc),
        };

        let l = Self::parse_location(&p);
        match p.as_rule() {
            Rule::number_literal => Ok(Lit::from(Self::parse_num_lit(p)?)),
            Rule::hex_literal => Ok(Lit::from(Self::parse_hex_lit(p)?)),
            Rule::string_literal => Ok(Lit::from(Self::parse_string_lit(p)?)),
            Rule::bool_literal => Ok(Lit::from(Self::parse_bool_lit(p)?)),
            _ => bail!("Invalid literal: {}\n\nLocation: {}", p, l),
        }
    }

    fn parse_num_lit(pair: Pair<Rule>) -> Result<NumLit> {
        assert_eq!(pair.as_rule(), Rule::number_literal);

        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.clone().into_inner();

        let p = match pair_iter.next() {
            Some(p) => p,
            None => bail!("Number literal not found!\n\nLocation: {}", loc),
        };
        let l = Self::parse_location(&p);

        match p.as_rule() {
            Rule::decimal_number => match p.as_str().parse() {
                Ok(num) => Ok(NumLit::new_decimal(num)),
                Err(e) => bail!("Invalid decimal number: {}\n\nLocation: {}", e, l),
            },
            Rule::hex_number => Ok(NumLit::new_hex(p.as_str().to_string())),
            _ => bail!("Need to parse number literal: {}\n\nLocation: {}", p, l),
        }
    }

    fn parse_hex_lit(pair: Pair<Rule>) -> Result<HexLit> {
        assert_eq!(pair.as_rule(), Rule::hex_literal);

        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.clone().into_inner();

        let p = match pair_iter.next() {
            Some(p) => p,
            None => {
                bail!("Hex literal not found!\n\nLocation: {}", loc)
            }
        };

        Ok(HexLit::new(p.as_str()))
    }

    fn parse_string_lit(pair: Pair<Rule>) -> Result<StringLit> {
        assert_eq!(pair.as_rule(), Rule::string_literal);

        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.clone().into_inner();

        let p = match pair_iter.next() {
            Some(p) => p,
            None => bail!("String literal not found!\n\nLocation: {}", loc),
        };

        Ok(StringLit::new(p.as_str()))
    }

    fn parse_bool_lit(pair: Pair<Rule>) -> Result<BoolLit> {
        assert_eq!(pair.as_rule(), Rule::bool_literal);

        let loc = Self::parse_location(&pair);
        let mut pair_iter = pair.clone().into_inner();

        let p = match pair_iter.next() {
            Some(p) => p,
            None => {
                bail!("Bool literal not found!\n\nLocation: {}", loc)
            }
        };
        let l = Self::parse_location(&p);

        match p.as_rule() {
            Rule::TRUE => Ok(BoolLit::new(true)),
            Rule::FALSE => Ok(BoolLit::new(false)),
            _ => bail!("Not a bool literal: {}\n\nLocation: {}", p, l),
        }
    }

    //-------------------------------------------------
    // Comment
    //-------------------------------------------------

    fn parse_comment(pair: Pair<Rule>) -> Comment {
        assert_eq!(pair.as_rule(), Rule::COMMENT);

        let comment = pair.as_str().to_string();
        Comment::new(comment)
    }

    //-------------------------------------------------
    // Location
    //-------------------------------------------------

    fn parse_location(pair: &Pair<Rule>) -> Loc {
        let span = pair.as_span();
        let (start_pos, end_pos) = (span.start_pos(), span.end_pos());
        let (l1, c1) = start_pos.line_col();
        let (l2, c2) = end_pos.line_col();
        Loc::new(l1 as isize, c1 as isize, l2 as isize, c2 as isize)
    }
}

pub fn parse_input_file(filename: &str) -> Result<SourceUnit> {
    let content = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => bail!("Unable to read file: {}\n\n{}", filename, err),
    };

    let mut pairs = match YulParser::parse(Rule::source_unit, &content) {
        Ok(pairs) => pairs,
        Err(err) => bail!("Error: failed to parse Yul file: {}{}", filename, err),
    };

    match pairs.next() {
        Some(p) => YulParser::parse_source_unit(p),
        None => bail!("Source unit not found!"),
    }
}

pub fn parse_inline_assembly_block(yul_block: String) -> Result<Block> {
    let mut pairs = match YulParser::parse(Rule::block, &yul_block) {
        Ok(pairs) => pairs,
        Err(err) => {
            bail!("Error: failed to parse Yul block: {}{}", yul_block, err)
        }
    };

    match pairs.next() {
        Some(p) => YulParser::parse_block(p),
        None => bail!("Yul block not found!"),
    }
}
