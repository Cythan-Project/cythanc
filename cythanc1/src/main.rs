extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::borrow::Cow;

use anyhow::{anyhow, Result};

use pest::{
    iterators::{Pair, Pairs},
    Parser,
};

#[derive(Parser)]
#[grammar = "../gramar.pest"]
pub struct CtParser;

fn main() {
    let unparsed_file = std::fs::read_to_string("in.ct").expect("cannot read file");

    let file = CtParser::parse(Rule::file, &unparsed_file)
        .expect("unsuccessful parse") // unwrap the parse result
        .next()
        .unwrap();
    println!("Hello, world!");
}

struct Function<'a> {
    name: Cow<'a, str>,
    arguments: Vec<(Cow<'a, str>, Option<Cow<'a, str>>)>,
    body: CodeBlock<'a>,
}

struct CodeBlock<'a> {
    code: Vec<Instruction<'a>>,
}

enum Instruction<'a> {
    Expression(Expression<'a>),
    If(BooleanExpression<'a>, CodeBlock<'a>, Option<CodeBlock<'a>>),
    Loop(CodeBlock<'a>),
    Return(Option<Expression<'a>>),
    Assign(Cow<'a, str>, Box<Expression<'a>>),
    Continue,
    Break,
}

struct BooleanExpression<'a>(Expression<'a>, BooleanTest, Expression<'a>);

enum BooleanTest {
    Equals,
    NotEquals,
}

enum Expression<'a> {
    FunctionCall(Cow<'a, str>, Vec<Expression<'a>>),
    Variable(Cow<'a, str>),
    Number(u8),
}

pub trait ExprInto: Sized {
    fn expr_into(pairs: Pair<Rule>) -> Result<Self>;
}

// impl<'a, U, T> ExprInto<Vec<T>> for U
// where
//     U: ExprInto<T>,
// {
//     fn expr_into(self) -> Result<Vec<T>> {
//         self.into_inner()
//             .map(|x| T::expr_from(x))
//             .collect::<Result<Vec<_>, _>>()
//     }
// }

trait I {
    fn parse<T: ExprInto>(self) -> Result<T>;
}

impl I for Pair<'_, Rule> {
    fn parse<T: ExprInto>(self) -> Result<T> {
        T::expr_into(self)
    }
}

impl<T: ExprInto> ExprInto for Vec<T> {
    fn expr_into(pairs: Pair<Rule>) -> Result<Self> {
        pairs
            .into_inner()
            .map(|x| x.parse())
            .collect::<Result<Vec<_>, _>>()
    }
}

impl<T: ExprInto> ExprInto for Option<T> {
    fn expr_into(pairs: Pair<Rule>) -> Result<Self> {
        if let Rule::empty = pairs.as_rule() {
            Ok(None)
        } else {
            Ok(Some(pairs.parse()?))
        }
    }
}

impl<'a> ExprInto for Cow<'a, str> {
    fn expr_into(pairs: Pair<Rule>) -> Result<Self> {
        match pairs.as_rule() {
            Rule::literal => Ok(Cow::Owned(pairs.as_str().to_owned())),
            e => Err(anyhow!("Invalid rule : {:?}", e)),
        }
    }
}

impl ExprInto for u8 {
    fn expr_into(pairs: Pair<Rule>) -> Result<Self> {
        match pairs.as_rule() {
            Rule::number => Ok(pairs.as_str().parse()?),
            e => Err(anyhow!("Invalid rule : {:?}", e)),
        }
    }
}

impl ExprInto for BooleanExpression<'_> {
    fn expr_into(pairs: Pair<Rule>) -> Result<Self> {
        match pairs.as_rule() {
            Rule::boolean_expr => {
                let mut iter = pairs.into_inner();
                Ok(BooleanExpression(
                    iter.next().unwrap().parse()?,
                    iter.next().unwrap().parse()?,
                    iter.next().unwrap().parse()?,
                ))
            }
            e => Err(anyhow!("Invalid rule : {:?}", e)),
        }
    }
}

impl ExprInto for BooleanTest {
    fn expr_into(pairs: Pair<Rule>) -> Result<Self> {
        match pairs.as_rule() {
            Rule::test => Ok(match pairs.as_str() {
                "==" => BooleanTest::Equals,
                "!=" => BooleanTest::Equals,
                e => return Err(anyhow!("Invalid string : {:?}", e)),
            }),
            e => Err(anyhow!("Invalid rule : {:?}", e)),
        }
    }
}

impl ExprInto for Instruction<'_> {
    fn expr_into(pairs: Pair<Rule>) -> Result<Self> {
        match pairs.as_rule() {
            Rule::instruction => {}
            Rule::i_expr => Ok(Instruction::Expression(
                pairs.into_inner().next().unwrap().parse()?,
            )),
            Rule::i_loop => Ok(Instruction::Loop(
                pairs.into_inner().next().unwrap().parse()?,
            )),
            Rule::i_return => Ok(Instruction::Return(
                pairs.into_inner().next().unwrap().parse()?,
            )),
            Rule::i_continue => Ok(Instruction::Continue),
            Rule::i_break => Ok(Instruction::Break),
            Rule::i_assign => {
                let mut args = pairs.into_inner();
                Ok(Instruction::Assign(
                    args.next().unwrap().parse()?,
                    Box::new(args.next().unwrap().parse()?),
                ))
            }
            Rule::if_block => {
                let mut args = pairs.into_inner();
                Ok(Instruction::If(
                    args.next().unwrap().parse()?,
                    args.next().unwrap().parse()?,
                    args.next().unwrap().parse()?,
                ))
            }
            e => Err(anyhow!("Invalid rule : {:?}", e)),
        }
    }
}

impl ExprInto for CodeBlock<'_> {
    fn expr_into(pairs: Pair<Rule>) -> Result<Self> {
        match pairs.as_rule() {
            Rule::code_block => Ok(CodeBlock {
                code: pairs.parse()?,
            }),
            e => Err(anyhow!("Invalid rule : {:?}", e)),
        }
    }
}

impl ExprInto for Expression<'_> {
    fn expr_into(pairs: Pair<Rule>) -> Result<Self> {
        match pairs.as_rule() {
            Rule::func_call => {
                let mut args = pairs.into_inner();
                Ok(Expression::FunctionCall(
                    args.next().unwrap().parse()?,
                    args.next().unwrap().parse()?,
                ))
            }
            Rule::literal => Ok(Expression::Variable(pairs.parse()?)),
            Rule::number => Ok(Expression::Number(pairs.parse()?)),
            e => Err(anyhow!("Invalid rule : {:?}", e)),
        }
    }
}
