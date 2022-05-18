#![cfg(feature="parse")]

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{char, multispace0};
use nom::combinator::opt;
use nom::error::{ParseError, FromExternalError};
use nom::multi::separated_list1;
use nom::sequence::{delimited, preceded};

use nom_locate::LocatedSpan;

use crate::ast::*;
use super::StrSpan;
use super::parser_types::*;
use super::number::parse_i64;
use super::util::ws;

#[cfg(test)]
use pagelistbot_parser_test_macro::parse_test;

/// Parse a ResultLimit modifier. Assume no leading or trailing spaces.
/// 
/// `limit(xx)`
/// 
/// `xx` is an `i64` integer, and there might be whitespaces between tokens.
#[cfg_attr(
    test,
    parse_test(test_result_limit, "test/result_limit.in"),
)]
fn parse_result_limit<'a, E>(input: StrSpan) -> IResult<StrSpan, ModifierType>
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

/// Parse a ResolveRedirects modifier. Assume no leading or trailing spaces.
/// 
/// Must be written as
/// * `resolve()`; or
/// * `resolve`
/// 
/// The brackets are optional. There might be spaces between tokens. 
#[cfg_attr(
    test,
    parse_test(test_resolve_redirects, "test/resolve_redirects.in"),
)]
fn parse_resolve_redirects<'a, E>(input: StrSpan) -> IResult<StrSpan, ModifierType>
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

/// Parse a Namespace modifier. Assume no leading or trailing spaces.
/// 
/// `ns(xx,xx,xx)`
/// 
/// `xx` is an `i64` integer, and there might be whitespaces between tokens.
#[cfg_attr(
    test,
    parse_test(test_namespace, "test/namespace.in"),
)]
fn parse_namespace<'a, E>(input: StrSpan) -> IResult<StrSpan, ModifierType>
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

/// Parse a RecursionDepth modifier. Assume no leading or trailing spaces.
/// 
/// `depth(xx)`
/// 
/// `xx` is an `i64` integer, and there might be whitespaces between tokens.
#[cfg_attr(
    test,
    parse_test(test_recursion_depth, "test/recursion_depth.in"),
)]
fn parse_recursion_depth<'a, E>(input: StrSpan) -> IResult<StrSpan, ModifierType>
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

/// Parse a NoRedirect modifier. Assume no leading or trailing spaces.
/// 
/// Must be written as
/// * `noredir()`; or
/// * `noredir`
/// 
/// The brackets are optional. There might be spaces between tokens. 
#[cfg_attr(
    test,
    parse_test(test_no_redirect, "test/no_redirect.in"),
)]
fn parse_no_redirect<'a, E>(input: StrSpan) -> IResult<StrSpan, ModifierType>
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

/// Parse a OnlyRedirect modifier. Assume no leading or trailing spaces.
/// 
/// Must be written as
/// * `onlyredir()`; or
/// * `onlyredir`
/// 
/// The brackets are optional. There might be spaces between tokens. 
#[cfg_attr(
    test,
    parse_test(test_only_redirect, "test/only_redirect.in"),
)]
fn parse_only_redirect<'a, E>(input: StrSpan) -> IResult<StrSpan, ModifierType>
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

/// Parse a DirectBacklink modifier. Assume no leading or trailing spaces.
/// 
/// Must be written as
/// * `direct()`; or
/// * `direct`
/// 
/// The brackets are optional. There might be spaces between tokens. 
#[cfg_attr(
    test,
    parse_test(test_direct_backlink, "test/direct_backlink.in"),
)]
fn parse_direct_backlink<'a, E>(input: StrSpan) -> IResult<StrSpan, ModifierType>
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
    parse_test(test_modifier, "test/modifier.in"),
)]
fn parse_modifier<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, ModifierType>
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
