use std::env;

use fern::{
    self,
    colors::{Color, ColoredLevelConfig},
};
use log::{error, info, warn, LevelFilter};

use crate::interpreter::Interpreter;

mod interpreter;
mod parse;

fn setup_logger() -> anyhow::Result<()> {
    let colors = ColoredLevelConfig::new()
        .error(Color::BrightRed)
        .warn(Color::Magenta)
        .info(Color::Blue)
        .debug(Color::BrightBlack);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "\x1B[{}m{}\x1B[{}m",
                colors.get_color(&record.level()).to_fg_str(),
                message,
                Color::White.to_fg_str()
            ))
        })
        .level(LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}

fn main() {
    setup_logger().unwrap();
    let args: Vec<String> = env::args().collect();
    let input_path = &args[1];
    let task = &args[2];

    let ast = parse::load_and_parse(input_path);

    match ast {
        Ok(file) => {
            let interpreter = Interpreter::new(file);
            match interpreter.run_task(&task) {
                Ok(_) => warn!("Successfully executed task {}", task),
                Err(err) => error!("Error while executing task {}:\n{:?}", task, err),
            }
        }
        Err(e) => error!("Could not parse {}:\n{}", input_path, e),
    }
}
