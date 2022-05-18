#![cfg(feature="parse")]

mod parser_types;

mod expr;
mod modifier;
mod string;

type StrSpan<'a> = nom_locate::LocatedSpan<&'a str>;

pub fn parse<'a, E>(input: &str) -> Result<crate::ast::Expr, E>
where
    E: nom::error::ParseError<StrSpan<'a>> + nom::error::FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    use nom::Finish;

    expr::parse(StrSpan::new(input)).finish().map(|(_, res)| res)
}
