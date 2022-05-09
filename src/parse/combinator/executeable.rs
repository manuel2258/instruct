use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::space0,
    character::complete::{char, multispace0},
    combinator::opt,
    error::ParseError,
    multi::many1,
    sequence::{delimited, preceded, terminated},
    IResult,
};

use crate::parse::ast::{Executeable, ExecuteableType, NamespaceOrExecuteable};
use crate::parse::combinator::variable::{
    option_variable_bindings, output_variable_bindings, variable,
};

pub fn executeable<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Executeable, E> {
    alt((
        command_executeable,
        call_executeable,
        block_executeable,
        task_executeable,
    ))(i)
}

pub fn executeable_or<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, NamespaceOrExecuteable, E> {
    let (i, executeable) = executeable(i)?;
    Ok((i, NamespaceOrExecuteable::Executeable(executeable)))
}

pub fn executor_name<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, String, E> {
    let (i, _) = preceded(space0, tag("as"))(i)?;
    let (i, name) = preceded(space0, variable)(i)?;
    Ok((i, name.into()))
}

fn command_executeable<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Executeable, E> {
    let (i, _) = multispace0(i)?;
    let (i, output_variables) = opt(output_variable_bindings)(i)?;
    let (i, _) = preceded(space0, tag("run"))(i)?;
    let (i, options) = opt(option_variable_bindings)(i)?;
    let (i, opt_name) = opt(executor_name)(i)?;
    let (i, _) = preceded(space0, char(':'))(i)?;
    let (i, cmd) = terminated(take_until(";"), char(';'))(i)?;
    let name = match opt_name {
        Some(val) => val,
        None => {
            let mut hasher = DefaultHasher::new();
            cmd.hash(&mut hasher);
            hasher.finish().to_string()
        }
    };
    Ok((
        i,
        Executeable {
            output_variables,
            name,
            options,
            executeable_type: ExecuteableType::Command {
                cmd: cmd.trim().into(),
            },
        },
    ))
}

fn call_executeable<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Executeable, E> {
    let (i, _) = multispace0(i)?;
    let (i, output_variables) = opt(output_variable_bindings)(i)?;
    let (i, _) = preceded(space0, tag("call"))(i)?;
    let (i, opt_name) = opt(executor_name)(i)?;
    let (i, _) = preceded(space0, char(':'))(i)?;
    let (i, target) = terminated(take_until(";"), char(';'))(i)?;
    let name = match opt_name {
        Some(val) => val,
        None => {
            let mut hasher = DefaultHasher::new();
            target.hash(&mut hasher);
            hasher.finish().to_string()
        }
    };
    Ok((
        i,
        Executeable {
            output_variables,
            name,
            options: None,
            executeable_type: ExecuteableType::Call {
                target: target.trim().into(),
            },
        },
    ))
}

fn block_executeable<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&str, Executeable, E> {
    let (i, _) = multispace0(i)?;
    let (i, output_variables) = opt(output_variable_bindings)(i)?;
    let (i, _) = preceded(space0, tag("block"))(i)?;
    let (i, options) = opt(option_variable_bindings)(i)?;
    let (i, opt_name) = opt(executor_name)(i)?;
    let (i, _) = preceded(space0, char(':'))(i)?;
    let (i, _) = delimited(space0, char('{'), multispace0)(i)?;
    let (i, execs) = many1(executeable)(i)?;
    let (i, _) = preceded(multispace0, char('}'))(i)?;
    let (i, _) = preceded(space0, char(';'))(i)?;
    let name = match opt_name {
        Some(val) => val,
        None => {
            let mut hasher = DefaultHasher::new();
            execs.hash(&mut hasher);
            hasher.finish().to_string()
        }
    };
    Ok((
        i,
        Executeable {
            output_variables,
            name,
            options,
            executeable_type: ExecuteableType::Block {
                executeables: execs,
            },
        },
    ))
}

