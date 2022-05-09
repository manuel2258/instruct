use thiserror::Error;

use crate::parse::ast::{Executeable, Namespace, NamespaceType};

#[derive(Error, Debug)]
pub enum NamespaceError {
    #[error("The referenced executeable is a collection")]
    FoundCollection,
    #[error("The referenced executeable could not be found")]
    ExecutableNotFound,
}

#[derive(Debug)]
pub struct NamespaceNode {
    namespace: Namespace,
    children: Vec<NamespaceChild>,
}

#[derive(Debug)]
pub enum NamespaceChild {
    SubNamespace(NamespaceNode),
    Executeable(Executeable),
}

impl NamespaceNode {
    pub fn new(namespace: Namespace) -> Self {
        match &namespace.namespace_type {
            NamespaceType::Task { execs } => Self {
                namespace: namespace.clone(),
                children: execs
                    .iter()
                    .map(|exec| NamespaceChild::Executeable(exec.clone()))
                    .collect(),
            },
            NamespaceType::Collection { namespaces } | NamespaceType::Module { namespaces } => {
                Self {
                    namespace: namespace.clone(),
                    children: namespaces
                        .iter()
                        .map(|namespace| {
                            NamespaceChild::SubNamespace(NamespaceNode::new(namespace.clone()))
                        })
                        .collect(),
                }
            }
        }
    }

    pub fn find_executeable(&self, name_parts: &[&str]) -> anyhow::Result<Option<&Executeable>> {
        match name_parts.len() {
            0 => Err(NamespaceError::ExecutableNotFound.into()),
            1 => {
                if let NamespaceType::Task { .. } = self.namespace.namespace_type && self.namespace.name == name_parts[0] {
                    Ok()
                } else {
                    Err(NamespaceError::ExecutableNotFound.into())
                }
            }
            _ => {
                for child in &self.children {
                    if let Some(val) = child.find_executeable(&name_parts[1..])? {
                        return Ok(Some(val));
                    }
                }
                Err(NamespaceError::ExecutableNotFound.into())
            }
        }
    }
}

impl NamespaceChild {
    pub fn find_executeable(&self, name_parts: &[&str]) -> anyhow::Result<Option<&Executeable>> {
        match self {
            NamespaceChild::SubNamespace(node) => node.find_executeable(name_parts),
            NamespaceChild::Executeable(exe) => {
                if let Some(exe_name) = &exe.name {
                    if name_parts.len() == 1 && exe_name == name_parts[0] {
                        Ok(Some(exe))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
        }
    }
}
