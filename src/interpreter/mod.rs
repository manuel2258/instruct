use thiserror::Error;

pub use self::namespace::RootNamespace;
use self::{
    executor::{get_executor, Executor},
    stack::{RcStack, Stack},
};

mod context;
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
    root_namespace: RootNamespace,
    execution_unit: Option<ExecutionUnit>,
}

impl Interpreter {
    pub fn new(root: RootNamespace) -> Self {
        Self {
            root_namespace: root,
            execution_unit: None,
        }
    }

    pub fn resolve(&mut self, task_name: &str) -> anyhow::Result<()> {
        let task_name_vec: Vec<&str> = task_name.split('.').collect();

        let executeable = self.root_namespace.resolve(&task_name_vec)?;

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
