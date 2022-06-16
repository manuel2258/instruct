use anyhow::Context;
use log::{debug, error, info, warn};

use crate::interpreter::context::ContextRef;
use crate::interpreter::interpolateable::Interpolateable;
use crate::interpreter::stack::StackRef;
use crate::interpreter::variables::Variables;
use crate::parse::ast::{Executeable, ExecuteableType};

use super::{Executor, ExecutorError, Stack};

pub struct CommandExecutor {
    variables: Variables,
    cmd: String,
    interpolateable_cmd: Option<Interpolateable>,
    stdin_variable: Option<String>,
    trim_stdout: bool,
    trim_stderr: bool,
    stack: Option<StackRef>,
}

impl CommandExecutor {
    pub fn new(input: Executeable) -> anyhow::Result<Self> {
        if let ExecuteableType::Command { cmd } = input.executeable_type {
            let (stdin_variable, trim_stdout, trim_stderr) = match input.options {
                Some(bindings) => (
                    bindings.find("stdin").map(|val| val.into()),
                    bindings.find("trim_stdout").is_some(),
                    bindings.find("trim_stderr").is_some(),
                ),
                None => (None, false, false),
            };
            let mut exe = CommandExecutor {
                variables: Variables::new(input.output_variables),
                cmd,
                interpolateable_cmd: None,
                stdin_variable,
                trim_stdout,
                trim_stderr,
                stack: None,
            };
            exe.interpolateable_cmd = Interpolateable::new(&exe.cmd);
            Ok(exe)
        } else {
            Err(ExecutorError::WrongExecutorType(input.executeable_type).into())
        }
    }

    pub fn interpolate(&self, stack: &StackRef) -> anyhow::Result<String> {
        match &self.interpolateable_cmd {
            None => Ok(self.cmd.clone()),
            Some(inter) => {
                let mut target = String::new();
                inter
                    .interpolate(stack, &mut target)
                    .with_context(|| self.error_context())?;
                Ok(target)
            }
        }
    }

    pub fn error_context(&self) -> String {
        format!("executing command: '{}'", self.cmd)
    }
}

impl Executor for CommandExecutor {
    fn init(&mut self, mut stack: StackRef, _ctx: ContextRef) -> anyhow::Result<()> {
        if let Some(interpolateable) = &self.interpolateable_cmd {
            interpolateable
                .assert_variables_allocated(&stack)
                .with_context(|| self.error_context())?;
        }

        if let Some(stdin_variable) = &self.stdin_variable {
            stack.borrow().assert_allocated(stdin_variable)?;
        }

        let mut child_stack: StackRef = Stack::inherit_new(&stack).into();
        {
            let mut child_stack_ref = child_stack.borrow_mut();
            child_stack_ref.allocate("stdout".into());
            child_stack_ref.allocate("stderr".into());
            child_stack_ref.allocate("status".into());
        }

        self.variables
            .allocate_and_check_all(&mut stack, &mut child_stack)?;
        self.stack = Some(child_stack);

        Ok(())
    }

    fn execute(&mut self, mut parent_stack: StackRef, ctx: ContextRef) -> anyhow::Result<()> {
        if let Some(mut child_stack) = self.stack.clone() {
            let interpolated = self.interpolate(&parent_stack)?;

            debug!("$  {}", &interpolated);

            let result = ctx
                .borrow()
                .runner
                .run(
                    "default".into(),
                    interpolated,
                    self.trim_stdout,
                    self.trim_stderr,
                )
                .map_err(|err| ExecutorError::RunnerInterfaceError(err))?;

            if result.status != "0" {
                error!("$? {}", result.status);
            }

            if !result.stdout.is_empty() {
                info!("1> {}", &result.stdout);
            }
            if !result.stderr.is_empty() {
                warn!("2> {}", &result.stderr);
            }

            {
                let mut child_stack_ref = child_stack.borrow_mut();
                child_stack_ref
                    .set("stdout".into(), result.stdout)
                    .with_context(|| self.error_context())?;
                child_stack_ref
                    .set("stderr".into(), result.stderr)
                    .with_context(|| self.error_context())?;
                child_stack_ref
                    .set("status".into(), result.status)
                    .with_context(|| self.error_context())?;
            }

            self.variables
                .carry_over(&mut parent_stack, &mut child_stack)?;

            Ok(())
        } else {
            Err(ExecutorError::NotInitialized.into())
        }
    }
}
