use std::thread::{self, JoinHandle};

use thiserror::Error;

use crate::TaskLangError;

pub use self::namespace::RootNamespace;
use self::{
    context::{Context, ContextRef, RunnerRequester},
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
    pub fn new(root: RootNamespace, runner_requester: RunnerRequester) -> Self {
        let root_clone = root.clone();
        Self {
            root_namespace: root,
            execution_unit: None,
            ctx: Context::new(root_clone, runner_requester).into(),
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

    pub fn execute(&mut self) -> anyhow::Result<()> {
        match &mut self.execution_unit {
            Some(unit) => unit.executor.execute(unit.stack.clone(), self.ctx.clone()),
            None => Err(InterpreterError::InvalidState.into()),
        }
    }

    pub fn run(&mut self, task_name: &str) -> anyhow::Result<()> {
        self.resolve(task_name)
            .map_err(|err| TaskLangError::ResolveError(task_name.into(), err))?;
        self.initialize()
            .map_err(|err| TaskLangError::StaticAnalysisError(task_name.into(), err))?;
        self.execute()
            .map_err(|err| TaskLangError::ExecutionError(task_name.into(), err))?;
        Ok(())
    }

    pub fn run_as_new_thread(
        root_namespace: RootNamespace,
        runner_requester: RunnerRequester,
        task_name: String,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut interpreter = Interpreter::new(root_namespace, runner_requester);
            interpreter.run(&task_name).unwrap();
        })
    }
}
