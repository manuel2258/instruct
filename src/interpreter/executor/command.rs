use std::process::Command;
use std::{collections::HashMap, str};

use crate::interpreter::interpolateable::Interpolateable;
use crate::parse::ast::{Executeable, ExecuteableType};

use super::{Executor, Stack, ExecutorError};


pub struct CommandExecutor<'a> {
    cmd: String,
    interpolateable_cmd: Option<Interpolateable<'a>>,
}

impl<'a> CommandExecutor<'a> {
    pub fn new(input: Executeable) -> anyhow::Result<Self> {
        if let ExecuteableType::Command{cmd} = input.exec_type {
            
            Ok(Self { cmd, Interpolateable::new() })
        } else {
            Err(ExecutorError::WrongExecutorType(input.exec_type))
        }
    }

    pub fn interpolate()
}

impl Executor for CommandExecutor {
    fn init(&mut self, ctx: &mut Stack) -> anyhow::Result<()> {

    }

    fn execute(&mut self, ctx: &mut Stack) -> anyhow::Result<()> {
        let mut cmd_iter = input.split(' ');
        let program = cmd_iter.next().unwrap();
        let args: Vec<&str> = cmd_iter.collect();


        let mut cmd = Command::new(program);
        cmd.args(args);

        let output = self.cmd.output()?;
        println!("{} -> {:?}", self.input, output);

        let mut res = HashMap::new();
        res.insert("stdout".into(), str::from_utf8(&output.stdout)?.into());
        res.insert("stderr".into(), str::from_utf8(&output.stderr)?.into());
        Ok(res)
    }
}
