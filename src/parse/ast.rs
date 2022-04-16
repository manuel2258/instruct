use std::collections::HashMap;

#[derive(Debug)]
pub struct File {
    pub tasks: Vec<Executeable>,
}

impl File {
    pub fn get_task(&self, name: &str) -> Option<Executeable> {
        match self.tasks.iter().find(|task| match &task.name {
            Some(val) => val == name,
            None => false,
        }) {
            Some(task) => Some(task.clone()),
            None => None,
        }
    }
}

pub struct Task {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Executeable {
    pub output_variables: Option<VariableBindings>,
    pub name: Option<String>,
    pub options: Option<VariableBindings>,
    pub exec_type: ExecuteableType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableBinding {
    Single(String),
    Dual(String, String),
}

impl From<&str> for VariableBinding {
    fn from(val: &str) -> Self {
        Self::Single(val.into())
    }
}

impl From<(&str, &str)> for VariableBinding {
    fn from(val: (&str, &str)) -> Self {
        Self::Dual(val.0.into(), val.1.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableBindings {
    pub bindings: Vec<VariableBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecuteableType {
    Command { cmd: String },
    Call { target: String },
    ShBlock { execs: Vec<Executeable> },
}
