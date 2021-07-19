use std::{borrow::Cow, convert::TryFrom, fmt::Display};

use crate::{
    template::{Instruction, Template},
    utils::number_to_hex,
    Value,
};

pub enum GenericFunction<'a> {
    Exit(DataRef<'a>),
    Inc(Cow<'a, str>),
    Dec(Cow<'a, str>),
    NoOp,
}

pub enum DataRef<'a> {
    Variable(Cow<'a, str>),
    RefNum(u8),
}

impl<'a> TryFrom<Value<'a>> for DataRef<'a> {
    type Error = ();

    fn try_from(value: Value<'a>) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::RefNum(a) => DataRef::RefNum(a),
            Value::Variable(a) => DataRef::Variable(a),
            Value::Num(_) | Value::Label(_) => return Err(()),
        })
    }
}

impl Display for DataRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataRef::Variable(a) => write!(f, "'var_{}", a),
            DataRef::RefNum(a) => write!(f, "'#{}", number_to_hex(*a)),
        }
    }
}

impl Instruction for GenericFunction<'_> {
    fn apply(&self, template: &mut Template) {
        match self {
            GenericFunction::Exit(a) => template.add_code(Cow::Owned(format!("exit({})", a))),
            GenericFunction::Inc(a) => template.add_code(Cow::Owned(format!("inc('var_{})", a))),
            GenericFunction::Dec(a) => template.add_code(Cow::Owned(format!("dec('var_{})", a))),
            GenericFunction::NoOp => template.add_code(Cow::Borrowed("no_op")),
        }
    }
}
