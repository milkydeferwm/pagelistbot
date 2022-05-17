pub mod ast;

#[cfg(feature="parse")]
mod parser_types;
#[cfg(feature="parse")]
mod parse;

/*
#[cfg(feature="parse")]
pub fn parse(input: &str) -> Result<(NomSpan, Expr), Error> {
    use nom::error::Error;
    use parse::NomSpan;

    todo!()
}
*/
