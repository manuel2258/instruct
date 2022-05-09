use thiserror::Error;

use crate::interpreter::stack::Stack;
use crate::parse::ast::{Executeable, ExecuteableType};

use self::command::CommandExecutor;

pub mod command;

#[derive(Error, Debug)]
pub enum ExecutorError {
    #[error("Tried to create executor with wrong executor type {0:?}, this should not happen!")]
    WrongExecutorType(ExecuteableType),
    #[error("Executor for type {0:?} not yet implemented!")]
    ExecutorNotImplemented(ExecuteableType),

    #[error("Invalid variable binding, executor does not provide value for '{0}'")]
    InvalidVariableBinding(String),
}

pub trait Executor {
    fn init(&mut self, stack: &mut Stack) -> anyhow::Result<()>;

    fn execute(&mut self, stack: &mut Stack) -> anyhow::Result<()>;
}

pub fn get_executor(input: Executeable, stack: &mut Stack) -> anyhow::Result<Box<dyn Executor>> {
    match &input.executeable_type {
        ExecuteableType::Command { .. } => Ok(Box::new(CommandExecutor::new(input)?)),
        exec_type => Err(ExecutorError::ExecutorNotImplemented(exec_type.clone()).into()),
    }
}
