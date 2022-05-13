use anyhow::Context;
use thiserror::Error;

use crate::parse::ast::{Executeable, Namespace, NamespaceOrExecuteable};

#[derive(Error, Debug)]
pub enum NamespaceError {
    #[error("The referenced namespace could not be found")]
    NotFound,
    #[error("The referenced namespace '{0}' is a executeable")]
    ContainedExecuteable(String),
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
        if name_parts.get(0) != Some(&self.namespace.name.as_ref()) {
            return Err(NamespaceError::NotFound.into());
        }

        let current_part: &str = match name_parts.get(1) {
            Some(val) => *val,
            None => return Err(NamespaceError::NotFound.into()),
        };

        match self.namespace.children.get(current_part) {
            Some(NamespaceOrExecuteable::Namespace(next)) => Ok(NamespaceResolver::new(next)
                .resolve(&name_parts[1..])
                .with_context(|| {
                    format!("at searching {} in {}", current_part, &self.namespace.name)
                })?),
            Some(NamespaceOrExecuteable::Executeable(executeable)) => {
                if name_parts.len() > 2 {
                    return Err(NamespaceError::ContainedExecuteable(current_part.into()).into());
                }
                Ok(executeable)
            }
            None => Err(NamespaceError::NotFound.into()),
        }
    }
}
