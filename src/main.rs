use log::{error, info};
use task_lang;

fn main() {
    task_lang::logger::setup_logger().unwrap();
    match task_lang::run() {
        Ok(_) => info!("Successfully executed task!"),
        Err(err) => error!("{}", err),
    }
}
