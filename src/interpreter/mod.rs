use std::cell::RefCell;

use thiserror::Error;

pub use self::namespace::RootNamespace;
use self::{
    context::{Context, ContextRef},
    executor::{get_executor, Executor},
    stack::{Stack, StackRef},
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
    stack: StackRef,
    executor: Box<dyn Executor>,
}

pub struct Interpreter {
    root_namespace: RootNamespace,
    execution_unit: Option<ExecutionUnit>,
    ctx: ContextRef,
}

impl Interpreter {
    pub fn new(root: RootNamespace) -> Self {
        let root_clone = root.clone();
        Self {
            root_namespace: root,
            execution_unit: None,
            ctx: Context::new(root_clone).into(),
        }
    }

    pub fn resolve(&mut self, task_name: &str) -> anyhow::Result<()> {
        let executeable = self.root_namespace.resolve_name(task_name)?;

        let stack: StackRef = Stack::new().into();
        let executor = get_executor(executeable.clone(), stack.clone())?;

        self.execution_unit = Some(ExecutionUnit { stack, executor });

        Ok(())
    }

    pub fn initialize(&mut self) -> anyhow::Result<()> {
        match &mut self.execution_unit {
            Some(unit) => unit.executor.init(unit.stack.clone(), self.ctx.clone()),
            None => Err(InterpreterError::InvalidState.into()),
        }
    }
    pub fn run(&mut self) -> anyhow::Result<()> {
        match &mut self.execution_unit {
            Some(unit) => unit.executor.execute(unit.stack.clone(), self.ctx.clone()),
            None => Err(InterpreterError::InvalidState.into()),
        }
    }
}
