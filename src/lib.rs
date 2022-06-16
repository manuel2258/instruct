use std::thread::JoinHandle;

use clap::Parser;
use log::error;
use thiserror::Error;

pub mod cli;
pub mod config;
pub mod interpreter;
pub mod logger;
pub mod parse;
pub mod runner;
pub mod util;

#[derive(Error, Debug)]
pub enum TaskLangError {
    #[error(
        "Error while loading configuration (make sure the task.toml is valid!){}", print_err(.0)
    )]
    ConfigError(anyhow::Error),
    #[error("Error while parsing module '{0}' at '{1}'{}", print_err(.2))]
    ParserError(String, String, anyhow::Error),
    #[error("Error while adding module '{0}' at '{1}'{}", print_err(.2))]
    NamespaceError(String, String, anyhow::Error),
    #[error("Error while resolving task '{0}'{}", print_err(.1))]
    ResolveError(String, anyhow::Error),
    #[error("Error while analising task '{0}'{}", print_err(.1))]
    StaticAnalysisError(String, anyhow::Error),
    #[error("Error while executing task '{0}'{}", print_err(.1))]
    ExecutionError(String, anyhow::Error),
    #[error("Error in the runner thread: {}", print_err(.0))]
    RunnerThreadPanic(anyhow::Error),
}

fn print_err(error: &anyhow::Error) -> String {
    let mut repr: String = "\ncaused by:\n\n".into();
    for (counter, source) in error.chain().enumerate() {
        repr += &format!("\t#{}: {}\n", counter, source);
    }
    repr
}

fn create_runner_thread() -> (
    util::channel::TwoWayChannel<runner::message::RunnerRequest, runner::message::RunnerResponse>,
    JoinHandle<()>,
) {
    let (runner_requester, runner_responder) = util::channel::TwoWayChannel::new_pair();
    let runner_server = runner::server::RunnerServer::new_thread(runner_responder);

    (runner_requester, runner_server)
}

fn parse_root_namespace(
    config: &config::Config,
) -> Result<interpreter::RootNamespace, TaskLangError> {
    let mut root_namespace = interpreter::RootNamespace::default();

    for (name, module) in &config.module {
        let location = &module.location;
        let namespace = parse::load_and_parse(location)
            .map_err(|err| TaskLangError::ParserError(name.into(), location.into(), err))?;
        root_namespace
            .add_root(namespace)
            .map_err(|err| TaskLangError::NamespaceError(name.into(), location.into(), err))?;
    }

    Ok(root_namespace)
}

pub fn run() {
    let cli = cli::Cli::parse();
    let config = config::Config::load(cli.task_file).unwrap();

    logger::setup_logger(&cli.log_level).unwrap();

    let root_namespace = parse_root_namespace(&config).unwrap();

    let (runner_requester, runner_server) = create_runner_thread();

    let interpreter_thread = interpreter::Interpreter::run_as_new_thread(
        root_namespace,
        runner_requester,
        cli.task.clone(),
    );

    interpreter_thread.join().unwrap();
    runner_server.join().unwrap();
}
