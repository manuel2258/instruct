use std::process::Command;
use std::{collections::HashMap, str};

use super::{ExecuteResult, Executor};

pub struct CommandExecutor {
    input: String,
    cmd: Command,
}

impl CommandExecutor {
    pub fn new(input: String) -> Self {
        let mut cmd_iter = input.split(' ');
        let program = cmd_iter.next().unwrap();
        let args: Vec<&str> = cmd_iter.collect();

        let mut cmd = Command::new(program);
        cmd.args(args);
        Self { input, cmd }
    }
}

impl Executor for CommandExecutor {
    fn execute(&mut self) -> ExecuteResult {
        let output = self.cmd.output()?;
        println!("{} -> {:?}", self.input, output);

        let mut res = HashMap::new();
        res.insert("stdout".into(), str::from_utf8(&output.stdout)?.into());
        res.insert("stderr".into(), str::from_utf8(&output.stderr)?.into());
        Ok(res)
    }
}
