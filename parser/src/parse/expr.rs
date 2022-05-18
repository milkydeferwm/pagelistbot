#![cfg(feature="parse")]

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::space0;
use nom::sequence::delimited;
use nom_locate::LocatedSpan;

use crate::ast::*;
use super::NomSpan;

#[cfg(test)]
use pagelistbot_parser_test_macro::parse_test;

pub fn parse(input: NomSpan) -> IResult<NomSpan, Expr> {
    todo!()
}

#[cfg_attr(
    test,
    parse_test(test_link_expr, "test/link_expr.in"),
)]
fn parse_link_expr(input: NomSpan) -> IResult<NomSpan, Expr> {
    todo!();
}
