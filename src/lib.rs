use std::{collections::HashMap, env, error::Error};

use clap::Parser;
use log::{error, warn};
use parse::ast::Namespace;
use thiserror::Error;

pub mod cli;
pub mod config;
pub mod interpreter;
pub mod logger;
pub mod parse;

#[derive(Error, Debug)]
pub enum TaskLangError {
    #[error("Error while loading configuration (make sure the task.toml is valid!):\n{0:?}")]
    ConfigError(anyhow::Error),
    #[error("Error while parsing module '{0}' at '{1}':\n{2:?}")]
    ParserError(String, String, anyhow::Error),
    #[error("Error while resolving task '{0}':\n{1:?}")]
    ResolveError(String, anyhow::Error),
    #[error("Error while analising task '{0}':\n{1:?}")]
    StaticAnalysisError(String, anyhow::Error),
    #[error("Error while executing task '{0}':\n{1:?}")]
    ExecutionError(String, anyhow::Error),
}

pub fn run() -> Result<(), TaskLangError> {
    let cli = cli::Cli::parse();
    let config =
        config::Config::load(cli.task_file).map_err(|err| TaskLangError::ConfigError(err))?;

    let task = &cli.task;

    let mut root_namespace = Namespace {
        name: "root".into(),
        children: HashMap::new(),
        namespace_type: parse::ast::NamespaceType::Collection,
    };

    for (name, module) in &config.module {
        let location = &module.location;
        let namespace = parse::load_and_parse(location)
            .map_err(|err| TaskLangError::ParserError(name.into(), location.into(), err))?;
        root_namespace.children.insert(
            name.into(),
            parse::ast::NamespaceOrExecuteable::Namespace(namespace),
        );
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
