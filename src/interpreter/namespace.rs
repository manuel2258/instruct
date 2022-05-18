use anyhow::Context;
use thiserror::Error;

use crate::parse::ast::{Executeable, Namespace, NamespaceOrExecuteable};

#[derive(Error, Debug)]
pub enum NamespaceError {
    #[error("searching '{0}' as '{1}' is invalid and should never happen!")]
    InvalidSearch(String, String),
    #[error("could not find namespace '{0}' in '{1}'")]
    NotFound(String, String),
    #[error("the referenced namespace '{0}' is a executeable and therefor not a namespace")]
    NotANamespace(String),
    #[error("the referenced task '{0}' is not a executeable")]
    NotAExecuteable(String),
}

#[derive(Debug)]
pub struct NamespaceResolver<'a> {
    namespace: &'a Namespace,
}

impl<'a> NamespaceResolver<'a> {
    pub fn new(namespace: &'a Namespace) -> Self {
        NamespaceResolver { namespace }
    }

    pub fn resolve(&self, name_parts: &[&str]) -> anyhow::Result<&'a Executeable> {
        let current_part: &str = match name_parts.get(0) {
            Some(val) => *val,
            None => return Err(NamespaceError::NotAExecuteable(self.namespace.name.clone()).into()),
        };

        if current_part != self.namespace.name {
            return Err(NamespaceError::InvalidSearch(
                current_part.into(),
                self.namespace.name.clone(),
            )
            .into());
        }

        let next_part: &str = match name_parts.get(1) {
            Some(val) => *val,
            None => return Err(NamespaceError::NotAExecuteable(self.namespace.name.clone()).into()),
        };

        match self.namespace.children.get(next_part) {
            Some(NamespaceOrExecuteable::Namespace(next)) => Ok(NamespaceResolver::new(next)
                .resolve(&name_parts[1..])
                .with_context(|| {
                    format!("at searching '{}' in '{}'", next_part, &self.namespace.name)
                })?),
            Some(NamespaceOrExecuteable::Executeable(executeable)) => {
                if name_parts.len() > 2 {
                    return Err(NamespaceError::NotANamespace(next_part.into()).into());
                }
                Ok(executeable)
            }
            None => Err(NamespaceError::NotFound(next_part.into(), current_part.into()).into()),
        }
    }
}
