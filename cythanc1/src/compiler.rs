use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use crate::{CodeBlock, Expression, FileElement, Instruction};

use anyhow::*;

#[derive(Debug, Default)]
pub struct CompilationContext<'a> {
    pub functions_refs: HashMap<String, (bool, Vec<Cow<'a, str>>)>,
    pub counter: usize,
    pub asm_file: Vec<Cow<'a, str>>,
    pub current_expression_out_expr: Cow<'a, str>,
    pub current_function_context: Option<FunctionContext<'a>>,
    pub loops: Vec<usize>,
    pub existing_vars: HashSet<Cow<'a, str>>,
}

#[derive(Debug)]
pub struct FunctionContext<'a> {
    arguments: HashSet<Cow<'a, str>>,
    name: Cow<'a, str>,
}

impl<'a> CompilationContext<'a> {
    pub fn check_func(&self, fnname: &str, expressions: &Vec<Expression>) -> Result<()> {
        let args = self
            .functions_refs
            .get(fnname)
            .map(|(_, x)| x)
            .ok_or_else(|| anyhow!("Can't find function {}", fnname))?;
        if args.len() != expressions.len() {
            return Err(anyhow!("Invalid number of arguments"));
        }
        Ok(())
    }

    pub fn count(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }

    pub fn add(&mut self, string: String) {
        self.asm_file.push(Cow::Owned(string))
    }

    pub fn add_str(&'a mut self, string: &'a str) {
        self.asm_file.push(Cow::Borrowed(string))
    }
}

impl<'a> FileElement<'a> {
    pub fn compile(&'a self, context: &'a mut CompilationContext) -> Result<()> {
        match self {
            FileElement::Function(a, b, c) => {
                if a == "main" {
                    context.current_function_context = None;
                    c.compile(context)?;
                } else {
                    context.add(format!("let {}_out 0", a));
                    context.add(format!("func {} {}", a, b.join(" ")));
                    context.current_function_context = Some(FunctionContext {
                        arguments: b
                            .iter()
                            .cloned()
                            .map(|x| Cow::Owned(x.into_owned()))
                            .collect(),
                        name: Cow::Owned(a.clone().into_owned()),
                    });
                    c.compile(context)?;
                    context.add("end_func".to_owned());
                    context.functions_refs.insert(
                        a.clone().into_owned(),
                        (
                            false,
                            b.iter()
                                .cloned()
                                .map(|x| Cow::Owned(x.into_owned()))
                                .collect(),
                        ),
                    );
                }
            }
            FileElement::FunctionExtern(a, b) => {
                context.functions_refs.insert(
                    a.clone().into_owned(),
                    (
                        true,
                        b.iter()
                            .cloned()
                            .map(|x| Cow::Owned(x.into_owned()))
                            .collect(),
                    ),
                );
            }
        }
        Ok(())
    }
}

impl CodeBlock<'_> {
    pub fn compile(&self, context: &mut CompilationContext) -> Result<()> {
        for i in &self.code {
            i.compile(context)?
        }
        Ok(())
    }
}

impl<'a> Instruction<'a> {
    pub fn compile(&self, context: &mut CompilationContext) -> Result<()> {
        match self {
            Instruction::Expression(a) => a.compile(context)?,
            Instruction::If(a, b, c) => {
                let current = context.count();
                if matches!(a.2, Expression::Number(0)) {
                    a.0.compile(context)?;
                } else if matches!(a.0, Expression::Number(0)) {
                    a.2.compile(context)?;
                } else {
                    Expression::FunctionCall(Cow::Borrowed("sub"), vec![a.0.clone(), a.2.clone()])
                        .compile(context)?;
                }
                match a.1 {
                    crate::BooleanTest::Equals => {
                        context.add(format!(
                            "if_0 {} 'if_true{}",
                            context.current_expression_out_expr, current
                        ));
                        if let Some(e) = c {
                            e.compile(context)?;
                        }
                        context.add(format!("jump 'if_end{}", current));
                        context.add(format!("label 'if_true{}", current));
                        b.compile(context)?;
                        context.add(format!("label 'if_end{}", current));
                    }
                    crate::BooleanTest::NotEquals => {
                        context.add(format!(
                            "if_0 {} 'if_false{}",
                            context.current_expression_out_expr, current
                        ));
                        if let Some(e) = c {
                            e.compile(context)?;
                        }
                        context.add(format!("jump 'if_end{}", current));
                        context.add(format!("label 'if_false{}", current));
                        b.compile(context)?;
                        context.add(format!("label 'if_end{}", current));
                    }
                }
            }
            Instruction::Loop(a) => {
                let current_loop = context.count();
                context.add(format!("label 'for{}", current_loop));
                context.loops.push(current_loop);
                a.compile(context)?;
                context.loops.pop();
                context.add(format!("label 'for_end{}", current_loop));
            }
            Instruction::Return(a) => {
                let fnname = &context
                    .current_function_context
                    .as_ref()
                    .ok_or_else(|| anyhow!("Can't use return outside of a function"))?
                    .name
                    .clone();
                if let Some(a) = a {
                    a.compile(context)?;
                    context.add(format!(
                        "set {}_out {}",
                        fnname, context.current_expression_out_expr
                    ));
                }
                context.add("ret".to_owned());
            }
            Instruction::Assign(a, b) => {
                b.compile(context)?;
                context.add(format!("set {} {}", a, context.current_expression_out_expr));
            }
            Instruction::Continue => {
                context.add(format!("jump 'for{}", context.loops.last().unwrap()))
            }
            Instruction::Break => {
                context.add(format!("jump 'for_end{}", context.loops.last().unwrap()))
            }
        }
        Ok(())
    }
}

impl<'a> Expression<'a> {
    pub fn compile(&self, context: &mut CompilationContext) -> Result<()> {
        match self {
            Expression::FunctionCall(a, b) => {
                let calln = context.count();
                context.check_func(a.as_ref(), b)?;
                if context.functions_refs.get(a.as_ref()).unwrap().0 {
                    let s = format!(
                        "{} {}",
                        a,
                        b.iter()
                            .map(|x| {
                                x.compile(context)?;
                                let out = &context.current_expression_out_expr;
                                Ok(out.as_ref().to_owned())
                            })
                            .collect::<Result<Vec<String>>>()?
                            .join(" ")
                    );
                    context.add(s);
                } else {
                    let s = format!(
                        "call {} {}",
                        a,
                        b.iter()
                            .map(|x| {
                                x.compile(context)?;
                                let out = &context.current_expression_out_expr;
                                Ok(out.as_ref().to_owned())
                            })
                            .collect::<Result<Vec<String>>>()?
                            .join(" ")
                    );
                    context.add(s);
                    context.add(format!("let TMP{} 0", calln));
                    context.add(format!("set TMP{} {}_out", calln, a));
                    context.current_expression_out_expr = Cow::Owned(format!("TMP{}", calln))
                }
            }
            Expression::Variable(a) => {
                if context
                    .current_function_context
                    .as_ref()
                    .map(|x| x.arguments.contains(a))
                    .unwrap_or(false)
                {
                    context.current_expression_out_expr = Cow::Owned(format!("${}", a));
                } else {
                    if !context.existing_vars.contains(a) {
                        context
                            .existing_vars
                            .insert(Cow::Owned(a.as_ref().to_owned()));
                        context.add(format!("let {} 0", a))
                    }
                    context.current_expression_out_expr = Cow::Owned(a.as_ref().to_owned());
                }
            }
            Expression::Number(a) => {
                context.current_expression_out_expr = Cow::Owned(format!("&{}", a));
            }
        }
        Ok(())
    }
}
