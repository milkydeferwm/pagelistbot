#![cfg(feature="parse")]

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{char, multispace0};
use nom::combinator::opt;
use nom::error::{ParseError, FromExternalError};
use nom::multi::{separated_list1, many0};
use nom::sequence::{delimited, preceded};

use crate::ast::*;
use super::StrSpan;
use super::parser_types::*;
use super::number::parse_i64;
use super::util::ws;

#[cfg(test)]
use pagelistbot_parser_test_macro::parse_test;

/// Parse a ResultLimit modifier. Assume no leading or trailing whitespaces.
/// 
/// `limit(xx)`
/// 
/// `xx` is an `i64` integer, and there might be whitespaces between tokens.
#[cfg_attr(
    test,
    parse_test(test_result_limit, "test/modifier/result_limit.in"),
)]
fn parse_result_limit<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, ModifierType, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, limit) = preceded(
        tag_no_case("limit"),
        ws(
            delimited(
                char('('),
                ws(parse_i64),
                char(')')
            )
        )
    )(input)?;
    Ok((input, ModifierType::ResultLimit(limit)))
}

/// Parse a ResolveRedirects modifier. Assume no leading or trailing whitespaces.
/// 
/// Must be written as
/// * `resolve()`; or
/// * `resolve`
/// 
/// The brackets are optional. There might be whitespaces between tokens. 
#[cfg_attr(
    test,
    parse_test(test_resolve_redirects, "test/modifier/resolve_redirects.in"),
)]
fn parse_resolve_redirects<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, ModifierType, E>
where
    E: ParseError<StrSpan<'a>>
{
    let (input, _) = preceded(
        tag_no_case("resolve"),
        opt(
            ws(
                delimited(
                    char('('),
                    multispace0,
                    char(')')
                )
            )
        )
    )(input)?;
    Ok((input, ModifierType::ResolveRedirects))
}

/// Parse a Namespace modifier. Assume no leading or trailing whitespaces.
/// 
/// `ns(xx,xx,xx)`
/// 
/// `xx` is an `i64` integer, and there might be whitespaces between tokens.
#[cfg_attr(
    test,
    parse_test(test_namespace, "test/modifier/namespace.in"),
)]
fn parse_namespace<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, ModifierType, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, nsvec) = preceded(
        tag_no_case("ns"),
        ws(
            delimited(
                char('('),
                ws(separated_list1(char(','), ws(parse_i64))),
                char(')')
            )
        )
    )(input)?;
    Ok((input, ModifierType::Namespace(std::collections::HashSet::from_iter(nsvec))))
}

/// Parse a RecursionDepth modifier. Assume no leading or trailing whitespaces.
/// 
/// `depth(xx)`
/// 
/// `xx` is an `i64` integer, and there might be whitespaces between tokens.
#[cfg_attr(
    test,
    parse_test(test_recursion_depth, "test/modifier/recursion_depth.in"),
)]
fn parse_recursion_depth<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, ModifierType, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, depth) = preceded(
        tag_no_case("depth"),
        ws(
            delimited(
                char('('),
                ws(parse_i64),
                char(')')
            )
        )
    )(input)?;
    Ok((input, ModifierType::RecursionDepth(depth)))
}

/// Parse a NoRedirect modifier. Assume no leading or trailing whitespaces.
/// 
/// Must be written as
/// * `noredir()`; or
/// * `noredir`
/// 
/// The brackets are optional. There might be whitespaces between tokens. 
#[cfg_attr(
    test,
    parse_test(test_no_redirect, "test/modifier/no_redirect.in"),
)]
fn parse_no_redirect<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, ModifierType, E>
where
    E: ParseError<StrSpan<'a>>
{
    let (input, _) = preceded(
        tag_no_case("noredir"),
        opt(
            ws(
                delimited(
                    char('('),
                    multispace0,
                    char(')')
                )
            )
        )
    )(input)?;
    Ok((input, ModifierType::NoRedirect))
}

/// Parse a OnlyRedirect modifier. Assume no leading or trailing whitespaces.
/// 
/// Must be written as
/// * `onlyredir()`; or
/// * `onlyredir`
/// 
/// The brackets are optional. There might be whitespaces between tokens. 
#[cfg_attr(
    test,
    parse_test(test_only_redirect, "test/modifier/only_redirect.in"),
)]
fn parse_only_redirect<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, ModifierType, E>
where
    E: ParseError<StrSpan<'a>>
{
    let (input, _) = preceded(
        tag_no_case("onlyredir"),
        opt(
            ws(
                delimited(
                    char('('),
                    multispace0,
                    char(')')
                )
            )
        )
    )(input)?;
    Ok((input, ModifierType::OnlyRedirect))
}

/// Parse a DirectBacklink modifier. Assume no leading or trailing whitespaces.
/// 
/// Must be written as
/// * `direct()`; or
/// * `direct`
/// 
/// The brackets are optional. There might be whitespaces between tokens. 
#[cfg_attr(
    test,
    parse_test(test_direct_backlink, "test/modifier/direct_backlink.in"),
)]
fn parse_direct_backlink<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, ModifierType, E>
where
    E: ParseError<StrSpan<'a>>
{
    let (input, _) = preceded(
        tag_no_case("direct"),
        opt(
            ws(
                delimited(
                    char('('),
                    multispace0,
                    char(')')
                )
            )
        )
    )(input)?;
    Ok((input, ModifierType::DirectBacklink))
}

/// Parse a modifier. Assume no leading or trailing whitespaces.
/// 
/// A modifier is written as follows
/// * `.limit(xx)`
/// * `.resolve`
/// * *etc*
/// 
/// There is a dot before each of the above modifiers. Whitespaces are allowed between tokens.
#[cfg_attr(
    test,
    parse_test(test_modifier, "test/modifier/modifier.in"),
)]
fn parse_modifier<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, ModifierType, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    preceded(
        char('.'),
        ws(
            alt((
                parse_result_limit::<E>,
                parse_resolve_redirects::<E>,
                parse_namespace::<E>,
                parse_recursion_depth::<E>,
                parse_no_redirect::<E>,
                parse_only_redirect::<E>,
                parse_direct_backlink::<E>,
            ))
        )
    )(input)
}

/// Parse a list of modifiers. Assume no leading or trailing whitespaces.
/// 
/// A list of modifiers may look like:
/// 
/// * `.ns(0).limit(-1)`
/// * `.depth(3) .ns(1) .resolve`
/// * *etc*
/// 
/// Whitespaces are permitted between tokens.
#[cfg_attr(
    test,
    parse_test(test_modifier_list, "test/modifier/modifier_list.in"),
)]
pub(crate) fn parse_modifier_list<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Modifier, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, list) = many0(ws(parse_modifier::<E>))(input)?;

    let mut builder = ModifierBuilder::new();
    for modifier in list {
        match modifier {
            ModifierType::ResultLimit(limit) => builder = builder.result_limit(limit),
            ModifierType::ResolveRedirects => builder = builder.resolve_redirects(),
            ModifierType::Namespace(ns) => builder = builder.namespace(&ns),
            ModifierType::RecursionDepth(depth) => builder = builder.categorymembers_recursion_depth(depth),
            ModifierType::NoRedirect => builder = builder.no_redirect(),
            ModifierType::OnlyRedirect => builder = builder.only_redirect(),
            ModifierType::DirectBacklink => builder = builder.direct_backlink(),
        }
    }
    Ok((input, builder.build()))
}
