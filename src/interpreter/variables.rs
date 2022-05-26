use log::trace;

use crate::parse::ast::{VariableBinding, VariableBindings};

use super::stack::StackRef;

pub struct Variables {
    bindings: Option<VariableBindings>,
}

impl Variables {
    pub fn new(bindings: Option<VariableBindings>) -> Self {
        Self { bindings }
    }

    /// Checks whether all variables, based on the bindings, are allocated in the from_stack and allocates them in the to_stack.
    pub fn allocate_and_check_all(
        &self,
        to_stack: &mut StackRef,
        from_stack: &mut StackRef,
    ) -> anyhow::Result<()> {
        if let Some(bindings) = &self.bindings {
            for output in &bindings.bindings {
                let (parent_name, child_name) = match &output {
                    VariableBinding::Single(val) => (val, val),
                    VariableBinding::Dual(parent_var, child_var) => (parent_var, child_var),
                };
                from_stack.borrow().assert_allocated(child_name)?;
                to_stack.borrow_mut().allocate(parent_name.into());
            }
        }
        Ok(())
    }

    pub fn carry_over(
        &mut self,
        to_stack: &mut StackRef,
        from_stack: &mut StackRef,
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
                let value = from_stack.borrow().get(child_name)?;
                to_stack.borrow_mut().set(parent_name.into(), value)?;
            }
        }
        Ok(())
    }
}
