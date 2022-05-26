use std::collections::HashMap;

use anyhow::Context;
use thiserror::Error;

use crate::parse::ast::{Executeable, Namespace, NamespaceOrExecuteable};

#[derive(Error, Debug, PartialEq, Eq)]
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

#[derive(Error, Debug)]
pub enum RootNamespaceError {
    #[error("The module name '{0}' is already used")]
    ModuleNameAlreadyUsed(String),
    #[error("A module with the name '{0}' could not be found")]
    ModuleNotFound(String),
    #[error("Tried to search an empty name, this should not happen")]
    EmptySearchName,
}

#[derive(Clone)]
pub struct RootNamespace {
    namespaces: HashMap<String, Namespace>,
}

impl RootNamespace {
    pub fn new() -> RootNamespace {
        RootNamespace {
            namespaces: HashMap::new(),
        }
    }

    pub fn add_root(&mut self, namespace: Namespace) -> anyhow::Result<()> {
        if self.namespaces.contains_key(&namespace.name) {
            return Err(RootNamespaceError::ModuleNameAlreadyUsed(namespace.name).into());
        }
        assert!(self
            .namespaces
            .insert(namespace.name.clone(), namespace)
            .is_none());

        Ok(())
    }

    pub fn resolve_name(&self, target_name: &str) -> anyhow::Result<&Executeable> {
        let target_name_vec: Vec<&str> = target_name.split('.').collect();
        self.resolve(&target_name_vec)
    }

    pub fn resolve(&self, name_parts: &[&str]) -> anyhow::Result<&Executeable> {
        if let Some(module_name) = name_parts.get(0) {
            match self.namespaces.get(*module_name) {
                Some(namespace) => NamespaceResolver::new(namespace).resolve(name_parts),
                None => Err(RootNamespaceError::ModuleNotFound((*module_name).into()).into()),
            }
        } else {
            Err(RootNamespaceError::EmptySearchName.into())
        }
    }
}

impl Default for RootNamespace {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::namespace::NamespaceError;
    use crate::parse::ast::Executeable;
    use crate::parse::ast::ExecuteableType;
    use crate::parse::ast::Namespace;
    use crate::parse::ast::NamespaceOrExecuteable;
    use crate::parse::ast::NamespaceType;

    use super::NamespaceResolver;

    fn get_collection(name: &'static str, mut children: Vec<NamespaceOrExecuteable>) -> Namespace {
        Namespace {
            name: name.into(),
            namespace_type: NamespaceType::Collection,
            children: children
                .drain(..)
                .map(|val| (val.get_name().to_owned(), val))
                .collect(),
        }
    }

    fn get_executeable(name: &'static str) -> NamespaceOrExecuteable {
        NamespaceOrExecuteable::Executeable(Executeable {
            output_variables: None,
            name: name.into(),
            options: None,
            executeable_type: ExecuteableType::Command { cmd: "".into() },
        })
    }

    fn split(value: &'static str) -> Vec<&str> {
        value.split(".").collect()
    }

    #[test]
    fn ok_resolve_1_depth() {
        let namespace = get_collection("root", vec![get_executeable("task")]);

        let name = split("root.task");
        let res = NamespaceResolver::new(&namespace).resolve(&name);

        assert!(res.is_ok());
        let task = res.unwrap();

        assert_eq!(&task.name, "task");
    }

    #[test]
    fn ok_resolve_1_depth_multiple_childs() {
        let namespace = get_collection(
            "root",
            vec![get_executeable("task"), get_executeable("other-task")],
        );

        let name = split("root.task");
        let res = NamespaceResolver::new(&namespace).resolve(&name);

        assert!(res.is_ok());
        let task = res.unwrap();

        assert_eq!(&task.name, "task");
    }

    #[test]
    fn ok_resolve_2_depth() {
        let namespace = get_collection(
            "root",
            vec![NamespaceOrExecuteable::Namespace(get_collection(
                "collection",
                vec![get_executeable("task")],
            ))],
        );

        let name = split("root.collection.task");
        let res = NamespaceResolver::new(&namespace).resolve(&name);

        assert!(res.is_ok());
        let task = res.unwrap();

        assert_eq!(&task.name, "task");
    }

    #[test]
    fn ok_resolve_4_depth() {
        let namespace = get_collection(
            "root",
            vec![NamespaceOrExecuteable::Namespace(get_collection(
                "collection1",
                vec![NamespaceOrExecuteable::Namespace(get_collection(
                    "collection2",
                    vec![NamespaceOrExecuteable::Namespace(get_collection(
                        "collection3",
                        vec![get_executeable("task")],
                    ))],
                ))],
            ))],
        );

        let name = split("root.collection1.collection2.collection3.task");
        let res = NamespaceResolver::new(&namespace).resolve(&name);

        assert!(res.is_ok());
        let task = res.unwrap();

        assert_eq!(&task.name, "task");
    }

    #[test]
    fn ok_resolve_4_depth_multiple_childs() {
        let namespace = get_collection(
            "root",
            vec![
                get_executeable("other-task"),
                NamespaceOrExecuteable::Namespace(get_collection(
                    "collection1",
                    vec![
                        NamespaceOrExecuteable::Namespace(get_collection(
                            "collection2",
                            vec![
                                get_executeable("other-task"),
                                get_executeable("another-task"),
                                NamespaceOrExecuteable::Namespace(get_collection(
                                    "collection3",
                                    vec![
                                        get_executeable("other-task"),
                                        get_executeable("another-task"),
                                        get_executeable("task"),
                                    ],
                                )),
                            ],
                        )),
                        get_executeable("other-task"),
                    ],
                )),
            ],
        );

        let name = split("root.collection1.collection2.collection3.task");
        let res = NamespaceResolver::new(&namespace).resolve(&name);

        assert!(res.is_ok());
        let task = res.unwrap();

        assert_eq!(&task.name, "task");
    }

    #[test]
    fn nok_task_is_collection() {
        let namespace = get_collection(
            "root",
            vec![
                get_executeable("other-task"),
                NamespaceOrExecuteable::Namespace(get_collection("collection", vec![])),
            ],
        );

        let name = split("root.collection");
        let res = NamespaceResolver::new(&namespace).resolve(&name);

        assert!(res.is_err());

        assert_eq!(
            res.unwrap_err().downcast::<NamespaceError>().unwrap(),
            NamespaceError::NotAExecuteable("collection".into())
        );
    }

    #[test]
    fn nok_collection_is_executeable() {
        let namespace = get_collection("root", vec![get_executeable("collection")]);

        let name = split("root.collection.task");
        let res = NamespaceResolver::new(&namespace).resolve(&name);

        assert!(res.is_err());

        assert_eq!(
            res.unwrap_err().downcast::<NamespaceError>().unwrap(),
            NamespaceError::NotANamespace("collection".into())
        );
    }

    #[test]
    fn nok_not_found_other_name() {
        let namespace = get_collection("root", vec![get_executeable("other-task")]);

        let name = split("root.task");
        let res = NamespaceResolver::new(&namespace).resolve(&name);

        assert!(res.is_err());

        assert_eq!(
            res.unwrap_err().downcast::<NamespaceError>().unwrap(),
            NamespaceError::NotFound("task".into(), "root".into())
        );
    }

    #[test]
    fn nok_not_found_empty() {
        let namespace = get_collection("root", vec![]);

        let name = split("root.task");
        let res = NamespaceResolver::new(&namespace).resolve(&name);

        assert!(res.is_err());

        assert_eq!(
            res.unwrap_err().downcast::<NamespaceError>().unwrap(),
            NamespaceError::NotFound("task".into(), "root".into())
        );
    }
}
