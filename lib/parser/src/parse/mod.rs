#![cfg(feature="parse")]

mod parser_types;

mod expr;
mod modifier;
mod number;
mod string;
mod util;

type StrSpan<'a> = nom_locate::LocatedSpan<&'a str>;

pub fn parse<'a, E: 'a>(input: &'a str) -> Result<crate::ast::Node, E>
where
    E: nom::error::ParseError<StrSpan<'a>> + nom::error::FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    use nom::Finish;

    expr::parse::<E>(StrSpan::new(input)).finish().map(|(_, res)| res)
}
