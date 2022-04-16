use std::io;
use std::{fs::File, io::Read};
use thiserror::Error;

use nom::error::{convert_error, VerboseError};
use nom::Err;

pub mod ast;
mod combinator;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("couldn't load file '{0}': {1}")]
    FileNotFound(String, io::Error),
    #[error("couldn't read file '{0}' as valid utf-8: {1}")]
    InvalidFileContent(String, io::Error),
    #[error("syntax error in task file: {0}")]
    SyntaxError(String),
}

/*pub fn load_and_parse(path: &str) -> Result<ast::File, ParseError> {
    let mut content = String::new();
    let mut content_path = match File::open(path) {
        Ok(val) => val,
        Err(e) => return Err(ParseError::FileNotFound(path.into(), e)),
    };
    match content_path.read_to_string(&mut content) {
        Ok(_) => (),
        Err(e) => return Err(ParseError::InvalidFileContent(path.into(), e)),
    };

    match executeable::file::<VerboseError<&str>>(&content) {
        Ok((_, ast)) => Ok(ast),
        Err(Err::Error(e)) | Err(Err::Failure(e)) => {
            Err(ParseError::SyntaxError(convert_error(content.as_str(), e)))
        }
        Err(e) => panic!("invalid state! incomplete: {:?}", e),
    }
}*/
