//! Parser utilities.

#![cfg(feature = "parse")]

use nom::{
    IResult,
    AsChar, InputTakeAtPosition,
    character::complete::multispace0,
    error::ParseError,
    sequence::{delimited, preceded, terminated},
};

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and 
/// trailing whitespace, returning the output of `inner`.
pub(crate) fn whitespace<'a, I, O, E, F>(inner: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    I: InputTakeAtPosition + 'a,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    F: FnMut(I) -> IResult<I, O, E> + 'a,
    E: ParseError<I>,
{
    delimited(
        multispace0,
        inner,
        multispace0
    )
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes leading 
/// whitespace, returning the output of `inner`.
pub(crate) fn leading_whitespace<'a, I, O, E, F>(inner: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    I: InputTakeAtPosition + 'a,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    F: FnMut(I) -> IResult<I, O, E> + 'a,
    E: ParseError<I>,
{
    preceded(
        multispace0,
        inner,
    )
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes 
/// trailing whitespace, returning the output of `inner`.
#[allow(dead_code)]
pub(crate) fn trailing_whitespace<'a, I, O, E, F>(inner: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    I: InputTakeAtPosition + 'a,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    F: FnMut(I) -> IResult<I, O, E> + 'a,
    E: ParseError<I>,
{
    terminated(
        inner,
        multispace0,
    )
}
