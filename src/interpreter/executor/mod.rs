
use thiserror::Error;

use crate::interpreter::stack::Stack;
use crate::parse::ast::{Executeable, ExecuteableType};

pub mod command;

#[derive(Error)]
pub enum ExecutorError {
    #[error("Tried to create executor with wrong executor type {0}, this should not happen!")]
    WrongExecutorType(ExecuteableType)

}




trait Executor {
    fn init(&mut self, ctx: &mut Stack) -> anyhow::Result<()>;

    fn execute(&mut self, ctx: &mut Stack) -> anyhow::Result<()>;
}

pub fn get_executor<'a>(input: Executeable, ctx: &'a Stack) -> Result<Box<dyn Executor>> {
    match input {
        Command()
    }
}
