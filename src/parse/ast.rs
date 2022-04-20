use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Namespace {
    pub name: String,
    pub ns_type: NamespaceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamespaceType {
    Module { namespaces: Vec<Namespace> },
    Collection { namespaces: Vec<Namespace> },
    Task { execs: Vec<Executeable> },
}

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
    Block { execs: Vec<Executeable> },
}
