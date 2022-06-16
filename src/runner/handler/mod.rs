use super::message::{RunnerAction, RunnerResponse};

mod command;

pub trait RunnerHandler {
    fn handle(&mut self, action: RunnerAction) -> RunnerResponse;
}

pub type DynRunnerHandler = Box<dyn RunnerHandler>;

pub fn create_new(runner_type: &str) -> Option<DynRunnerHandler> {
    match runner_type {
        "command" => Some(command::CommandHandler::new()),
        _ => None,
    }
}
