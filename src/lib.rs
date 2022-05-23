use clap::Parser;
use log::error;
use thiserror::Error;

pub mod cli;
pub mod config;
pub mod interpreter;
pub mod logger;
pub mod parse;

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
}

fn print_err(error: &anyhow::Error) -> String {
    let mut repr: String = "\ncaused by:\n\n".into();
    for (counter, source) in error.chain().enumerate() {
        repr += &format!("\t#{}: {}\n", counter, source);
    }
    repr
}

pub fn run() -> Result<(), TaskLangError> {
    let cli = cli::Cli::parse();
    let config = config::Config::load(cli.task_file).map_err(TaskLangError::ConfigError)?;

    logger::setup_logger(&cli.log_level).unwrap();

    let task = &cli.task;

    let mut root_namespace = interpreter::RootNamespace::default();

    for (name, module) in &config.module {
        let location = &module.location;
        let namespace = parse::load_and_parse(location)
            .map_err(|err| TaskLangError::ParserError(name.into(), location.into(), err))?;
        root_namespace
            .add_root(namespace)
            .map_err(|err| TaskLangError::NamespaceError(name.into(), location.into(), err))?;
    }

    let mut interpreter = interpreter::Interpreter::new(root_namespace);
    interpreter
        .resolve(task)
        .map_err(|err| TaskLangError::ResolveError(task.into(), err))?;
    interpreter
        .initialize()
        .map_err(|err| TaskLangError::StaticAnalysisError(task.into(), err))?;
    interpreter
        .run()
        .map_err(|err| TaskLangError::ExecutionError(task.into(), err))?;

    Ok(())
}
