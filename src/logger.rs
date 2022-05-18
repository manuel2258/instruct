use std::env;

use clap::Parser;
use fern::{
    self,
    colors::{Color, ColoredLevelConfig},
};
use log::{error, warn, Level, LevelFilter};

pub fn setup_logger() -> anyhow::Result<()> {
    let colors = ColoredLevelConfig::new()
        .error(Color::BrightRed)
        .warn(Color::Magenta)
        .info(Color::Blue)
        .debug(Color::BrightBlack)
        .trace(Color::BrightCyan);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            if record.level() == Level::Trace {
                out.finish(format_args!(
                    "\x1B[{}m[{}] {}\x1B[{}m",
                    colors.get_color(&record.level()).to_fg_str(),
                    record.target(),
                    message,
                    Color::White.to_fg_str()
                ))
            } else {
                out.finish(format_args!(
                    "\x1B[{}m{}\x1B[{}m",
                    colors.get_color(&record.level()).to_fg_str(),
                    message,
                    Color::White.to_fg_str()
                ))
            }
        })
        .level(LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}
