#![cfg(feature="parse")]

mod parser_types;

mod expr;

type NomSpan<'a> = nom_locate::LocatedSpan<&'a str>;

pub fn parse(input: &str) -> Result<crate::ast::Expr, nom::error::Error<NomSpan>> {
    use nom::Finish;

    expr::parse(NomSpan::new(input)).finish().map(|(_, res)| res)
}