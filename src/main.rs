use log::{error, info};

fn main() {
    match instruct::run() {
        Ok(_) => info!("Successfully executed task!"),
        Err(err) => error!("{}", err),
    }
}
