use std::env;

use nom::error::ParseError;

mod interpreter;
mod parse;

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_path = &args[1];
    let task = &args[2];

    let ast = parse::load_and_parse(input_path);

    println!("parsed into: {:?}", ast);

    match ast {
        Ok(file) => interpreter::execute(file, task),
        Err(e) => println!("Could not parse {}:\n{}", input_path, e),
    }
}
