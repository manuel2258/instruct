use std::io::Write;
use std::process::{Command, Stdio};
use std::str;

use anyhow::Context;
use log::{debug, error, info, warn};

use crate::interpreter::interpolateable::Interpolateable;
use crate::interpreter::stack::RcStack;
use crate::interpreter::variables::Variables;
use crate::parse::ast::{Executeable, ExecuteableType};

use super::{Executor, ExecutorError, Stack};

pub struct CommandExecutor {
    variables: Variables,
    cmd: String,
    interpolateable_cmd: Option<Interpolateable>,
    stdin_variable: Option<String>,
    stack: Option<RcStack>,
}

impl CommandExecutor {
    pub fn new(input: Executeable) -> anyhow::Result<Self> {
        if let ExecuteableType::Command { cmd } = input.executeable_type {
            let stdin_variable: Option<String> = match input.options {
                Some(bindings) => bindings.find("stdin").map(|val| val.into()),
                None => None,
            };
            let mut exe = CommandExecutor {
                variables: Variables::new(input.output_variables),
                cmd,
                interpolateable_cmd: None,
                stdin_variable,
                stack: None,
            };
            exe.interpolateable_cmd = Interpolateable::new(&exe.cmd);
            Ok(exe)
        } else {
            Err(ExecutorError::WrongExecutorType(input.executeable_type).into())
        }
    }

    pub fn interpolate(&self, stack: &RcStack) -> anyhow::Result<String> {
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
    fn init(&mut self, mut stack: RcStack) -> anyhow::Result<()> {
        if let Some(interpolateable) = &self.interpolateable_cmd {
            interpolateable
                .assert_variables_allocated(&stack)
                .with_context(|| self.error_context())?;
        }

        if let Some(stdin_variable) = &self.stdin_variable {
            stack.borrow().assert_allocated(stdin_variable)?;
        }

        let mut child_stack: RcStack = Stack::inherit_new(&stack).into();
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

    fn execute(&mut self, mut parent_stack: RcStack) -> anyhow::Result<()> {
        if let Some(mut child_stack) = self.stack.clone() {
            let interpolated = self.interpolate(&mut parent_stack)?;
            let mut cmd_iter = interpolated.split(' ');
            let program = cmd_iter.next().unwrap();
            let args: Vec<&str> = cmd_iter.collect();

            let mut cmd = Command::new(program);
            cmd.args(args);
            cmd.stdin(Stdio::piped());
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            debug!("$  {}", &interpolated);
            let mut process = cmd.spawn().with_context(|| self.error_context())?;

            if let Some(stdin_variable) = &self.stdin_variable {
                let stdin = parent_stack.borrow().get(stdin_variable)?;
                debug!("<  {}", &stdin);
                write!(process.stdin.as_mut().unwrap(), "{}", &stdin)?;
            }

            let output = process
                .wait_with_output()
                .with_context(|| self.error_context())?;
            let stdout: String = str::from_utf8(&output.stdout).unwrap().into();
            let stderr: String = str::from_utf8(&output.stderr).unwrap().into();
            let status: String = output.status.code().unwrap().to_string();

            if !output.status.success() {
                error!("$? {}", status);
            }

            if !stdout.is_empty() {
                info!("1> {}", &stdout);
            }
            if !stderr.is_empty() {
                warn!("2> {}", &stderr);
            }

            {
                let mut child_stack_ref = child_stack.borrow_mut();
                child_stack_ref
                    .set("stdout".into(), stdout)
                    .with_context(|| self.error_context())?;
                child_stack_ref
                    .set("stderr".into(), stderr)
                    .with_context(|| self.error_context())?;
                child_stack_ref
                    .set("status".into(), status)
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
