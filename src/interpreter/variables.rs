use log::trace;

use crate::parse::ast::{VariableBinding, VariableBindings};

use super::stack::RcStack;

pub struct Variables {
    bindings: Option<VariableBindings>,
}

impl Variables {
    pub fn new(bindings: Option<VariableBindings>) -> Self {
        Self { bindings }
    }

    pub fn allocate_and_check_all(
        &self,
        parent_stack: &mut RcStack,
        child_stack: &mut RcStack,
    ) -> anyhow::Result<()> {
        if let Some(bindings) = &self.bindings {
            for output in &bindings.bindings {
                let (parent_name, child_name) = match &output {
                    VariableBinding::Single(val) => (val, val),
                    VariableBinding::Dual(parent_var, child_var) => (parent_var, child_var),
                };
                child_stack.borrow().assert_allocated(child_name)?;
                parent_stack.borrow_mut().allocate(parent_name.into());
            }
        }
        Ok(())
    }

    pub fn carry_over(
        &mut self,
        parent_stack: &mut RcStack,
        child_stack: &mut RcStack,
    ) -> anyhow::Result<()> {
        if let Some(bindings) = &self.bindings {
            for output in &bindings.bindings {
                let (parent_name, child_name) = match &output {
                    VariableBinding::Single(val) => (val, val),
                    VariableBinding::Dual(parent_var, child_var) => (parent_var, child_var),
                };
                trace!(
                    "Carring over variable from '{}' to '{}'",
                    &child_name,
                    &parent_name
                );
                let value = child_stack.borrow().get(child_name)?;
                parent_stack.borrow_mut().set(parent_name.into(), value)?;
            }
        }
        Ok(())
    }
}
