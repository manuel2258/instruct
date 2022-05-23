use thiserror::Error;

use crate::interpreter::stack::Stack;
use crate::parse::ast::{Executeable, ExecuteableType};

use self::block::BlockExecutor;
use self::command::CommandExecutor;
use self::task::TaskExecutor;

use super::stack::RcStack;

mod block;
mod command;
mod task;

#[derive(Error, Debug)]
pub enum ExecutorError {
    #[error("Tried to create executor with wrong executor type {0:?}, this should not happen!")]
    WrongExecutorType(ExecuteableType),
    #[error("Executor for type {0:?} not yet implemented!")]
    NotImplemented(ExecuteableType),
    #[error("Executor was not initialized")]
    NotInitialized,
}

pub trait Executor {
    fn init(&mut self, stack: RcStack) -> anyhow::Result<()>;

    fn execute(&mut self, stack: RcStack) -> anyhow::Result<()>;
}

type DynExecutor = Box<dyn Executor>;

pub fn get_executor(input: Executeable, _stack: RcStack) -> anyhow::Result<DynExecutor> {
    match &input.executeable_type {
        ExecuteableType::Command { .. } => Ok(Box::new(CommandExecutor::new(input)?)),
        ExecuteableType::Task { .. } => Ok(Box::new(TaskExecutor::new(input)?)),
        ExecuteableType::Block { .. } => Ok(Box::new(BlockExecutor::new(input)?)),
        exec_type => Err(ExecutorError::NotImplemented(exec_type.clone()).into()),
    }
}
