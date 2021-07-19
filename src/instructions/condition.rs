use std::borrow::Cow;

use crate::template::{Instruction, Template};

pub enum Condition<'a> {
    If0(Cow<'a, str>, Cow<'a, str>),
}

impl Instruction for Condition<'_> {
    fn apply(&self, template: &mut Template) {
        match self {
            Self::If0(a, b) => {
                template.add_code(Cow::Owned(format!("if_0('var_{} 'label_{})", a, b)));
            }
        }
    }
}
