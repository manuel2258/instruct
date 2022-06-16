#[derive(Debug)]
pub struct RunnerRequest {
    pub runner_name: String,
    pub action: RunnerAction,
}

pub mod action {
    use std::collections::HashMap;

    #[derive(Debug)]
    pub struct RunAction {
        pub command: String,
        pub trim_stdout: bool,
        pub trim_stderr: bool,
    }

    #[derive(Debug)]
    pub struct CreateAction {
        pub runner_name: String,
        pub runner_type: String,
        pub args: HashMap<String, String>,
    }
}

#[derive(Debug)]
pub enum RunnerAction {
    Create(action::CreateAction),
    Run(action::RunAction),
}

pub mod result {
    #[derive(Debug, PartialEq)]
    pub struct RunResult {
        pub stdout: String,
        pub stderr: String,
        pub status: String,
    }
}

#[derive(Debug, PartialEq)]
pub enum RunnerResponse {
    Output(result::RunResult),
    Created,
    CommandNotFound(String),
    RunnerAlreadyExists(String),
    RunnerNotExisting(String),
    RunnerTypeNotExisting(String),
}
