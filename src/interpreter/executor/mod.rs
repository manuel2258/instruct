use thiserror::Error;

use crate::interpreter::stack::Stack;
use crate::parse::ast::{Executeable, ExecuteableType};

use self::block::BlockExecutor;
use self::call::CallExecutor;
use self::command::CommandExecutor;
use self::task::TaskExecutor;

use super::context::ContextRef;
use super::stack::StackRef;

mod block;
mod call;
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
    fn init(&mut self, stack: StackRef, ctx: ContextRef) -> anyhow::Result<()>;

    fn execute(&mut self, stack: StackRef, ctx: ContextRef) -> anyhow::Result<()>;
}

type DynExecutor = Box<dyn Executor>;

#[allow(unreachable_patterns)]
pub fn get_executor(input: Executeable, _stack: StackRef) -> anyhow::Result<DynExecutor> {
    match &input.executeable_type {
        ExecuteableType::Command { .. } => Ok(Box::new(CommandExecutor::new(input)?)),
        ExecuteableType::Task { .. } => Ok(Box::new(TaskExecutor::new(input)?)),
        ExecuteableType::Block { .. } => Ok(Box::new(BlockExecutor::new(input)?)),
        ExecuteableType::Call { .. } => Ok(Box::new(CallExecutor::new(input)?)),
        exec_type => Err(ExecutorError::NotImplemented(exec_type.clone()).into()),
    }
}
