use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    character::complete::{char, multispace0},
    combinator::eof,
    error::ParseError,
    multi::{many0, many1},
    sequence::{delimited, preceded},
    IResult,
};

use crate::parse::ast::{Namespace, NamespaceType};
use crate::parse::combinator::executeable::{executeable, executor_name};

pub fn namespace<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Namespace, E> {
    alt((task, collection, module))(i)
}

pub fn module<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Namespace, E> {
    let (i, _) = multispace0(i)?;
    let (i, _) = preceded(space0, tag("module"))(i)?;
    let (i, name) = executor_name(i)?;
    let (i, _) = preceded(space0, char(';'))(i)?;
    let (i, namespaces) = many0(namespace)(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = eof(i)?;

    Ok((
        i,
        Namespace {
            name,
            ns_type: NamespaceType::Module { namespaces },
        },
    ))
}

pub fn collection<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Namespace, E> {
    let (i, _) = multispace0(i)?;
    let (i, _) = preceded(space0, tag("collection"))(i)?;
    let (i, name) = executor_name(i)?;
    let (i, _) = preceded(space0, char(':'))(i)?;
    let (i, _) = delimited(space0, char('{'), multispace0)(i)?;
    let (i, namespaces) = many1(namespace)(i)?;
    let (i, _) = preceded(multispace0, char('}'))(i)?;
    let (i, _) = preceded(space0, char(';'))(i)?;
    Ok((
        i,
        Namespace {
            name,
            ns_type: NamespaceType::Collection { namespaces },
        },
    ))
}

pub fn task<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Namespace, E> {
    let (i, _) = multispace0(i)?;
    let (i, _) = preceded(space0, tag("task"))(i)?;
    let (i, name) = executor_name(i)?;
    let (i, _) = preceded(space0, char(':'))(i)?;
    let (i, _) = delimited(space0, char('{'), multispace0)(i)?;
    let (i, execs) = many1(executeable)(i)?;
    let (i, _) = preceded(multispace0, char('}'))(i)?;
    let (i, _) = preceded(space0, char(';'))(i)?;
    Ok((
        i,
        Namespace {
            name,
            ns_type: NamespaceType::Task { execs },
        },
    ))
}
