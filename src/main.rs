#![feature(assoc_char_funcs)]

use std::{
    borrow::Cow,
    collections::HashMap,
    convert::{TryFrom, TryInto},
};

use crate::{
    instructions::{Condition, DataRef, GenericFunction, Jumps, Label, VariableDef, VariableSet},
    template::{Instruction, Template},
};

mod instructions;
mod template;
mod utils;

fn main() {
    let file = std::fs::read_to_string("in.ct").unwrap();
    let data = std::fs::read_to_string("template.ct")
        .unwrap()
        .replace("\r", "");
    let mut template = Template::new(&data);
    let mut state = State::default();
    file.lines()
        .map(|x| x.trim())
        .filter(|x| !x.starts_with("#") && !x.is_empty())
        .map(|x| compile(Cow::Borrowed(x), &mut state, &mut template))
        .collect::<Result<(), _>>()
        .unwrap();
    std::fs::write("out.ct", template.build()).unwrap();
}

fn compile<'a>(
    mut s: Cow<'a, str>,
    state: &mut State,
    template: &mut Template,
) -> Result<(), &'static str> {
    if let Some(e) = &state.func_state {
        s = Cow::Owned(
            e.arguments
                .iter()
                .enumerate()
                .fold(s.to_string(), |a, (i, b)| {
                    a.replace(&format!("${}", b), &format!("{}_in{}", e.name, i + 1))
                }),
        );
    }
    let mut iter = s.split(' ').filter(|x| !x.is_empty());
    let fnname = iter.next().ok_or_else(|| "Can't find function name")?;
    if fnname == "ret" {
        Jumps::JumpFuncEnd(Cow::Borrowed(&state.func_state.as_ref().unwrap().name)).apply(template);
        return Ok(());
    }
    if fnname == "call" {
        let fnname = iter.next().ok_or_else(|| "Can't find function name")?;
        if let Some(e) = state.cythan_funcs.get(fnname) {
            let arguments = iter
                .map(Value::from_str)
                .collect::<Result<Vec<Value>, _>>()?;
            if arguments.len() != *e as usize {
                return Err("Invalid number of args");
            }
            for (i, value) in arguments.iter().enumerate() {
                match value {
                    Value::RefNum(a) => VariableSet::FunctionInput(
                        Cow::Borrowed(fnname),
                        i as u8 + 1,
                        DataRef::RefNum(*a),
                    ),
                    Value::Variable(a) => VariableSet::FunctionInput(
                        Cow::Borrowed(fnname),
                        i as u8 + 1,
                        DataRef::Variable(a.clone()),
                    ),
                    Value::Num(_) => {
                        return Err(
                            "Can't pass a `num` as function argument expecyed `&num` or `var`",
                        );
                    }
                    Value::Label(_) => {
                        return Err(
                            "Can't pass a `'label` as function argument expecyed `&num` or `var`",
                        );
                    }
                }
                .apply(template);
            }
            let count = state.count();
            template.add_section(
                "VAR_DEF",
                Cow::Owned(format!("'#global_continue_{}:'continue_{}", count, count)),
            );
            template.add_code(Cow::Owned(format!(
                "'#global_continue_{} '{}_cb",
                count, fnname
            )));
            template.add_code(Cow::Owned(format!("jump('fnstart_{})", fnname)));
            template.add_code(Cow::Owned(format!("'continue_{}:no_op", count)));

            return Ok(());
        } else {
            println!("NAME: {}", fnname);
            return Err("No function declared with this name (Check the case)");
        }
    }
    if fnname == "func" {
        let name = iter.next().ok_or_else(|| "Can't find function name")?;
        let vec = iter.map(|x| x.to_owned()).collect::<Vec<_>>();
        template.set_code_section(Cow::Borrowed("FUNCTION_DEF"));
        template.add_section("VAR_DEF", Cow::Owned(format!("'{}_cb:16", name)));
        template.add_code(Cow::Owned(format!("'fnstart_{}:no_op\n", name)));
        for (i, _) in vec.iter().enumerate() {
            VariableDef::FunctionVariable(Cow::Borrowed(name), i as u8 + 1).apply(template);
        }
        state.func_state = Some(FuncState {
            name: name.to_owned(),
            arguments: vec,
        });
        return Ok(());
    }
    if fnname == "end_func" {
        Jumps::JumpFuncEnd(Cow::Borrowed(&state.func_state.as_ref().unwrap().name)).apply(template);
        let state1 = state.func_state.as_ref().unwrap();
        state
            .cythan_funcs
            .insert(state1.name.to_owned(), state1.arguments.len() as u32);
        state.func_state = None;
        template.set_code_section(Cow::Borrowed("CODE"));
        return Ok(());
    }
    let arguments = iter
        .map(Value::from_str)
        .collect::<Result<Vec<Value>, _>>()?;
    if let Some((func, compiler)) = state.functions.get(fnname) {
        if func.len() != arguments.len() {
            Err("Invalid number of args")
        } else if func.iter().zip(arguments.iter()).any(|(a, b)| !a.check(b)) {
            Err("Invalid argument")
        } else {
            Ok(compiler(arguments, template))
        }
    } else {
        println!("INVALID FUNCTION : {}", fnname);
        Err("Invalid function")
    }
}

