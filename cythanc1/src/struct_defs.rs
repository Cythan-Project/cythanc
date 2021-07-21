use std::borrow::Cow;

#[derive(Debug)]
pub struct CodeBlock<'a> {
    pub code: Vec<Instruction<'a>>,
}

#[derive(Debug)]
pub enum Instruction<'a> {
    Expression(Expression<'a>),
    If(BooleanExpression<'a>, CodeBlock<'a>, Option<CodeBlock<'a>>),
    Loop(CodeBlock<'a>),
    Return(Option<Expression<'a>>),
    Assign(Cow<'a, str>, Box<Expression<'a>>),
    Continue,
    Break,
}

#[derive(Debug)]
pub struct BooleanExpression<'a>(pub Expression<'a>, pub BooleanTest, pub Expression<'a>);

#[derive(Debug)]
pub enum BooleanTest {
    Equals,
    NotEquals,
}

#[derive(Debug, Clone)]
pub enum Expression<'a> {
    FunctionCall(Cow<'a, str>, Vec<Expression<'a>>),
    Variable(Cow<'a, str>),
    Number(u8),
}

#[derive(Debug)]
pub enum FileElement<'a> {
    Function(Cow<'a, str>, Vec<Cow<'a, str>>, CodeBlock<'a>),
    FunctionExtern(Cow<'a, str>, Vec<Cow<'a, str>>),
}
