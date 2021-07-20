use std::{borrow::Cow, collections::{HashMap, HashSet}};

use crate::Expression;

use anyhow::*;



pub struct CompilationContext<'a> {
    pub functions_refs: HashMap<String, Vec<Cow<'a, str>>>,
    pub counter: usize,
    pub asm_file: Vec<Cow<'a, str>>,
    pub current_expression_out_expr: Cow<'a, str>,
    pub current_in_use_vars: HashSet<Cow<'a, str>>
}

impl <'a> CompilationContext<'a> {

    pub fn check_func(&self, fnname: &str, expressions: &Vec<Expression>) -> Result<()> {
        let args = self.functions_refs.get(fnname).ok_or_else(|| anyhow!("Can't find function {}",fnname))?;
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

impl <'a> Expression<'a> {

    fn compile(&self, context: &mut CompilationContext) -> Result<()> {
        match self {
            Expression::FunctionCall(a, b) => {
                context.check_func(a.as_ref(), b)?;
                context.add(format!("call {} {}",a, b.iter().map(|x| {
                    x.compile(&mut context);
                    let out = context.current_expression_out_expr;

                    out
                }).collect::<Vec<String>>().join(" ")))
                call SUB &5 &3
            },
            Expression::Variable(_) => todo!(),
            Expression::Number(_) => todo!(),
        }
    }
}