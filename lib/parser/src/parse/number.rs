#![cfg(feature="parse")]

use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{one_of, char};
use nom::combinator::{opt, recognize, map_res};
use nom::error::{FromExternalError, ParseError};
use nom::IResult;
use nom::multi::many1;
use nom::sequence::tuple;

use crate::ast::NumberOrInf;

use super::StrSpan;

#[cfg(test)]
use parser_test_macro::parse_test;

/// Parse a i32 number. Assume no leading or trailing spaces.
/// 
/// The definition of number is heavily simplified. It must be
/// * `xxxx`; or
/// * `-xxxx`; or
/// * `+xxxx`
/// 
/// The following patterns are not allowed, unfortunately
/// * `xxxxExx`
/// * `+`
/// * `-`
/// * `xx_xx_xx` (this is allowed in rust)
/// * *maybe more?*
#[cfg_attr(
    test,
    parse_test(test_i32, "test/number/i32.in")
)]
pub(crate) fn parse_i32<'a, E>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, i32, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>,
{
    map_res(
        recognize(
            tuple((
                opt(alt((char('+'), char('-')))),
                many1(one_of("0123456789"))
            ))
        ),
        |res: StrSpan| res.parse::<i32>()
    )(input)
}

#[cfg_attr(
    test,
    parse_test(test_usize_with_inf, "test/number/usize_with_inf.in")
)]
pub(crate) fn parse_usize_with_inf<'a, E>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, NumberOrInf<usize>, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>,
{
    alt((
        map_res(
            recognize(
                tuple((
                    opt(char('+')),
                    many1(one_of("0123456789"))
                )),
            ),
            |res: StrSpan| res.parse::<usize>().map(NumberOrInf::Finite)
        ),
        map_res(
            tag_no_case("inf"),
            |_| Ok(NumberOrInf::Infinity)
        )
    ))(input)
}
