use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap()]
    pub task: String,

    #[clap(short, long, default_value_t = 1)]
    pub log_level: u8,

    #[clap(short, long)]
    pub task_file: Option<PathBuf>,
}
