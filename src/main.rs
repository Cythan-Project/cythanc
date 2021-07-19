use std::{borrow::Cow, collections::HashMap};

fn main() {
    let file = std::fs::read_to_string("code.ct").unwrap();
    let mut template = std::fs::read_to_string("template.ct")
        .unwrap()
        .replace("\r", "");
    let state = State::default();
    file.lines()
        .map(|x| x.trim())
        .filter(|x| !x.starts_with("#") && !x.is_empty())
        .map(|x| compile(x, &state, &mut template))
        .collect::<Result<(), _>>()
        .unwrap();
    std::fs::write("out.ct", template).unwrap();
}

fn compile(s: &str, state: &State, template: &mut String) -> Result<(), &'static str> {
    let mut iter = s.split(' ').filter(|x| !x.is_empty());
    let fnname = iter.next().ok_or_else(|| "Can't find function name")?;
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
        Err("Invalid function")
    }
}

struct State {
    functions: HashMap<String, (Vec<ValueType>, fn(Vec<Value>, &mut String))>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            functions: {
                let mut map: HashMap<String, (Vec<ValueType>, fn(Vec<Value>, &mut String))> =
                    HashMap::new();
                map.insert(
                    "label".to_owned(),
                    (vec![ValueType::Label], |a: Vec<Value>, b: &mut String| {
                        before(
                            b,
                            "CODE",
                            &format!(
                                "'label_{}:",
                                if let Value::Label(a) = &a[0] {
                                    a.as_ref()
                                } else {
                                    ""
                                }
                            ),
                        );
                    }),
                );
                map.insert(
                    "let".to_owned(),
                    (
                        vec![ValueType::Variable, ValueType::Num],
                        |a: Vec<Value>, b: &mut String| {
                            before(
                                b,
                                "VAR_DEF",
                                &format!("'var_{}:{}\n", a[0].var().unwrap(), a[1].num().unwrap()),
                            );
                        },
                    ),
                );
                map.insert(
                    "if_0".to_owned(),
                    (
                        vec![ValueType::Variable, ValueType::Label],
                        |a: Vec<Value>, b: &mut String| {
                            before(
                                b,
                                "CODE",
                                &format!(
                                    "if_0('var_{} 'label_{})\n",
                                    a[0].var().unwrap(),
                                    a[1].label().unwrap()
                                ),
                            );
                        },
                    ),
                );
                map.insert(
                    "set".to_owned(),
                    (
                        vec![
                            ValueType::Variable,
                            ValueType::Or(vec![ValueType::Variable, ValueType::RefNum]),
                        ],
                        |a: Vec<Value>, b: &mut String| {
                            before(
                                b,
                                "CODE",
                                &format!(
                                    "{} 'var_{}\n",
                                    match &a[1] {
                                        Value::Variable(e) => {
                                            format!("'var_{}", e)
                                        }
                                        Value::RefNum(_) => {
                                            a[1].ref_num().unwrap().to_owned()
                                        }
                                        _ => unreachable!(),
                                    },
                                    a[0].var().unwrap(),
                                ),
                            );
                        },
                    ),
                );
                map.insert(
                    "exit".to_owned(),
                    (
                        vec![ValueType::Or(vec![ValueType::Variable, ValueType::RefNum])],
                        |a: Vec<Value>, b: &mut String| {
                            before(
                                b,
                                "CODE",
                                &format!(
                                    "exit({})\n",
                                    match &a[0] {
                                        Value::Variable(e) => {
                                            format!("'var_{}", e)
                                        }
                                        Value::RefNum(_) => {
                                            a[0].ref_num().unwrap().to_owned()
                                        }
                                        _ => unreachable!(),
                                    },
                                ),
                            );
                        },
                    ),
                );
                map.insert(
                    "inc".to_owned(),
                    (
                        vec![ValueType::Variable],
                        |a: Vec<Value>, b: &mut String| {
                            before(
                                b,
                                "CODE",
                                &format!(
                                    "inc('var_{})\n",
                                    match &a[0] {
                                        Value::Variable(e) => e,
                                        _ => unreachable!(),
                                    },
                                ),
                            );
                        },
                    ),
                );
                map.insert(
                    "no_op".to_owned(),
                    (vec![], |_a: Vec<Value>, b: &mut String| {
                        before(b, "CODE", "no_op\n");
                    }),
                );
                map.insert(
                    "dec".to_owned(),
                    (
                        vec![ValueType::Variable],
                        |a: Vec<Value>, b: &mut String| {
                            before(
                                b,
                                "CODE",
                                &format!(
                                    "dec('var_{})\n",
                                    match &a[0] {
                                        Value::Variable(e) => e,
                                        _ => unreachable!(),
                                    },
                                ),
                            );
                        },
                    ),
                );
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
                map.insert(
                    "set_lbl".to_owned(),
                    (
                        vec![ValueType::Variable, ValueType::Label],
                        |a: Vec<Value>, b: &mut String| {
                            if !b.contains(&format!(
                                "'#var_label_{}:'label_{}\n",
                                a[1].label().unwrap(),
                                a[1].label().unwrap()
                            )) {
                                before(
                                    b,
                                    "VAR_DEF",
                                    &format!(
                                        "'#var_label_{}:'label_{}\n",
                                        a[1].label().unwrap(),
                                        a[1].label().unwrap()
                                    ),
                                );
                            }
                            before(
                                b,
                                "CODE",
                                &format!(
                                    "'#var_label_{} 'var_{} \n",
                                    a[1].label().unwrap(),
                                    match &a[0] {
                                        Value::Variable(e) => e,
                                        _ => unreachable!(),
                                    },
                                ),
                            );
                        },
                    ),
                );
                map
            },
        }
    }
}

