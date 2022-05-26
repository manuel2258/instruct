use anyhow::Context;

use crate::interpreter::context::ContextRef;
use crate::interpreter::stack::StackRef;
use crate::interpreter::variables::Variables;
use crate::parse::ast::{Executeable, ExecuteableType};

use super::{get_executor, DynExecutor, Executor, ExecutorError, Stack};

struct Executors {
    calle: DynExecutor,
    stack: StackRef,
}

pub struct CallExecutor {
    variables: Variables,
    target_name: String,
    executors: Option<Executors>,
}

impl CallExecutor {
    pub fn new(input: Executeable) -> anyhow::Result<Self> {
        if let ExecuteableType::Call { target } = input.executeable_type {
            let mut exe = CallExecutor {
                variables: Variables::new(input.output_variables),
                target_name: target,
                executors: None,
            };
            Ok(exe)
        } else {
            Err(ExecutorError::WrongExecutorType(input.executeable_type).into())
        }
    }

    pub fn error_context(&self) -> String {
        format!("calling: '{}'", self.target_name)
    }
}

impl Executor for CallExecutor {
    fn init(&mut self, mut stack: StackRef, ctx: ContextRef) -> anyhow::Result<()> {
        let calle_executeable = ctx
            .borrow()
            .root_namespace
            .resolve_name(&self.target_name)
            .with_context(|| self.error_context())?
            .clone();
        let mut calle_executor =
            get_executor(calle_executeable, stack.clone()).with_context(|| self.error_context())?;

        let mut child_stack: StackRef = Stack::inherit_new(&stack).into();

        calle_executor.init(child_stack.clone(), ctx.clone())?;

        self.variables
            .allocate_and_check_all(&mut stack, &mut child_stack)?;
        self.executors = Some(Executors {
            calle: calle_executor,
            stack: child_stack,
        });

        Ok(())
    }

    fn execute(&mut self, mut parent_stack: StackRef, ctx: ContextRef) -> anyhow::Result<()> {
        if let Some(mut executors) = self.executors.take() {
            executors
                .calle
                .execute(executors.stack.clone(), ctx.clone())
                .with_context(|| self.error_context())?;

            self.variables
                .carry_over(&mut parent_stack, &mut executors.stack)?;

            Ok(())
        } else {
            Err(ExecutorError::NotInitialized.into())
        }
    }
}
