use std::{cell::RefCell, rc::Rc};

use crate::{
    runner::{
        interface::RunnerInterface,
        message::{RunnerRequest, RunnerResponse},
    },
    util::channel::TwoWayChannel,
};

use super::RootNamespace;

pub type ContextRef = Rc<RefCell<Context>>;
pub type RunnerRequester = TwoWayChannel<RunnerRequest, RunnerResponse>;

pub struct Context {
    pub root_namespace: RootNamespace,
    pub runner: RunnerInterface,
}

impl Context {
    pub fn new(root_namespace: RootNamespace, runner_requester: RunnerRequester) -> Self {
        Self {
            root_namespace,
            runner: RunnerInterface::new(runner_requester),
        }
    }
}

impl From<Context> for ContextRef {
    fn from(ctx: Context) -> Self {
        Rc::new(RefCell::new(ctx))
    }
}