fn after(tmp: &mut String, section: &str, value: &str) {
    *tmp = tmp.replace(
        &format!("# header {}\n", section),
        &format!("# header {}\n{}", section, value),
    );
}

fn before(tmp: &mut String, section: &str, value: &str) {
    *tmp = tmp.replace(
        &format!("# header {}\n", section),
        &format!("{}# header {}\n", value, section),
    );
}

enum Value<'a> {
    RefNum(u8),
    Variable(Cow<'a, str>),
    Num(u8),
    Label(Cow<'a, str>),
}

impl<'a> Value<'a> {
    fn var(&'a self) -> Option<&'a str> {
        match &self {
            Self::Variable(e) => Some(e.as_ref()),
            _ => return None,
        }
    }
    fn label(&'a self) -> Option<&'a str> {
        match &self {
            Self::Label(e) => Some(e.as_ref()),
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
    fn num_hex(&self) -> Option<&'static str> {
        Some(match self {
            Self::Num(0) => "0",
            Self::Num(1) => "1",
            Self::Num(2) => "2",
            Self::Num(3) => "3",
            Self::Num(4) => "4",
            Self::Num(5) => "5",
            Self::Num(6) => "6",
            Self::Num(7) => "7",
            Self::Num(8) => "8",
            Self::Num(9) => "9",
            Self::Num(10) => "A",
            Self::Num(11) => "B",
            Self::Num(12) => "C",
            Self::Num(13) => "D",
            Self::Num(14) => "E",
            Self::Num(15) => "F",
            _ => return None,
        })
    }
    fn ref_num(&self) -> Option<&'static str> {
        Some(match self {
            Self::RefNum(0) => "'#0",
            Self::RefNum(1) => "'#1",
            Self::RefNum(2) => "'#2",
            Self::RefNum(3) => "'#3",
            Self::RefNum(4) => "'#4",
            Self::RefNum(5) => "'#5",
            Self::RefNum(6) => "'#6",
            Self::RefNum(7) => "'#7",
            Self::RefNum(8) => "'#8",
            Self::RefNum(9) => "'#9",
            Self::RefNum(10) => "'#A",
            Self::RefNum(11) => "'#B",
            Self::RefNum(12) => "'#C",
            Self::RefNum(13) => "'#D",
            Self::RefNum(14) => "'#E",
            Self::RefNum(15) => "'#F",
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
