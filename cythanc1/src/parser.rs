use pest::iterators::Pair;

use crate::*;
use anyhow::*;

pub trait I {
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
            e => Err(anyhow!("Invalid rule 0 : {:?}", e)),
        }
    }
}

impl ExprInto for u8 {
    fn expr_into(pairs: Pair<Rule>) -> Result<Self> {
        match pairs.as_rule() {
            Rule::number => Ok(pairs.as_str().parse()?),
            e => Err(anyhow!("Invalid rule 1 : {:?}", e)),
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
            e => Err(anyhow!("Invalid rule 2 : {:?}", e)),
        }
    }
}

impl ExprInto for Option<FileElement<'_>> {
    fn expr_into(pairs: Pair<Rule>) -> Result<Self> {
        match pairs.as_rule() {
            Rule::function => {
                let mut iter = pairs.into_inner();
                Ok(Some(FileElement::Function(
                    iter.next().unwrap().parse()?,
                    iter.next().unwrap().parse()?,
                    iter.next().unwrap().parse()?,
                )))
            }
            Rule::EOI => Ok(None),
            Rule::extern_function => {
                let mut iter = pairs.into_inner();
                Ok(Some(FileElement::FunctionExtern(
                    iter.next().unwrap().parse()?,
                    iter.next().unwrap().parse()?,
                )))
            }
            e => Err(anyhow!("Invalid rule 4 : {:?} {}", e, pairs.as_str())),
        }
    }
}

impl ExprInto for BooleanTest {
    fn expr_into(pairs: Pair<Rule>) -> Result<Self> {
        match pairs.as_rule() {
            Rule::test => Ok(match pairs.as_str() {
                "==" => BooleanTest::Equals,
                "!=" => BooleanTest::NotEquals,
                e => return Err(anyhow!("Invalid string : {:?}", e)),
            }),
            e => Err(anyhow!("Invalid rule 5 : {:?}", e)),
        }
    }
}

impl ExprInto for Instruction<'_> {
    fn expr_into(pairs: Pair<Rule>) -> Result<Self> {
        match pairs.as_rule() {
            Rule::instruction => pairs.into_inner().next().unwrap().parse(),
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
            e => Err(anyhow!("Invalid rule 6 : {:?}", e)),
        }
    }
}

impl ExprInto for CodeBlock<'_> {
    fn expr_into(pairs: Pair<Rule>) -> Result<Self> {
        match pairs.as_rule() {
            Rule::code_block => Ok(CodeBlock {
                code: pairs.parse()?,
            }),
            e => Err(anyhow!("Invalid rule 7 : {:?}", e)),
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
            Rule::expr => pairs.into_inner().next().unwrap().parse(),
            e => Err(anyhow!("Invalid rule 8 : {:?}", e)),
        }
    }
}

pub trait ExprInto: Sized {
    fn expr_into(pairs: Pair<Rule>) -> Result<Self>;
}
