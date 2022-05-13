use anyhow::Context;
use thiserror::Error;

use crate::parse::ast::Namespace;

use self::{
    executor::get_executor,
    namespace::NamespaceResolver,
    stack::{RcStack, Stack},
};

mod executor;
mod interpolateable;
mod namespace;
mod stack;
mod variables;

#[derive(Error, Debug)]
pub enum InterpreterError {}

pub struct Interpreter {
    root_namespace: Namespace,
}

impl Interpreter {
    pub fn new(file: Namespace) -> Self {
        Self {
            root_namespace: file,
        }
    }

    pub fn run_task(&self, task_name: &str) -> anyhow::Result<()> {
        let task_name_vec: Vec<&str> = task_name.split(".").collect();

        let resolver = NamespaceResolver::new(&self.root_namespace);

        let executeable = resolver.resolve(&task_name_vec)?;

        let stack: RcStack = Stack::new().into();
        let mut executor = get_executor(executeable.clone(), stack.clone())?;
        executor
            .init(stack.clone())
            .with_context(|| format!("at initializing task {0}", task_name))?;
        executor
            .execute(stack)
            .with_context(|| format!("at executing task {0}", task_name))?;

        Ok(())
    }
}
