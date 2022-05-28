use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};

#[derive(Deserialize, Debug)]
pub struct Module {
    pub location: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub module: HashMap<String, Module>,
}

impl Config {
    pub fn load(path: Option<PathBuf>) -> anyhow::Result<Config> {
        let mut figment = Figment::new();
        figment = if let Some(overwrite_path) = path {
            figment.merge(Toml::file(overwrite_path))
        } else {
            figment
        };
        Ok(figment
            .join(Toml::file("instruct.toml"))
            .join(Env::prefixed("INSTRUCT_"))
            .extract()?)
    }
}
