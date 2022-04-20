use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(
        "a variable '{0}' that was allocated but not set was accessed, this should not happen!"
    )]
    UnsetVariableAccessed(String),
    #[error("a variable '{0}' that not allocated was accessed, this should not happen!")]
    UnallocatedVariableAccessed(String),
    #[error("tried to access undefined variable '{0}'")]
    UndefinedVariableAccessed(String),
}

type Result<T> = std::result::Result<T, Error>;

pub struct Stack<'a> {
    variables: HashMap<String, Option<String>>,
    child: Option<&'a Stack<'a>>,
}

impl<'a> Stack<'a> {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            child: None,
        }
    }

    pub fn pop_new(&'a self) -> Stack<'a> {
        Self {
            variables: HashMap::new(),
            child: Some(self),
        }
    }

    pub fn get(&'a self, name: &str) -> Result<&'a str> {
        match self.variables.get(name) {
            Some(Some(val)) => Ok(val),
            Some(None) => Err(Error::UnsetVariableAccessed(name.into())),
            None => match self.child {
                Some(child) => child.get(name),
                None => Err(Error::UnallocatedVariableAccessed(name.into())),
            },
        }
    }

    pub fn set(&mut self, name: String, value: String) -> Result<()> {
        if let None = self.variables.insert(name.clone(), Some(value)) {
            return Err(Error::UnallocatedVariableAccessed(name.into()));
        }
        Ok(())
    }

    pub fn allocate(&mut self, name: String) {
        self.variables.insert(name, None);
    }

    pub fn assert_allocated(&mut self, name: &str) -> Result<()> {
        match self.variables.contains_key(name) {
            true => Ok(()),
            false => Err(Error::UndefinedVariableAccessed(name.into())),
        }
    }
}

impl<'a> From<Vec<(&'static str, &'static str)>> for Stack<'a> {
    fn from(values: Vec<(&str, &str)>) -> Stack<'a> {
        let mut stack = Stack::new();

        for (key, value) in values {
            stack.allocate(key.into());
            stack.set(key.into(), value.into()).unwrap();
        }

        stack
    }
}
