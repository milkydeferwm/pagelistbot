#![cfg(feature="parse")]

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::space0;
use nom::sequence::delimited;
use nom_locate::LocatedSpan;

use crate::ast::*;
use super::StrSpan;
use super::parser_types::*;

#[cfg(test)]
use pagelistbot_parser_test_macro::parse_test;

pub fn parse<'a, E>(input: StrSpan) -> IResult<StrSpan, Expr, E>
where
    E: nom::error::ParseError<StrSpan<'a>> + nom::error::FromExternalError<StrSpan<'a>, std::num::ParseIntError>
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
