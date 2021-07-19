use std::borrow::Cow;

use crate::template::Instruction;

pub enum Label<'a> {
    Label(Cow<'a, str>),
}

impl Instruction for Label<'_> {
    fn apply(&self, template: &mut crate::template::Template) {
        match self {
            Label::Label(a) => template.add_code(Cow::Owned(format!("'label_{}:no_op", a))),
        }
    }
}
