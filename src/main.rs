use log::{error, info};

fn main() {
    match task_lang::run() {
        Ok(_) => info!("Successfully executed task!"),
        Err(err) => error!("{}", err),
    }
}
