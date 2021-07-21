extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::{borrow::Cow, collections::HashMap};

use anyhow::{anyhow, Result};

use pest::Parser;

mod parser;
use parser::*;

mod struct_defs;
pub use struct_defs::*;

mod compiler;
use compiler::*;

#[derive(Parser)]
#[grammar = "../gramar.pest"]
pub struct CtParser;

fn main() {
    let unparsed_file = std::fs::read_to_string("in.ct").expect("cannot read file");

    let file = CtParser::parse(Rule::file, &unparsed_file)
        .expect("unsuccessful parse") // unwrap the parse result
        .next()
        .unwrap();
    let functions: Vec<Option<FileElement>> = file.parse().unwrap();
    let functions: Vec<FileElement> = functions.into_iter().flatten().collect();
    let mut context = CompilationContext::default();
    functions
        .iter()
        .for_each(|x| x.compile(&mut context).unwrap());
    std::fs::write("out.ct", context.asm_file.join("\n")).unwrap();
}
