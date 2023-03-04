//! Parse module for literal types.

#![cfg(feature = "parse")]

use alloc::string::String;
use core::num::ParseIntError;
use crate::{
    Span,
    parse_util::whitespace,
};
use super::{LitString, LitInt, LitIntOrInf};
use nom::{
    IResult, Finish, Slice,
    error::{FromExternalError, ParseError},
    branch::alt,
    bytes::complete::{take_while_m_n, is_not},
    character::complete::{char, multispace1, one_of},
    combinator::{all_consuming, map_res, map_opt, value, verify, map, recognize, opt},
    multi::{fold_many0, many1},
    sequence::{preceded, delimited, tuple},
};
use nom_locate::position;

impl<'a> LitString<'a> {
    /// Parse a `LitString` from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let span = Span::new(program);
        all_consuming(
            whitespace(Self::parse_internal::<E>)
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse a `LitString` from a span. Assume no whitespaces before.
    pub(crate) fn parse_internal<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let (program, pos_start) = position(program)?;
        let (residual, val) = parse_string(program)?;
        let (residual, pos_end) = position(residual)?;
        let length = pos_end.location_offset() - pos_start.location_offset();
        let lit_string = Self {
            span: program.slice(..length),
            val,
        };
        Ok((residual, lit_string))
    }
}

impl<'a> LitIntOrInf<'a> {
    /// Parse a `LitIntOrInf` from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let span = Span::new(program);
        all_consuming(
            whitespace(
                Self::parse_internal::<E>
            )
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse a `LitIntOrInf` from a span. Assume no whitespaces before.
    pub(crate) fn parse_internal<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let (program, pos_start) = position(program)?;
        let (residual, val) = parse_i32(program)?;
        let (residual, pos_end) = position(residual)?;
        let length = pos_end.location_offset() - pos_start.location_offset();
        let lit_intorinf = Self {
            span: program.slice(..length),
            val: val.into(),
        };
        Ok((residual, lit_intorinf))
    }
}

impl<'a> LitInt<'a> {
    /// Parse a `LitInt` from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let span = Span::new(program);
        all_consuming(
            whitespace(
                Self::parse_internal::<E>
            )
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse a `LitInt` from a span. Assume no whitespaces before.
    pub(crate) fn parse_internal<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let (program, pos_start) = position(program)?;
        let (residual, val) = parse_i32(program)?;
        let (residual, pos_end) = position(residual)?;
        let length = pos_end.location_offset() - pos_start.location_offset();
        let lit_int = Self {
            span: program.slice(..length),
            val,
        };
        Ok((residual, lit_int))
    }
}

// The following contents are taken from nom's examples.
// https://github.com/Geal/nom/blob/main/examples/string.rs
// 
// Refer to the online comments for more information

/// Parse a unicode sequence, of the form u{XXXX}, where XXXX is 1 to 6
/// hexadecimal numerals. We will combine this later with `parse_escaped_char`
/// to parse sequences like `\u{00AC}`.
fn parse_unicode<'a, E>(input: Span<'a>) -> IResult<Span<'a>, char, E>
where
    E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
{
    let parse_hex = take_while_m_n::<_, Span<'a>, _>(1, 6, |c: char| c.is_ascii_hexdigit());
    let parse_delimited_hex = preceded(
        char('u'),
        delimited(char('{'), parse_hex, char('}')),
    );

    let parse_u32 = map_res(
        parse_delimited_hex,
        move |hex| u32::from_str_radix(&hex, 16)
    );

    map_opt(parse_u32, core::char::from_u32)(input)
}

/// Parse an escaped character: \n, \t, \r, \u{00AC}, etc.
fn parse_escaped_char<'a, E>(input: Span<'a>) -> IResult<Span<'a>, char, E>
where
    E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
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
fn parse_escaped_whitespace<'a, E>(input: Span<'a>) -> IResult<Span<'a>, Span<'a>, E>
where
    E: ParseError<Span<'a>>,
{
    preceded(char('\\'), multispace1)(input)
}

/// Parse a non-empty block of text that doesn't include \ or "
fn parse_literal<'a, E>(input: Span<'a>) -> IResult<Span<'a>, Span<'a>, E>
where
    E: ParseError<Span<'a>>,
{
    let not_quote_slash = is_not("\"\\");
    verify(not_quote_slash, |s: &Span<'a>| !s.is_empty())(input)
}

/// A string fragment contains a fragment of a string being parsed: either
/// a non-empty Literal (a series of non-escaped characters), a single
/// parsed escaped character, or a block of escaped whitespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(Span<'a>),
    EscapedChar(char),
    EscapedWS,
}

/// Combine parse_literal, parse_escaped_whitespace, and parse_escaped_char
/// into a StringFragment.
fn parse_fragment<'a, E>(input: Span<'a>) -> IResult<Span<'a>, StringFragment<'a>, E>
where
    E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
{
    alt((
        map(parse_literal, StringFragment::Literal),
        map(parse_escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, parse_escaped_whitespace),
    ))(input)
}

