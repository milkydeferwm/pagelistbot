#![cfg(feature="parse")]

use nom::IResult;
use nom::error::ParseError;
use super::StrSpan;

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and 
/// trailing whitespace, returning the output of `inner`.
pub(crate) fn ws<'a, F: 'a, O, E>(inner: F) -> impl FnMut(StrSpan<'a>) -> IResult<StrSpan<'a>, O, E>
where
    F: FnMut(StrSpan<'a>) -> IResult<StrSpan<'a>, O, E>,
    E: ParseError<StrSpan<'a>>
{
    use nom::character::complete::multispace0;
    nom::sequence::delimited(
        multispace0,
        inner,
        multispace0
    )
}
