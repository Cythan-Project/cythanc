use std::borrow::Cow;

use crate::{
    instructions::DataRef,
    template::{Instruction, Template},
    utils::number_to_hex,
};

pub enum VariableDef<'a> {
    NumberVariable(Cow<'a, str>, u8),
    FunctionVariable(Cow<'a, str>, u8), /* FUNCTION_NAME, ARGUMENT_NUMBER */
}

impl Instruction for VariableDef<'_> {
    fn apply(&self, template: &mut Template) {
        match self {
            VariableDef::NumberVariable(a, b) => {
                template.add_section("VAR_DEF", Cow::Owned(format!("'var_{}:{}", a, b)))
            }
            VariableDef::FunctionVariable(a, b) => {
                template.add_section("VAR_DEF", Cow::Owned(format!("'var_{}_in{}:0", a, b)))
            }
        }
    }
}

pub enum VariableSet<'a> {
    Number(Cow<'a, str>, u8),
    Variable(Cow<'a, str>, Cow<'a, str>),
    Label(Cow<'a, str>, Cow<'a, str>),
    FunctionInput(Cow<'a, str>, u8, DataRef<'a>),
}

impl Instruction for VariableSet<'_> {
    fn apply(&self, template: &mut Template) {
        match self {
            VariableSet::Number(a, b) => {
                template.add_code(Cow::Owned(format!("'#{} 'var_{}", number_to_hex(*b), a)));
            }
            VariableSet::Variable(a, b) => {
                template.add_code(Cow::Owned(format!("'var_{} 'var_{}", b, a)));
                // CHANGED FROM format!("{} 'var_{}", b, a)
            }
            VariableSet::Label(a, b) => {
                if template
                    .section_contains("VAR_DEF", &format!("'#var_label_{}:'label_{}\n", b, b))
                {
                    template.add_section(
                        "VAR_DEF",
                        Cow::Owned(format!("'#var_label_{}:'label_{}\n", b, b)),
                    );
                }
                template.add_code(Cow::Owned(format!("'#var_label_{} 'var_{}\n", b, a,)));
            }
            VariableSet::FunctionInput(a, b, c) => match c {
                DataRef::Variable(c) => {
                    template.add_code(Cow::Owned(format!("'var_{} 'var_{}_in{}\n", c, a, b)));
                }
                DataRef::RefNum(c) => {
                    template.add_code(Cow::Owned(format!(
                        "'#{} 'var_{}_in{}\n",
                        number_to_hex(*c),
                        a,
                        b
                    )));
                }
            },
        }
    }
}
