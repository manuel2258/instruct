use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    character::complete::{alpha1, alphanumeric1, space0},
    combinator::{not, recognize},
    error::ParseError,
    multi::{many0_count, separated_list1},
    sequence::{pair, preceded},
    IResult,
};

use crate::parse::ast::{VariableBinding, VariableBindings};

pub fn output_variable_bindings<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, VariableBindings, E> {
    let (i, _) = preceded(space0, tag("let"))(i)?;
    let (i, bindings) = variable_bindings(i)?;
    let (i, _) = preceded(space0, tag("from"))(i)?;
    Ok((i, bindings))
}

pub fn option_variable_bindings<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, VariableBindings, E> {
    let (i, _) = preceded(space0, tag("with"))(i)?;
    let (i, bindings) = variable_bindings(i)?;
    Ok((i, bindings))
}

fn variable_bindings<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, VariableBindings, E> {
    let (i, _) = preceded(space0, char('('))(i)?;
    let (i, bindings) = separated_list1(
        char(','),
        alt((single_variable_binding, dual_variable_binding)),
    )(i)?;
    let (i, _) = preceded(space0, char(')'))(i)?;
    Ok((i, VariableBindings { bindings }))
}

fn dual_variable_binding<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, VariableBinding, E> {
    let (i, output) = preceded(space0, variable)(i)?;
    let (i, _) = preceded(space0, char(':'))(i)?;
    let (i, input) = preceded(space0, variable)(i)?;
    Ok((i, VariableBinding::Dual(output.into(), input.into())))
}

fn single_variable_binding<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, VariableBinding, E> {
    let (i, input) = preceded(space0, variable)(i)?;
    not(preceded(space0, char(':')))(i)?;
    Ok((i, VariableBinding::Single(input.into())))
}

