use std::{cell::RefCell, rc::Rc};

use super::RootNamespace;

pub type ContextRef = Rc<RefCell<Context>>;

pub struct Context {
    pub root_namespace: RootNamespace,
}

impl Context {
    pub fn new(root_namespace: RootNamespace) -> Self {
        Self { root_namespace }
    }
}

impl From<Context> for ContextRef {
    fn from(ctx: Context) -> Self {
        Rc::new(RefCell::new(ctx))
    }
}
