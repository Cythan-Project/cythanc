use std::borrow::Cow;

use crate::template::Instruction;

pub enum Jumps<'a> {
    JumpLabel(Cow<'a, str>),
    JumpVariable(Cow<'a, str>),
    JumpFuncEnd(Cow<'a, str>),
}

impl Instruction for Jumps<'_> {
    fn apply(&self, template: &mut crate::template::Template) {
        match self {
            Jumps::JumpLabel(a) => template.add_code(Cow::Owned(format!("jump('label_{})", a))),
            Jumps::JumpVariable(a) => {
                template.add_code(Cow::Owned(format!("'var_{} ~+3 ~+2 0 earasable", a)))
            }
            Jumps::JumpFuncEnd(a) => {
                template.add_code(Cow::Owned(format!("'{}_cb ~+3 ~+2 0 earasable", a)))
            }
        }
    }
}

/*

                map.insert(
                    "jump".to_owned(),
                    (vec![ValueType::Label], |a: Vec<Value>, b: &mut String| {
                        before(
                            b,
                            "CODE",
                            &format!(
                                "jump('label_{})\n",
                                match &a[0] {
                                    Value::Label(e) => e,
                                    _ => unreachable!(),
                                },
                            ),
                        );
                    }),
                );
                map.insert(
                    "jump_var".to_owned(),
                    (
                        vec![ValueType::Variable],
                        |a: Vec<Value>, b: &mut String| {
                            before(
                                b,
                                "CODE",
                                &format!(
                                    "'var_{} ~+3 ~+2 0 earasable\n",
                                    match &a[0] {
                                        Value::Variable(e) => e,
                                        _ => unreachable!(),
                                    },
                                ),
                            );
                        },
                    ),
                );
*/
