use log::trace;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("a variable '{0}' that not allocated was accessed, this should not happen!")]
    UnallocatedVariableAccessed(String),
    #[error("tried to access undefined variable '{0}'")]
    UndefinedVariableAccessed(String),
}

pub type StackRef = Rc<RefCell<Stack>>;

pub struct Stack {
    variables: HashMap<String, Option<String>>,
    parent: Option<StackRef>,
    height: u16,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
            height: 0,
        }
    }

    pub fn inherit_new(parent: &StackRef) -> Stack {
        Self {
            variables: HashMap::new(),
            parent: Some(parent.clone()),
            height: parent.borrow().height + 1,
        }
    }

    pub fn get(&self, name: &str) -> anyhow::Result<String> {
        trace!("Getting '{}' from stack {}@{:p}", name, self.height, self);
        match self.variables.get(name) {
            Some(Some(val)) => Ok(val.into()),
            Some(None) | None => match &self.parent {
                Some(child) => child.borrow().get(name),
                None => Err(Error::UnallocatedVariableAccessed(name.into()).into()),
            },
        }
    }

    pub fn set(&mut self, name: String, value: String) -> anyhow::Result<()> {
        if value.len() > 10 {
            trace!(
                "Setting '{}' to {:?}..{:?}' for stack {}@{:p}",
                &name,
                &value[..10],
                &value[value.len() - 10..],
                self.height,
                self
            );
        } else {
            trace!(
                "Setting '{}' to {:?} for stack {}@{:p}",
                &name,
                &value,
                self.height,
                self
            );
        }
        if self.variables.insert(name.clone(), Some(value)).is_none() {
            return Err(Error::UnallocatedVariableAccessed(name).into());
        }
        Ok(())
    }

    pub fn allocate(&mut self, name: String) {
        trace!(
            "Allocating '{}' for stack {}@{:p}",
            &name,
            self.height,
            self
        );
        self.variables.insert(name, None);
    }

    pub fn assert_allocated(&self, name: &str) -> anyhow::Result<()> {
        trace!(
            "Asserting allocation '{}' for stack {}@{:p}",
            &name,
            self.height,
            self
        );
        match self.variables.contains_key(name) {
            true => Ok(()),
            false => match &self.parent {
                Some(parent) => parent.borrow().assert_allocated(name),
                None => Err(Error::UndefinedVariableAccessed(name.into()).into()),
            },
        }
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<(&'static str, &'static str)>> for Stack {
    fn from(values: Vec<(&str, &str)>) -> Stack {
        let mut stack = Stack::new();

        for (key, value) in values {
            stack.allocate(key.into());
            stack.set(key.into(), value.into()).unwrap();
        }

        stack
    }
}

impl From<Stack> for StackRef {
    fn from(stack: Stack) -> StackRef {
        Rc::new(RefCell::new(stack))
    }
}