pub fn variable<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, String, E> {
    let (i, name) = recognize(pair(
        alt((alpha1, tag("_"), tag("-"))),
        many0_count(alt((alphanumeric1, tag("_"), tag("-")))),
    ))(i)?;

    Ok((i, name.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::Error;

    #[cfg(test)]
    mod variable {
        use super::*;

        #[test]
        fn ok_simple() {
            assert_eq!(variable::<Error<&str>>("x"), Ok(("", "x".into())));
        }

        #[test]
        fn ok_complex() {
            assert_eq!(
                variable::<Error<&str>>("var_with_complex_name1234"),
                Ok(("", "var_with_complex_name1234".into()))
            );
        }

        #[test]
        fn ok_complex_preceded_underscore() {
            assert_eq!(
                variable::<Error<&str>>("_var_with_complex_name"),
                Ok(("", "_var_with_complex_name".into()))
            );
        }

        #[test]
        fn nok_preceded_number() {
            assert!(variable::<Error<&str>>("1x").is_err());
        }

        #[test]
        fn nok_special_char1() {
            assert_eq!(
                variable::<Error<&str>>("var-name"),
                Ok(("", "var-name".into()))
            );
        }

        #[test]
        fn nok_special_char2() {
            assert_ne!(
                variable::<Error<&str>>("var()name"),
                Ok(("", "var()name".into()))
            );
        }
    }

    #[cfg(test)]
    mod single_variable_binding {
        use super::*;

        #[test]
        fn ok_simple() {
            assert_eq!(
                single_variable_binding::<Error<&str>>("x"),
                Ok(("", "x".into()))
            );
        }

        #[test]
        fn ok_preceded_spaces() {
            assert_eq!(
                single_variable_binding::<Error<&str>>("  x"),
                Ok(("", "x".into()))
            );
        }

        #[test]
        fn nok_dual_binding() {
            assert_ne!(
                single_variable_binding::<Error<&str>>("x: y"),
                Ok(("", ("x", "y").into()))
            );
        }

        #[test]
        fn nok_double_dot() {
            assert_ne!(
                single_variable_binding::<Error<&str>>("x:"),
                Ok(("", "x".into()))
            );
        }
    }

    #[cfg(test)]
    mod dual_variable_binding {
        use super::*;

        #[test]
        fn ok_simple() {
            assert_eq!(
                dual_variable_binding::<Error<&str>>("x: y"),
                Ok(("", ("x", "y").into()))
            );
        }

        #[test]
        fn ok_preceded_spaces() {
            assert_eq!(
                dual_variable_binding::<Error<&str>>("  x :  y"),
                Ok(("", ("x", "y").into()))
            );
        }

        #[test]
        fn nok_single_binding() {
            assert!(dual_variable_binding::<Error<&str>>("x").is_err());
        }

        #[test]
        fn nok_missing_right_side() {
            assert!(dual_variable_binding::<Error<&str>>("x:").is_err());
        }

        #[test]
        fn nok_missing_double_dot() {
            assert!(dual_variable_binding::<Error<&str>>("x y").is_err());
        }
    }

    #[cfg(test)]
    mod output_variable_bindings {
        use super::*;

        #[test]
        fn ok_simple() {
            assert_eq!(
                output_variable_bindings::<Error<&str>>("let (x: y) from"),
                Ok((
                    "",
                    VariableBindings {
                        bindings: vec!(("x", "y").into())
                    }
                ))
            );
        }

        #[test]
        fn ok_spaced() {
            assert_eq!(
                output_variable_bindings::<Error<&str>>("let   (  x  :  y)     from"),
                Ok((
                    "",
                    VariableBindings {
                        bindings: vec!(("x", "y").into())
                    }
                ))
            );
        }

        #[test]
        fn ok_no_spaces() {
            assert_eq!(
                output_variable_bindings::<Error<&str>>("let(x:y)from"),
                Ok((
                    "",
                    VariableBindings {
                        bindings: vec!(("x", "y").into())
                    }
                ))
            );
        }

        #[test]
        fn ok_multiple() {
            assert_eq!(
                output_variable_bindings::<Error<&str>>(
                    "let (x: y, stdout, stderr: input_var) from"
                ),
                Ok((
                    "",
                    VariableBindings {
                        bindings: vec!(
                            ("x", "y").into(),
                            "stdout".into(),
                            ("stderr", "input_var").into()
                        )
                    }
                ))
            );
        }

        #[test]
        fn nok_missing_double_dot() {
            assert_ne!(
                output_variable_bindings::<Error<&str>>("let (x y) from"),
                Ok((
                    "",
                    VariableBindings {
                        bindings: vec!(("x", "y").into())
                    }
                ))
            );
        }

        #[test]
        fn nok_missing_brackets() {
            assert_ne!(
                output_variable_bindings::<Error<&str>>("let (x: y from"),
                Ok((
                    "",
                    VariableBindings {
                        bindings: vec!(("x", "y").into())
                    }
                ))
            );
        }

        #[test]
        fn nok_missing_from() {
            assert_ne!(
                output_variable_bindings::<Error<&str>>("let (x: y)"),
                Ok((
                    "",
                    VariableBindings {
                        bindings: vec!(("x", "y").into())
                    }
                ))
            );
        }
    }

    #[cfg(test)]
    mod option_variable_bindings {
        use super::*;

        #[test]
        fn ok_simple() {
            assert_eq!(
                option_variable_bindings::<Error<&str>>("with (x: y)"),
                Ok((
                    "",
                    VariableBindings {
                        bindings: vec!(("x", "y").into())
                    }
                ))
            );
        }

        #[test]
        fn ok_spaced() {
            assert_eq!(
                option_variable_bindings::<Error<&str>>("with   (  x  :  y)"),
                Ok((
                    "",
                    VariableBindings {
                        bindings: vec!(("x", "y").into())
                    }
                ))
            );
        }

        #[test]
        fn ok_multiple() {
            assert_eq!(
                option_variable_bindings::<Error<&str>>("with (x: y, stdout, stderr: input_var)"),
                Ok((
                    "",
                    VariableBindings {
                        bindings: vec!(
                            ("x", "y").into(),
                            "stdout".into(),
                            ("stderr", "input_var").into()
                        )
                    }
                ))
            );
        }

        #[test]
        fn nok_missing_double_dot() {
            assert_ne!(
                option_variable_bindings::<Error<&str>>("with (x y)"),
                Ok((
                    "",
                    VariableBindings {
                        bindings: vec!(("x", "y").into())
                    }
                ))
            );
        }

        #[test]
        fn nok_missing_brackets() {
            assert_ne!(
                option_variable_bindings::<Error<&str>>("with (x: y"),
                Ok((
                    "",
                    VariableBindings {
                        bindings: vec!(("x", "y").into())
                    }
                ))
            );
        }

        #[test]
        fn nok_missing_with() {
            assert_ne!(
                option_variable_bindings::<Error<&str>>("(x: y)"),
                Ok((
                    "",
                    VariableBindings {
                        bindings: vec!(("x", "y").into())
                    }
                ))
            );
        }
    }
}
