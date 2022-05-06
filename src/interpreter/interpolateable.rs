use crate::interpreter::stack::Stack;
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, PartialEq)]
enum InterpolateableAfter {
    Other(Box<Interpolateable>),
    Value(String),
    None,
}

#[derive(Debug, PartialEq)]
pub struct Interpolateable {
    before: String,
    after: InterpolateableAfter,
    variable_name: String,
}

impl Interpolateable {
    pub fn new(value: &str) -> Option<Self> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"\$\{(?P<variable_name>(\w|_)+)\}").unwrap();
        }
        match RE.captures(value) {
            Some(capture) => {
                let variable_name = capture.name("variable_name").unwrap();

                // Mask out ${}
                let before_variable = &value[..variable_name.start() - 2];
                let after_variable = &value[variable_name.end() + 1..];

                let next = Interpolateable::new(after_variable);
                let after = match next {
                    Some(other) => InterpolateableAfter::Other(Box::new(other)),
                    None => InterpolateableAfter::Value(after_variable.into()),
                };

                Some(Self {
                    before: before_variable.into(),
                    after,
                    variable_name: variable_name.as_str().into(),
                })
            }
            None => None,
        }
    }

    pub fn interpolate(&self, stack: &Stack, target: &mut String) -> anyhow::Result<()> {
        target.push_str(&self.before);
        target.push_str(stack.get(&self.variable_name)?);

        match &self.after {
            InterpolateableAfter::Other(other) => other.interpolate(stack, target)?,
            InterpolateableAfter::Value(after) => target.push_str(after),
            InterpolateableAfter::None => (),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::stack::Stack;

    use super::Interpolateable;

    #[test]
    fn empty() {
        let value = "".to_owned();

        let interpolateable = Interpolateable::new(&value);

        assert_eq!(interpolateable, None);
    }

    #[test]
    fn without_variable() {
        let value = "value_without_var".to_owned();

        let interpolateable = Interpolateable::new(&value);

        assert_eq!(interpolateable, None);
    }

    #[test]
    fn single_variable_at_end() {
        let value = "value_with_${var}".to_owned();

        let interpolateable = Interpolateable::new(&value).unwrap();

        let stack: Stack = vec![("var", "test-value")].into();

        let mut output = String::new();
        interpolateable.interpolate(&stack, &mut output).unwrap();

        assert_eq!(&output, "value_with_test-value");
    }

    #[test]
    fn single_variable_in_middle() {
        let value = "value_with_${var}_variable".to_owned();

        let interpolateable = Interpolateable::new(&value).unwrap();

        let stack: Stack = vec![("var", "val")].into();

        let mut output = String::new();
        interpolateable.interpolate(&stack, &mut output).unwrap();

        assert_eq!(&output, "value_with_val_variable");
    }

    #[test]
    fn multiple_variable_in_middle() {
        let value = "value_with_${var1}_multiple_${var2}_variable".to_owned();

        let interpolateable = Interpolateable::new(&value).unwrap();

        let stack: Stack = vec![("var1", "val1"), ("var2", "val2")].into();

        let mut output = String::new();
        interpolateable.interpolate(&stack, &mut output).unwrap();

        assert_eq!(&output, "value_with_val1_multiple_val2_variable");
    }
}
