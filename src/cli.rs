use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap()]
    pub task: String,

    #[clap(short, long)]
    pub log_level: Option<String>,

    #[clap(short, long)]
    pub task_file: Option<PathBuf>,
}