/// Parse a string. Use a loop of parse_fragment and push all of the fragments
/// into an output string.
fn parse_string<'a, E>(input: Span<'a>) -> IResult<Span<'a>, String, E>
where
    E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
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
pub fn parse_i32<'a, E>(input: Span<'a>) -> IResult<Span<'a>, i32, E>
where
    E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
{
    map_res(
        recognize(
            tuple((
                opt(alt((char::<Span<'a>, E>('+'), char::<Span<'a>, E>('-')))),
                many1(one_of("0123456789"))
            ))
        ),
        |res| res.parse::<i32>()
    )(input)
}

#[cfg(test)]
mod test {
    use crate::{Span, IntOrInf};
    use super::{LitString, LitIntOrInf, LitInt};
    use nom::error::Error;

    #[test]
    fn test_parse_litstring() {
        let gt = "hello world, こんにちは、\n\", ί.";
        let input_1 = "\"hello world, こんにちは、\\n\\\", \\u{03AF}.\"";
        let input_2 = "  \"hello world, こんにちは、\\n\\\", \\u{03AF}.\"";
        let input_3 = "\"hello world, こんにちは、\\n\\\", \\u{03AF}.\"  ";
        let input_4 = " \"hello world, こんにちは、\\n\\\", \\u{03AF}.\" ";

        let lit_1 = LitString::parse::<Error<Span<'_>>>(input_1).unwrap();
        let lit_2 = LitString::parse::<Error<Span<'_>>>(input_2).unwrap();
        let lit_3 = LitString::parse::<Error<Span<'_>>>(input_3).unwrap();
        let lit_4 = LitString::parse::<Error<Span<'_>>>(input_4).unwrap();

        assert_eq!(lit_1.val, gt);
        assert_eq!(lit_2.val, gt);
        assert_eq!(lit_3.val, gt);
        assert_eq!(lit_4.val, gt);

        assert_eq!(*lit_1.get_span().fragment(), input_1);
        assert_eq!(*lit_2.get_span().fragment(), input_1);
        assert_eq!(*lit_3.get_span().fragment(), input_1);
        assert_eq!(*lit_4.get_span().fragment(), input_1);

        assert_eq!(lit_1.get_span().location_offset(), 0);
        assert_eq!(lit_2.get_span().location_offset(), 2);
        assert_eq!(lit_3.get_span().location_offset(), 0);
        assert_eq!(lit_4.get_span().location_offset(), 1);
    }

    #[test]
    fn test_parse_litintorinf() {
        let input_1 = "0";
        let input_2 = "  100";
        let input_3 = "-1 ";
        let input_4 = " +10000 ";

        let lit_1 = LitIntOrInf::parse::<Error<Span<'_>>>(input_1).unwrap();
        let lit_2 = LitIntOrInf::parse::<Error<Span<'_>>>(input_2).unwrap();
        let lit_3 = LitIntOrInf::parse::<Error<Span<'_>>>(input_3).unwrap();
        let lit_4 = LitIntOrInf::parse::<Error<Span<'_>>>(input_4).unwrap();

        assert_eq!(lit_1.val, IntOrInf::from(0));
        assert_eq!(lit_2.val, IntOrInf::from(100));
        assert_eq!(lit_3.val, IntOrInf::from(-1));
        assert_eq!(lit_4.val, IntOrInf::from(10000));

        assert_eq!(*lit_1.get_span().fragment(), "0");
        assert_eq!(*lit_2.get_span().fragment(), "100");
        assert_eq!(*lit_3.get_span().fragment(), "-1");
        assert_eq!(*lit_4.get_span().fragment(), "+10000");

        assert_eq!(lit_1.get_span().location_offset(), 0);
        assert_eq!(lit_2.get_span().location_offset(), 2);
        assert_eq!(lit_3.get_span().location_offset(), 0);
        assert_eq!(lit_4.get_span().location_offset(), 1);
    }

    #[test]
    fn test_parse_litint() {
        let input_1 = "+0";
        let input_2 = "  100";
        let input_3 = "-1 ";
        let input_4 = " 10000 ";

        let lit_1 = LitInt::parse::<Error<Span<'_>>>(input_1).unwrap();
        let lit_2 = LitInt::parse::<Error<Span<'_>>>(input_2).unwrap();
        let lit_3 = LitInt::parse::<Error<Span<'_>>>(input_3).unwrap();
        let lit_4 = LitInt::parse::<Error<Span<'_>>>(input_4).unwrap();

        assert_eq!(lit_1.val, 0);
        assert_eq!(lit_2.val, 100);
        assert_eq!(lit_3.val, -1);
        assert_eq!(lit_4.val, 10000);

        assert_eq!(*lit_1.get_span().fragment(), "+0");
        assert_eq!(*lit_2.get_span().fragment(), "100");
        assert_eq!(*lit_3.get_span().fragment(), "-1");
        assert_eq!(*lit_4.get_span().fragment(), "10000");

        assert_eq!(lit_1.get_span().location_offset(), 0);
        assert_eq!(lit_2.get_span().location_offset(), 2);
        assert_eq!(lit_3.get_span().location_offset(), 0);
        assert_eq!(lit_4.get_span().location_offset(), 1);
    }
}
