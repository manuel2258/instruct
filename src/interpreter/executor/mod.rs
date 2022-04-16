use anyhow::Result;
use std::collections::HashMap;

use crate::parse::ast::Executeable;

pub mod block;
pub mod command;
pub mod task;

type ExecuteResult = Result<HashMap<String, String>>;

struct ExecutionContext<'a> {
    variables: HashMap<String, String>,
    child: Option<&'a ExecutionContext<'a>>
}

impl<'a> ExecutionContext<'a> {
    pub fn get(&self, name: &str) -> Option<String> {
        match self.variables.get(name) {
            Some(val) => Some(val.into()),
            None => {
                match self.child {
                    Some(child) => child.get(name),
                    None => None
                }
            }
        }
    }

    pub fn set(&mut self, name: String, value: String) {
        self.variables.insert(name, value);
    }
}

trait Executor {
    fn execute(&mut self) -> ExecuteResult;
}

pub fn get_executor<'a>(input: Executeable, ctx: &'a ExecutionContext) -> Result<Box<dyn Executor>> {
    match input {
        Command()
    }
}
