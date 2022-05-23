use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Namespace {
    pub name: String,
    pub namespace_type: NamespaceType,
    pub children: HashMap<String, NamespaceOrExecuteable>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamespaceType {
    Module,
    Collection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamespaceOrExecuteable {
    Namespace(Namespace),
    Executeable(Executeable),
}

impl NamespaceOrExecuteable {
    pub fn get_name(&self) -> &str {
        match self {
            NamespaceOrExecuteable::Executeable(executeable) => &executeable.name,
            NamespaceOrExecuteable::Namespace(namespace) => &namespace.name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Executeable {
    pub output_variables: Option<VariableBindings>,
    pub name: String,
    pub options: Option<VariableBindings>,
    pub executeable_type: ExecuteableType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariableBindings {
    pub bindings: Vec<VariableBinding>,
}

impl VariableBindings {
    pub fn find(&self, name: &str) -> Option<&str> {
        for binding in &self.bindings {
            match binding {
                VariableBinding::Single(binding_name) if binding_name == name => {
                    return Some(binding_name)
                }
                VariableBinding::Dual(binding_name, value) if binding_name == name => {
                    return Some(value)
                }
                _ => (),
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExecuteableType {
    Command { cmd: String },
    Call { target: String },
    Block { executeables: Vec<Executeable> },
    Task { executeables: Vec<Executeable> },
}
