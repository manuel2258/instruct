use std::env;

use nom::error::ParseError;

use crate::interpreter::Interpreter;

mod interpreter;
mod parse;

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_path = &args[1];
    let task = &args[2];

    let ast = parse::load_and_parse(input_path);

    match ast {
        Ok(file) => {
            let interpreter = Interpreter::new(file);
            match interpreter.run_task(&task) {
                Ok(_) => println!("Successfully executed task {}", task),
                Err(err) => println!("Error while executing task {}:\n{:?}", task, err),
            }
        }
        Err(e) => println!("Could not parse {}:\n{}", input_path, e),
    }
}