pub fn task_executeable<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Executeable, E> {
    let (i, _) = multispace0(i)?;
    let (i, _) = preceded(space0, tag("task"))(i)?;
    let (i, name) = executor_name(i)?;
    let (i, _) = preceded(space0, char(':'))(i)?;
    let (i, _) = delimited(space0, char('{'), multispace0)(i)?;
    let (i, executeables) = many1(executeable)(i)?;
    let (i, _) = preceded(multispace0, char('}'))(i)?;
    let (i, _) = preceded(space0, char(';'))(i)?;
    Ok((
        i,
        Executeable {
            output_variables: None,
            name,
            options: None,
            executeable_type: ExecuteableType::Task { executeables },
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::ast::VariableBindings;
    use nom::error::Error;

    #[cfg(test)]
    mod command {
        use super::*;

        #[test]
        fn ok_simple() {
            assert_eq!(
                executeable::<Error<&str>>("run: test;"),
                Ok((
                    "",
                    Executeable {
                        output_variables: None,
                        name: "".into(),
                        options: None,
                        executeable_type: ExecuteableType::Command { cmd: "test".into() }
                    }
                ))
            );
        }

        #[test]
        fn ok_binding() {
            assert_eq!(
                executeable::<Error<&str>>("let (var: stdout) from run: test;"),
                Ok((
                    "",
                    Executeable {
                        output_variables: Some(VariableBindings {
                            bindings: vec![("var", "stdout").into()]
                        }),
                        name: "".into(),
                        options: None,
                        executeable_type: ExecuteableType::Command { cmd: "test".into() }
                    }
                ))
            );
        }

        #[test]
        fn ok_name() {
            assert_eq!(
                executeable::<Error<&str>>("run as test_cmd: test;"),
                Ok((
                    "",
                    Executeable {
                        output_variables: None,
                        name: "test_cmd".into(),
                        options: None,
                        executeable_type: ExecuteableType::Command { cmd: "test".into() }
                    }
                ))
            );
        }

        #[test]
        fn ok_options() {
            assert_eq!(
                executeable::<Error<&str>>("run with (silent, cd: test_dir): test;"),
                Ok((
                    "",
                    Executeable {
                        output_variables: None,
                        name: "".into(),
                        options: Some(VariableBindings {
                            bindings: vec!["silent".into(), ("cd", "test_dir").into()]
                        }),
                        executeable_type: ExecuteableType::Command { cmd: "test".into() }
                    }
                ))
            );
        }

        #[test]
        fn ok_all() {
            assert_eq!(
                executeable::<Error<&str>>(
                    "let (var: stdout) from run with (silent, cd: test_dir) as test_cmd: test;"
                ),
                Ok((
                    "",
                    Executeable {
                        output_variables: Some(VariableBindings {
                            bindings: vec![("var", "stdout").into()]
                        }),
                        name: "test_cmd".into(),
                        options: Some(VariableBindings {
                            bindings: vec!["silent".into(), ("cd", "test_dir").into()]
                        }),
                        executeable_type: ExecuteableType::Command { cmd: "test".into() }
                    }
                ))
            );
        }
    }

    #[cfg(test)]
    mod call {
        use super::*;

        #[test]
        fn ok_simple() {
            assert_eq!(
                executeable::<Error<&str>>("call: test;"),
                Ok((
                    "",
                    Executeable {
                        output_variables: None,
                        name: "".into(),
                        options: None,
                        executeable_type: ExecuteableType::Call {
                            target: "test".into()
                        }
                    }
                ))
            );
        }

        #[test]
        fn ok_binding() {
            assert_eq!(
                executeable::<Error<&str>>("let (var: stdout) from call: test;"),
                Ok((
                    "",
                    Executeable {
                        output_variables: Some(VariableBindings {
                            bindings: vec![("var", "stdout").into()]
                        }),
                        name: "".into(),
                        options: None,
                        executeable_type: ExecuteableType::Call {
                            target: "test".into()
                        }
                    }
                ))
            );
        }

        #[test]
        fn ok_name() {
            assert_eq!(
                executeable::<Error<&str>>("call as test_call: test;"),
                Ok((
                    "",
                    Executeable {
                        output_variables: None,
                        name: "test_call".into(),
                        options: None,
                        executeable_type: ExecuteableType::Call {
                            target: "test".into()
                        }
                    }
                ))
            );
        }

        #[test]
        fn ok_all() {
            assert_eq!(
                executeable::<Error<&str>>("let (var: stdout) from call as test_call: test;"),
                Ok((
                    "",
                    Executeable {
                        output_variables: Some(VariableBindings {
                            bindings: vec![("var", "stdout").into()]
                        }),
                        name: "test_call".into(),
                        options: None,
                        executeable_type: ExecuteableType::Call {
                            target: "test".into()
                        }
                    }
                ))
            );
        }
    }

    #[cfg(test)]
    mod block {
        use super::*;

        #[test]
        fn ok_simple() {
            assert_eq!(
                executeable::<Error<&str>>("block: {run: test;};"),
                Ok((
                    "",
                    Executeable {
                        output_variables: None,
                        name: "".into(),
                        options: None,
                        executeable_type: ExecuteableType::Block {
                            executeables: vec![Executeable {
                                output_variables: None,
                                name: "".into(),
                                options: None,
                                executeable_type: ExecuteableType::Command { cmd: "test".into() }
                            }]
                        }
                    }
                ))
            );
        }

        #[test]
        fn ok_simple_newlines() {
            assert_eq!(
                executeable::<Error<&str>>("block: {\n\trun: test;\n} ;"),
                Ok((
                    "",
                    Executeable {
                        output_variables: None,
                        name: "".into(),
                        options: None,
                        executeable_type: ExecuteableType::Block {
                            executeables: vec![Executeable {
                                output_variables: None,
                                name: "".into(),
                                options: None,
                                executeable_type: ExecuteableType::Command { cmd: "test".into() }
                            }]
                        }
                    }
                ))
            );
        }

        #[test]
        fn ok_named() {
            assert_eq!(
                executeable::<Error<&str>>("block as pre: {run: test;};"),
                Ok((
                    "",
                    Executeable {
                        output_variables: None,
                        name: "pre".into(),
                        options: None,
                        executeable_type: ExecuteableType::Block {
                            executeables: vec![Executeable {
                                output_variables: None,
                                name: "".into(),
                                options: None,
                                executeable_type: ExecuteableType::Command { cmd: "test".into() }
                            }]
                        }
                    }
                ))
            );
        }

        #[test]
        fn ok_options() {
            assert_eq!(
                executeable::<Error<&str>>("block with (runner: sh): {run: test;};"),
                Ok((
                    "",
                    Executeable {
                        output_variables: None,
                        name: "".into(),
                        options: Some(VariableBindings {
                            bindings: vec![("runner", "sh").into()]
                        }),
                        executeable_type: ExecuteableType::Block {
                            executeables: vec![Executeable {
                                output_variables: None,
                                name: "".into(),
                                options: None,
                                executeable_type: ExecuteableType::Command { cmd: "test".into() }
                            }]
                        }
                    }
                ))
            );
        }

        #[test]
        fn ok_output() {
            assert_eq!(
                executeable::<Error<&str>>("let (var: stdout) from block: {run: test;};"),
                Ok((
                    "",
                    Executeable {
                        output_variables: Some(VariableBindings {
                            bindings: vec![("var", "stdout").into()]
                        }),
                        name: "".into(),
                        options: None,
                        executeable_type: ExecuteableType::Block {
                            executeables: vec![Executeable {
                                output_variables: None,
                                name: "".into(),
                                options: None,
                                executeable_type: ExecuteableType::Command { cmd: "test".into() }
                            }]
                        }
                    }
                ))
            );
        }

        #[test]
        fn ok_nested() {
            assert_eq!(
                executeable::<Error<&str>>("block as pre1: {block as pre2: {run: test;};};"),
                Ok((
                    "",
                    Executeable {
                        output_variables: None,
                        name: "pre1".into(),
                        options: None,
                        executeable_type: ExecuteableType::Block {
                            executeables: vec![Executeable {
                                output_variables: None,
                                name: "pre2".into(),
                                options: None,
                                executeable_type: ExecuteableType::Block {
                                    executeables: vec![Executeable {
                                        output_variables: None,
                                        name: "".into(),
                                        options: None,
                                        executeable_type: ExecuteableType::Command {
                                            cmd: "test".into()
                                        }
                                    }]
                                }
                            }],
                        }
                    }
                ))
            );
        }

        #[test]
        fn nok_empty() {
            assert_ne!(
                executeable::<Error<&str>>("block: {};"),
                Ok((
                    "",
                    Executeable {
                        output_variables: None,
                        name: "".into(),
                        options: None,
                        executeable_type: ExecuteableType::Block {
                            executeables: vec![]
                        }
                    }
                ))
            );
        }

        #[test]
        fn nok_missing_semicolon() {
            assert_ne!(
                executeable::<Error<&str>>("block: {run: test;}"),
                Ok((
                    "",
                    Executeable {
                        output_variables: None,
                        name: "".into(),
                        options: None,
                        executeable_type: ExecuteableType::Block {
                            executeables: vec![Executeable {
                                output_variables: None,
                                name: "".into(),
                                options: None,
                                executeable_type: ExecuteableType::Command { cmd: "test".into() }
                            }]
                        }
                    }
                ))
            );
        }
    }
}
