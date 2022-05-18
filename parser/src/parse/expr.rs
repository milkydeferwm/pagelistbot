#![cfg(feature="parse")]

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::char;
use nom::error::{ParseError, FromExternalError};
use nom::multi::separated_list1;
use nom::sequence::{delimited, preceded};

use nom_locate::{position, LocatedSpan};

use crate::ast::*;
use super::StrSpan;
use super::parser_types::*;
use super::string::parse_string;

#[cfg(test)]
use pagelistbot_parser_test_macro::parse_test;

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and 
/// trailing whitespace, returning the output of `inner`.
fn ws<'a, F: 'a, O, E>(inner: F) -> impl FnMut(StrSpan<'a>) -> IResult<StrSpan<'a>, O, E>
where
    F: FnMut(StrSpan<'a>) -> IResult<StrSpan<'a>, O, E>,
    E: ParseError<StrSpan<'a>>
{
    use nom::character::complete::multispace0;
    delimited(
        multispace0,
        inner,
        multispace0
    )
}

/// Parse a `page` expression. Assumes no leading or trailing whitespaces.
/// 
/// Page expression has one of the two forms:
/// * `page("title1","title2","title3")`; or simply
/// * `"title1","title2","title3"`
/// 
/// With optional whitespaces between tokens.
#[cfg_attr(
    test,
    parse_test(test_page_expr, "test/page_expr.in"),
)]
fn parse_page_expr<'a, E>(input: StrSpan) -> IResult<StrSpan, Expr>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, pos) = position(input)?;
    let (input, list) = alt((
        separated_list1(char(','), ws(parse_string)),
        preceded(
            tag_no_case("page"),
            ws(
                delimited(
                    char('('),
                    ws(separated_list1(char(','), ws(parse_string))),
                    char(')')
                )
            )
        )
    ))(input)?;

    Ok((
        input,
        Expr::Page {
            span: Span { offset: pos.location_offset(), line: pos.location_line(), column: pos.get_utf8_column() },
            titles: list
        }
    ))
}

pub fn parse<'a, E>(input: StrSpan) -> IResult<StrSpan, Expr, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    todo!()
}

#[cfg_attr(
    test,
    parse_test(test_link_expr, "test/link_expr.in"),
)]
fn parse_link_expr<'a, E>(input: StrSpan) -> IResult<StrSpan, Expr>
where
    E: nom::error::ParseError<StrSpan<'a>> + nom::error::FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    todo!();
}
