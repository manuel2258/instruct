use std::process::Command;
use std::{collections::HashMap, str};

use anyhow::Context;
use log::{debug, error, info, warn};

use crate::interpreter::interpolateable::Interpolateable;
use crate::interpreter::variables::Variables;
use crate::parse::ast::{Executeable, ExecuteableType, VariableBindings};

use super::{Executor, ExecutorError, Stack};

pub struct CommandExecutor {
    variables: Variables,
    cmd: String,
    interpolateable_cmd: Option<Interpolateable>,
}

impl CommandExecutor {
    pub fn new(input: Executeable) -> anyhow::Result<CommandExecutor> {
        if let ExecuteableType::Command { cmd } = input.executeable_type {
            let mut exe = CommandExecutor {
                variables: Variables::new(input.output_variables),
                cmd,
                interpolateable_cmd: None,
            };
            exe.interpolateable_cmd = Interpolateable::new(&exe.cmd);
            Ok(exe)
        } else {
            Err(ExecutorError::WrongExecutorType(input.executeable_type).into())
        }
    }

    pub fn interpolate(&self, stack: &Stack) -> anyhow::Result<String> {
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

    pub fn check_output_var_valid(name: &str) -> anyhow::Result<()> {
        match name {
            "stdout" => Ok(()),
            "stderr" => Ok(()),
            "status" => Ok(()),
            _ => Err(ExecutorError::InvalidVariableBinding(name.into()).into()),
        }
    }

    pub fn error_context(&self) -> String {
        format!("at command: {:?}", self.cmd)
    }
}

impl Executor for CommandExecutor {
    fn init(&mut self, stack: &mut Stack) -> anyhow::Result<()> {
        self.variables
            .allocate_and_check_all(stack, &Self::check_output_var_valid)?;

        Ok(())
    }

    fn execute(&mut self, stack: &mut Stack) -> anyhow::Result<()> {
        let interpolated = self.interpolate(stack)?;
        let mut cmd_iter = interpolated.split(' ');
        let program = cmd_iter.next().unwrap();
        let args: Vec<&str> = cmd_iter.collect();

        let mut cmd = Command::new(program);
        cmd.args(args);

        debug!("$  {}", &interpolated);
        let output = cmd.output().with_context(|| self.error_context())?;
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

        self.variables
            .set_value(stack, "stdout", stdout)
            .with_context(|| self.error_context())?;
        self.variables
            .set_value(stack, "stderr", stderr)
            .with_context(|| self.error_context())?;
        self.variables
            .set_value(stack, "status", status)
            .with_context(|| self.error_context())?;

        Ok(())
    }
}
