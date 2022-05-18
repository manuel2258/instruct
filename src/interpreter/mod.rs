use anyhow::Context;
use thiserror::Error;

use crate::parse::ast::Namespace;

use self::{
    executor::{get_executor, Executor},
    namespace::NamespaceResolver,
    stack::{RcStack, Stack},
};

mod executor;
mod interpolateable;
mod namespace;
mod stack;
mod variables;

#[derive(Error, Debug)]
pub enum InterpreterError {
    #[error("Interpreter is in invalid state")]
    InvalidState,
}

struct ExecutionUnit {
    stack: RcStack,
    executor: Box<dyn Executor>,
}

pub struct Interpreter {
    root_namespace: Namespace,
    execution_unit: Option<ExecutionUnit>,
}

impl Interpreter {
    pub fn new(root: Namespace) -> Self {
        Self {
            root_namespace: root,
            execution_unit: None,
        }
    }

    pub fn resolve(&mut self, task_name: &str) -> anyhow::Result<()> {
        let task_name_vec: Vec<&str> = task_name.split(".").collect();

        let resolver = NamespaceResolver::new(&self.root_namespace);

        let executeable = resolver.resolve(&task_name_vec)?;

        let stack: RcStack = Stack::new().into();
        let executor = get_executor(executeable.clone(), stack.clone())?;

        self.execution_unit = Some(ExecutionUnit { stack, executor });

        Ok(())
    }

    pub fn initialize(&mut self) -> anyhow::Result<()> {
        match &mut self.execution_unit {
            Some(unit) => unit.executor.init(unit.stack.clone()),
            None => Err(InterpreterError::InvalidState.into()),
        }
    }
    pub fn run(&mut self) -> anyhow::Result<()> {
        match &mut self.execution_unit {
            Some(unit) => unit.executor.execute(unit.stack.clone()),
            None => Err(InterpreterError::InvalidState.into()),
        }
    }
}
