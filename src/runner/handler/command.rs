use std::{
    process::{Command, Stdio},
    str::from_utf8,
};

use crate::runner::message::{
    action::{CreateAction, RunAction},
    result::RunResult,
    RunnerAction, RunnerResponse,
};

use super::{DynRunnerHandler, RunnerHandler};

pub struct CommandHandler;

impl CommandHandler {
    pub fn new() -> DynRunnerHandler {
        Box::new(Self)
    }

    fn handle_run_action(&self, run_action: RunAction) -> RunnerResponse {
        let command = &run_action.command;
        let mut cmd_iter = command.split(' ');
        let program = cmd_iter.next().unwrap();
        let args: Vec<&str> = cmd_iter.collect();

        let mut cmd = Command::new(program);
        cmd.args(args);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let process = match cmd.spawn() {
            Ok(child) => child,
            Err(_) => return RunnerResponse::CommandNotFound(command.into()),
        };

        let output = process.wait_with_output().unwrap();
        let mut stdout: String = from_utf8(&output.stdout).unwrap().into();
        if run_action.trim_stdout {
            stdout = stdout.trim().into();
        }
        let mut stderr: String = from_utf8(&output.stderr).unwrap().into();
        if run_action.trim_stderr {
            stderr = stderr.trim().into();
        }
        let status: String = output.status.code().unwrap().to_string();

        RunnerResponse::Output(RunResult {
            stdout,
            stderr,
            status,
        })
    }

    fn handle_create_action(&self, _create_action: CreateAction) -> RunnerResponse {
        return RunnerResponse::Created;
    }
}

#[allow(unreachable_patterns)]
impl RunnerHandler for CommandHandler {
    fn handle(&mut self, action: RunnerAction) -> RunnerResponse {
        match action {
            RunnerAction::Run(run_action) => self.handle_run_action(run_action),
            RunnerAction::Create(create_action) => self.handle_create_action(create_action),
            _ => panic!("received invalid action {:?}", &action),
        }
    }
}
