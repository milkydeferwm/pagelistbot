//! The following contents are taken from nom's examples.
//! https://github.com/Geal/nom/blob/main/examples/string.rs
//! 
//! Refer to the online comments for more information

#![cfg(feature="parse")]

use nom::branch::alt;
use nom::bytes::complete::{is_not, take_while_m_n};
use nom::character::complete::{char, multispace1};
use nom::combinator::{map, map_opt, map_res, value, verify};
use nom::error::{FromExternalError, ParseError};
use nom::multi::fold_many0;
use nom::sequence::{delimited, preceded};
use nom::IResult;

use super::StrSpan;

#[cfg(test)]
use pagelistbot_parser_test_macro::parse_test;

/// Parse a unicode sequence, of the form u{XXXX}, where XXXX is 1 to 6
/// hexadecimal numerals. We will combine this later with parse_escaped_char
/// to parse sequences like \u{00AC}.
#[cfg_attr(
    test,
    parse_test(test_unicode, "test/unicode.in")
)]
fn parse_unicode<'a, E>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, char, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>,
{
    let parse_hex = take_while_m_n::<_, StrSpan<'a>, _>(1, 6, |c: char| c.is_ascii_hexdigit());
    let parse_delimited_hex = preceded(
        char('u'),
        delimited(char('{'), parse_hex, char('}')),
    );

    let parse_u32 = map_res(parse_delimited_hex, move |hex| u32::from_str_radix(&hex, 16));

    map_opt(parse_u32, |value| std::char::from_u32(value))(input)
}

/// Parse an escaped character: \n, \t, \r, \u{00AC}, etc.
#[cfg_attr(
    test,
    parse_test(test_escaped_char, "test/escaped_char.in")
)]
fn parse_escaped_char<'a, E>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, char, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>,
{
    preceded(
        char('\\'),
        alt((
            parse_unicode,
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            value('\u{08}', char('b')),
            value('\u{0C}', char('f')),
            value('\\', char('\\')),
            value('/', char('/')),
            value('"', char('"')),
        )),
    )(input)
}

/// Parse a backslash, followed by any amount of whitespace. This is used later
/// to discard any escaped whitespace.
fn parse_escaped_whitespace<'a, E: ParseError<StrSpan<'a>>>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, StrSpan<'a>, E> {
    preceded(char('\\'), multispace1)(input)
}

/// Parse a non-empty block of text that doesn't include \ or "
fn parse_literal<'a, E: ParseError<StrSpan<'a>>>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, StrSpan<'a>, E> {
    let not_quote_slash = is_not("\"\\");
    verify(not_quote_slash, |s: &StrSpan<'a>| !s.is_empty())(input)
}

/// A string fragment contains a fragment of a string being parsed: either
/// a non-empty Literal (a series of non-escaped characters), a single
/// parsed escaped character, or a block of escaped whitespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(StrSpan<'a>),
    EscapedChar(char),
    EscapedWS,
}

/// Combine parse_literal, parse_escaped_whitespace, and parse_escaped_char
/// into a StringFragment.
fn parse_fragment<'a, E>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, StringFragment<'a>, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>,
{
    alt((
        map(parse_literal, StringFragment::Literal),
        map(parse_escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, parse_escaped_whitespace),
    ))(input)
}

/// Parse a string. Use a loop of parse_fragment and push all of the fragments
/// into an output string.
#[cfg_attr(
    test,
    parse_test(test_string, "test/string.in")
)]
pub(crate) fn parse_string<'a, E>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, String, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>,
{
    let build_string = fold_many0(
        parse_fragment,
        String::new,
        |mut string, fragment| {
            match fragment {
                StringFragment::Literal(s) => string.push_str(&s),
                StringFragment::EscapedChar(c) => string.push(c),
                StringFragment::EscapedWS => {}
            }
            string
        },
    );

    delimited(char('"'), build_string, char('"'))(input)
}
