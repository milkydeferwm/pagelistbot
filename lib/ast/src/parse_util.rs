//! Parser utilities.

use alloc::vec::Vec;
use nom::{
    IResult,
    AsChar, InputLength, InputTakeAtPosition, Parser,
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
    F: Parser<I, O, E> + 'a,
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
    F: Parser<I, O, E> + 'a,
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
    F: Parser<I, O, E> + 'a,
    E: ParseError<I>,
{
    terminated(
        inner,
        multispace0,
    )
}

/// A combinator that works like `separated_list1` but also collects the separator.
pub(crate) fn alternating1<'a, I, O, O2, E, F, G>(mut sep: G, mut f: F) -> impl FnMut(I) -> IResult<I, (Vec<O>, Vec<O2>), E>
where
    I: Clone + InputLength,
    F: Parser<I, O, E> + 'a,
    G: Parser<I, O2, E> + 'a,
    E: ParseError<I>,
{
    move |mut i: I| {
        let mut res_f = Vec::new();
        let mut res_sep = Vec::new();

        // Parse the first element
        match f.parse(i.clone()) {
            Err(e) => return Err(e),
            Ok((i1, o)) => {
                res_f.push(o);
                i = i1;
            }
        }
        // Parse the remaining
        loop {
            let len = i.input_len();
            // Parse the separator
            match sep.parse(i.clone()) {
                Err(nom::Err::Error(_)) => return Ok((i, (res_f, res_sep))),
                Err(e) => return Err(e),
                Ok((i1, s)) => {
                    res_sep.push(s);
                    i = i1;
                }
            }
            // Parse next
            match f.parse(i.clone()) {
                Err(nom::Err::Error(_)) => return Ok((i, (res_f, res_sep))),
                Err(e) => return Err(e),
                Ok((i1, o)) => {
                    // infinite loop check.
                    if i1.input_len() == len {
                        return Err(nom::Err::Error(E::from_error_kind(i1, nom::error::ErrorKind::SeparatedList)));
                    }
                    // the parser is consuming something in the end.
                    res_f.push(o);
                    i = i1;
                }
            }
        }
    }
}
