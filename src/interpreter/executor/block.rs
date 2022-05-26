use anyhow::Context;
use log::debug;

use crate::interpreter::context::ContextRef;
use crate::interpreter::stack::StackRef;
use crate::interpreter::variables::Variables;
use crate::parse::ast::{Executeable, ExecuteableType};

use super::{get_executor, DynExecutor, Executor, ExecutorError, Stack};

pub struct BlockExecutor {
    name: String,
    variables: Variables,
    executeables: Vec<Executeable>,
    executors: Vec<DynExecutor>,
    stack: Option<StackRef>,
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
    fn init(&mut self, mut parent_stack: StackRef, ctx: ContextRef) -> anyhow::Result<()> {
        let mut child_stack: StackRef = Stack::inherit_new(&parent_stack).into();

        for executeable in self.executeables.drain(..) {
            let mut executor = get_executor(executeable, child_stack.clone())?;
            executor.init(child_stack.clone(), ctx.clone())?;
            self.executors.push(executor);
        }

        self.variables
            .allocate_and_check_all(&mut parent_stack, &mut child_stack)?;

        self.stack = Some(child_stack);

        Ok(())
    }

    #[allow(clippy::needless_collect)]
    fn execute(&mut self, mut parent_stack: StackRef, ctx: ContextRef) -> anyhow::Result<()> {
        if let Some(mut child_stack) = self.stack.take() {
            debug!("{}: {{", &self.name);
            let executors: Vec<DynExecutor> = self.executors.drain(..).collect();
            for (counter, mut executor) in executors.into_iter().enumerate() {
                executor
                    .execute(child_stack.clone(), ctx.clone())
                    .with_context(|| self.error_context(counter))?;
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
