use std::str;

use anyhow::Context;
use log::{error, info};
use thiserror::Error;

use crate::interpreter::stack::RcStack;
use crate::interpreter::variables::Variables;
use crate::parse::ast::{Executeable, ExecuteableType};

use super::{get_executor, DynExecutor, Executor, ExecutorError, Stack};

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("Task {0} is missing it's main")]
    MissingMain(String),
}

struct Executeables {
    pre: Option<Executeable>,
    main: Executeable,
    post: Option<Executeable>,
}

struct Executors {
    pre: Option<DynExecutor>,
    main: DynExecutor,
    post: Option<DynExecutor>,
    stack: RcStack,
}

pub struct TaskExecutor {
    name: String,
    variables: Variables,
    executeables: Option<Executeables>,
    executors: Option<Executors>,
}

impl TaskExecutor {
    pub fn new(input: Executeable) -> anyhow::Result<TaskExecutor> {
        if let ExecuteableType::Task { executeables } = input.executeable_type {
            let pre_executeable = TaskExecutor::find_executeable(&executeables, "pre");
            let main_executeable = match TaskExecutor::find_executeable(&executeables, "main") {
                Some(executeable) => executeable,
                None => return Err(TaskError::MissingMain(input.name).into()),
            };
            let post_executeable = TaskExecutor::find_executeable(&executeables, "post");
            Ok(TaskExecutor {
                name: input.name,
                variables: Variables::new(input.output_variables),
                executeables: Some(Executeables {
                    pre: pre_executeable,
                    main: main_executeable,
                    post: post_executeable,
                }),
                executors: None,
            })
        } else {
            Err(ExecutorError::WrongExecutorType(input.executeable_type).into())
        }
    }

    fn find_executeable(executeables: &[Executeable], name: &str) -> Option<Executeable> {
        for executeable in executeables {
            if executeable.name == name {
                return Some(executeable.clone());
            }
        }

        None
    }

    pub fn error_context(&self, stage: &'static str) -> String {
        format!("executing task '{}' at stage '{}'", self.name, stage)
    }

    fn convert_and_init_executeable(
        stack: &mut RcStack,
        executeable: Option<Executeable>,
    ) -> anyhow::Result<Option<DynExecutor>> {
        let mut executor = match executeable {
            Some(executeable) => get_executor(executeable, stack.clone())?,
            None => return Ok(None),
        };

        executor.init(stack.clone())?;

        Ok(Some(executor))
    }
}

impl Executor for TaskExecutor {
    fn init(&mut self, mut parent_stack: RcStack) -> anyhow::Result<()> {
        if let Some(executeables) = self.executeables.take() {
            let mut child_stack: RcStack = Stack::inherit_new(&parent_stack).into();

            let pre = Self::convert_and_init_executeable(&mut child_stack, executeables.pre)?;
            let mut main = get_executor(executeables.main, child_stack.clone())?;
            main.init(child_stack.clone())?;
            let post = Self::convert_and_init_executeable(&mut child_stack, executeables.post)?;

            self.variables
                .allocate_and_check_all(&mut parent_stack, &mut child_stack)?;

            self.executors = Some(Executors {
                pre,
                main,
                post,
                stack: child_stack,
            });

            Ok(())
        } else {
            Err(ExecutorError::NotInitialized.into())
        }
    }

    fn execute(&mut self, mut parent_stack: RcStack) -> anyhow::Result<()> {
        if let Some(mut executors) = self.executors.take() {
            info!("-> {}", &self.name);
            if let Some(mut pre) = executors.pre {
                pre.execute(executors.stack.clone())
                    .with_context(|| self.error_context("pre"))?;
            }
            executors
                .main
                .execute(executors.stack.clone())
                .with_context(|| self.error_context("main"))?;

            if let Some(mut post) = executors.post {
                post.execute(executors.stack.clone())
                    .with_context(|| self.error_context("post"))?;
            }

            self.variables
                .carry_over(&mut parent_stack, &mut executors.stack)?;
            Ok(())
        } else {
            Err(ExecutorError::NotInitialized.into())
        }
    }
}
