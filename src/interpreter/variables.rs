use crate::parse::ast::{VariableBinding, VariableBindings};

use crate::interpreter::stack::Stack;

pub struct Variables {
    bindings: Option<VariableBindings>,
}

impl Variables {
    pub fn new(bindings: Option<VariableBindings>) -> Self {
        Self { bindings }
    }

    pub fn allocate_and_check_all(
        &self,
        stack: &mut Stack,
        check_output_var_valid: &dyn Fn(&str) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        if let Some(bindings) = &self.bindings {
            for output in &bindings.bindings {
                let name: &str = match &output {
                    VariableBinding::Single(val) => val,
                    VariableBinding::Dual(val, input_var) => {
                        check_output_var_valid(input_var)?;
                        val
                    }
                };
                stack.allocate(name.into());
            }
        }
        Ok(())
    }

    pub fn set_value(
        &mut self,
        stack: &mut Stack,
        name: &'static str,
        value: String,
    ) -> anyhow::Result<()> {
        let output_name = match self.get_output_var_name_for(name) {
            Some(val) => val,
            None => return Ok(()),
        };

        stack.set(output_name.into(), value);

        Ok(())
    }

    pub fn get_output_var_name_for(&mut self, input_name: &'static str) -> Option<&str> {
        if let Some(bindings) = &self.bindings {
            for output in &bindings.bindings {
                let (output, input) = match output {
                    VariableBinding::Single(val) => (val, val),
                    VariableBinding::Dual(out, input) => (out, input),
                };
                if input == input_name {
                    return Some(&output);
                }
            }
        }
        None
    }
}
