use nom::{
    branch::alt,
    bytes::complete::{tag, take_till, take_until, take_while, take_while1},
    character::{
        complete::{alpha1, alphanumeric1, newline, space0},
        is_alphanumeric, is_space,
    },
    character::{
        complete::{char, multispace0},
        is_newline,
    },
    combinator::{eof, fail, opt, recognize},
    error::ParseError,
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, terminated},
    Compare, IResult, InputLength, InputTake,
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
