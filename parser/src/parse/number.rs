#![cfg(feature="parse")]

use nom::branch::alt;
use nom::character::complete::{one_of, char};
use nom::combinator::{opt, recognize, map_res};
use nom::error::{FromExternalError, ParseError};
use nom::IResult;
use nom::multi::many1;
use nom::sequence::tuple;

use super::StrSpan;

#[cfg(test)]
use pagelistbot_parser_test_macro::parse_test;

/// Parse a i64 number. Assume no leading or trailing spaces.
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
    parse_test(test_i64, "test/i64.in")
)]
pub(crate) fn parse_i64<'a, E>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, i64, E>
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
        |res: StrSpan| res.parse::<i64>()
    )(input)
}