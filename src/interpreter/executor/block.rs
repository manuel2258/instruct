use anyhow::Context;
use log::debug;

use crate::interpreter::stack::RcStack;
use crate::interpreter::variables::Variables;
use crate::parse::ast::{Executeable, ExecuteableType};

use super::{get_executor, DynExecutor, Executor, ExecutorError, Stack};

pub struct BlockExecutor {
    name: String,
    variables: Variables,
    executeables: Vec<Executeable>,
    executors: Vec<DynExecutor>,
    stack: Option<RcStack>,
}

impl BlockExecutor {
    pub fn new(input: Executeable) -> anyhow::Result<BlockExecutor> {
        if let ExecuteableType::Block { executeables } = input.executeable_type {
            Ok(BlockExecutor {
                name: input.name,
                variables: Variables::new(input.output_variables),
                executeables,
                executors: Vec::new(),
                stack: None,
            })
        } else {
            Err(ExecutorError::WrongExecutorType(input.executeable_type).into())
        }
    }

    pub fn error_context(&self, index: usize) -> String {
        format!("executing block '{}' at index '{}'", self.name, index)
    }
}

impl Executor for BlockExecutor {
    fn init(&mut self, mut parent_stack: RcStack) -> anyhow::Result<()> {
        let mut child_stack: RcStack = Stack::inherit_new(&parent_stack).into();

        for executeable in self.executeables.drain(..) {
            let mut executor = get_executor(executeable, child_stack.clone())?;
            executor.init(child_stack.clone())?;
            self.executors.push(executor);
        }

        self.variables
            .allocate_and_check_all(&mut parent_stack, &mut child_stack)?;

        self.stack = Some(child_stack);

        Ok(())
    }

    fn execute(&mut self, mut parent_stack: RcStack) -> anyhow::Result<()> {
        if let Some(mut child_stack) = self.stack.take() {
            debug!("{}: {{", &self.name);
            let mut counter = 0;
            let executors: Vec<DynExecutor> = self.executors.drain(..).collect();
            for mut executor in executors {
                executor
                    .execute(child_stack.clone())
                    .with_context(|| self.error_context(counter))?;
                counter += 1;
            }
            self.variables
                .carry_over(&mut parent_stack, &mut child_stack)?;
            debug!("}}\n");
            Ok(())
        } else {
            Err(ExecutorError::NotInitialized.into())
        }
    }
}
