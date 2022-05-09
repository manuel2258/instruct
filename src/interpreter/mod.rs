use anyhow::Context;
use thiserror::Error;

use crate::parse::ast::Namespace;

use self::{executor::get_executor, namespace::NamespaceNode, stack::Stack};

mod executor;
mod interpolateable;
mod namespace;
mod stack;
mod task;
mod variables;

#[derive(Error, Debug)]
pub enum InterpreterError {
    #[error("Could not find task {0}")]
    TaskNotFound(String),
}

pub struct Interpreter {
    root_namespace: NamespaceNode,
}

impl Interpreter {
    pub fn new(file: Namespace) -> Self {
        Self {
            root_namespace: NamespaceNode::new(file),
        }
    }

    pub fn run_task(&self, task_name: &str) -> anyhow::Result<()> {
        let task_name_vec: Vec<&str> = task_name.split(".").collect();

        let executeable = match self.root_namespace.find_executeable(&task_name_vec)? {
            Some(val) => val,
            None => return Err(InterpreterError::TaskNotFound(task_name.into()).into()),
        };

        let mut stack = Stack::new();
        let mut executor = get_executor(executeable.clone(), &mut stack)?;
        executor
            .init(&mut stack)
            .with_context(|| format!("at initializing task {0}", task_name))?;
        executor
            .execute(&mut stack)
            .with_context(|| format!("at executing task {0}", task_name))?;

        Ok(())
    }
}