struct State {
    functions: HashMap<String, (Vec<ValueType>, fn(Vec<Value>, &mut Template))>,
    cythan_funcs: HashMap<String, u32>,
    func_state: Option<FuncState>,
    counter: usize,
}

impl State {
    fn count(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }
}

struct FuncState {
    name: String,
    arguments: Vec<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            counter: 0,
            cythan_funcs: HashMap::new(),
            func_state: None,
            functions: {
                let mut map: HashMap<String, (Vec<ValueType>, fn(Vec<Value>, &mut Template))> =
                    HashMap::new();
                map.insert(
                    "label".to_owned(),
                    (vec![ValueType::Label], |a, b| {
                        Label::Label(a[0].label().unwrap().clone()).apply(b);
                    }),
                );
                map.insert(
                    "let".to_owned(),
                    (vec![ValueType::Variable, ValueType::Num], |a, b| {
                        VariableDef::NumberVariable(
                            a[0].var().unwrap().clone(),
                            a[1].num().unwrap(),
                        )
                        .apply(b);
                    }),
                );
                map.insert(
                    "if_0".to_owned(),
                    (vec![ValueType::Variable, ValueType::Label], |a, b| {
                        Condition::If0(a[0].var().unwrap().clone(), a[1].label().unwrap().clone())
                            .apply(b);
                    }),
                );
                map.insert(
                    "set".to_owned(),
                    (
                        vec![
                            ValueType::Variable,
                            ValueType::Or(vec![ValueType::Variable, ValueType::RefNum]),
                        ],
                        |a, b| {
                            match &a[1] {
                                Value::Variable(e) => {
                                    VariableSet::Variable(a[0].var().unwrap().clone(), e.clone())
                                }
                                Value::RefNum(e) => {
                                    VariableSet::Number(a[0].var().unwrap().clone(), *e)
                                }
                                _ => unreachable!(),
                            }
                            .apply(b);
                        },
                    ),
                );
                map.insert(
                    "exit".to_owned(),
                    (
                        vec![ValueType::Or(vec![ValueType::Variable, ValueType::RefNum])],
                        |a, b| {
                            GenericFunction::Exit(a[0].clone().try_into().unwrap()).apply(b);
                        },
                    ),
                );
                map.insert(
                    "inc".to_owned(),
                    (vec![ValueType::Variable], |a, b| {
                        GenericFunction::Inc(a[0].var().unwrap().clone()).apply(b);
                    }),
                );
                map.insert(
                    "no_op".to_owned(),
                    (vec![], |_, b| GenericFunction::NoOp.apply(b)),
                );
                map.insert(
                    "dec".to_owned(),
                    (vec![ValueType::Variable], |a, b| {
                        GenericFunction::Dec(a[0].var().unwrap().clone()).apply(b);
                    }),
                );
                map.insert(
                    "jump".to_owned(),
                    (vec![ValueType::Label], |a, b| {
                        Jumps::JumpLabel(a[0].label().unwrap().clone()).apply(b);
                    }),
                );
                map.insert(
                    "jump_var".to_owned(),
                    (vec![ValueType::Variable], |a, b| {
                        Jumps::JumpVariable(a[0].var().unwrap().clone()).apply(b);
                    }),
                );
                map.insert(
                    "set_lbl".to_owned(),
                    (vec![ValueType::Variable, ValueType::Label], |a, b| {
                        VariableSet::Label(
                            a[0].var().unwrap().clone(),
                            a[1].label().unwrap().clone(),
                        )
                        .apply(b)
                    }),
                );
                map
            },
        }
    }
}

#[derive(Clone)]
enum Value<'a> {
    RefNum(u8),
    Variable(Cow<'a, str>),
    Num(u8),
    Label(Cow<'a, str>),
}

impl<'a> Value<'a> {
    fn var(&'a self) -> Option<&Cow<'a, str>> {
        match &self {
            Self::Variable(e) => Some(e),
            _ => return None,
        }
    }
    fn label(&'a self) -> Option<&Cow<'a, str>> {
        match &self {
            Self::Label(e) => Some(e),
            _ => return None,
        }
    }
    fn num(&self) -> Option<u8> {
        Some(match self {
            Self::Num(0) => 16,
            Self::Num(a) => *a,
            _ => return None,
        })
    }
    fn from_str(s: &'a str) -> Result<Self, &'static str> {
        Ok(if s.starts_with("'") {
            Self::Label(Cow::Borrowed(&s[1..]))
        } else if s.starts_with("&") {
            if let Ok(e) = s[1..].parse::<u8>() {
                Self::RefNum(e)
            } else {
                return Err("No variable ref allowed");
            }
        } else {
            if let Ok(e) = s.parse::<u8>() {
                Self::Num(e)
            } else {
                Self::Variable(Cow::Borrowed(&s))
            }
        })
    }
}

enum ValueType {
    Or(Vec<ValueType>),
    RefNum,
    Variable,
    Num,
    Label,
}

impl ValueType {
    fn check(&self, value: &Value) -> bool {
        match self {
            ValueType::Or(e) => e.iter().any(|x| x.check(value)),
            ValueType::RefNum => matches!(value, Value::RefNum(_)),
            ValueType::Variable => matches!(value, Value::Variable(_)),
            ValueType::Num => matches!(value, Value::Num(_)),
            ValueType::Label => matches!(value, Value::Label(_)),
        }
    }
}
