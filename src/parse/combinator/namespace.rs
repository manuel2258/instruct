use std::collections::HashMap;

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

use crate::parse::ast::{Namespace, NamespaceOrExecuteable, NamespaceType};
use crate::parse::combinator::executeable::executor_name;

use super::executeable::executeable_or;

pub fn namespace<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Namespace, E> {
    alt((collection, module))(i)
}

pub fn namespace_or<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, NamespaceOrExecuteable, E> {
    let (i, namespace) = namespace(i)?;
    Ok((i, NamespaceOrExecuteable::Namespace(namespace)))
}

pub fn namespace_or_executeable<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, NamespaceOrExecuteable, E> {
    alt((namespace_or, executeable_or))(i)
}

pub fn namespace_or_executeable_map<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, HashMap<String, NamespaceOrExecuteable>, E> {
    let (i, namespaces) = many0(alt((namespace_or, executeable_or)))(i)?;
    Ok((
        i,
        namespaces
            .drain(..)
            .map(|val| (val.get_name().into(), val))
            .collect(),
    ))
}

pub fn module<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Namespace, E> {
    let (i, _) = multispace0(i)?;
    let (i, _) = preceded(space0, tag("module"))(i)?;
    let (i, name) = executor_name(i)?;
    let (i, _) = preceded(space0, char(';'))(i)?;
    let (i, children) = namespace_or_executeable_map(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = eof(i)?;

    Ok((
        i,
        Namespace {
            name,
            namespace_type: NamespaceType::Module,
            children,
        },
    ))
}

pub fn collection<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Namespace, E> {
    let (i, _) = multispace0(i)?;
    let (i, _) = preceded(space0, tag("collection"))(i)?;
    let (i, name) = executor_name(i)?;
    let (i, _) = preceded(space0, char(':'))(i)?;
    let (i, _) = delimited(space0, char('{'), multispace0)(i)?;
    let (i, children) = namespace_or_executeable_map(i)?;
    let (i, _) = preceded(multispace0, char('}'))(i)?;
    let (i, _) = preceded(space0, char(';'))(i)?;
    Ok((
        i,
        Namespace {
            name,
            namespace_type: NamespaceType::Collection,
            children,
        },
    ))
}
