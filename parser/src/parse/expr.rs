#![cfg(feature="parse")]

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::char;
use nom::error::{ParseError, FromExternalError};
use nom::multi::{separated_list1, many0};
use nom::sequence::{delimited, preceded, tuple};

use nom_locate::{position, LocatedSpan};

use crate::ast::*;
use super::StrSpan;
use super::parser_types::*;
use super::modifier::parse_modifier_list;
use super::string::parse_string;
use super::util::ws;

#[cfg(test)]
use pagelistbot_parser_test_macro::parse_test;

/// Parse a `page` expression. Assumes no leading or trailing whitespaces.
/// 
/// Page expression has one of the two forms:
/// * `page("title1","title2","title3")`; or simply
/// * `"title1","title2","title3"`
/// 
/// With optional whitespaces between tokens.
#[cfg_attr(
    test,
    parse_test(test_page_expr, "test/expr/page_expr.in"),
)]
fn parse_page_expr<'a, E>(input: StrSpan) -> IResult<StrSpan, Expr>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, pos) = position(input)?;
    let (input, list) = alt((
        separated_list1(char(','), ws(parse_string)),
        preceded(
            tag_no_case("page"),
            ws(
                delimited(
                    char('('),
                    ws(separated_list1(char(','), ws(parse_string))),
                    char(')')
                )
            )
        )
    ))(input)?;

    Ok((
        input,
        Expr::Page {
            span: Span { offset: pos.location_offset(), line: pos.location_line(), column: pos.get_utf8_column() },
            titles: list
        }
    ))
}

/// Parse a link expression. Assume no leading or trailing whitespaces.
/// 
/// `link(<ExprTier1>)<modifier list>`
/// 
/// With optional whitespaces between tokens
#[cfg_attr(
    test,
    parse_test(test_link_expr, "test/expr/link_expr.in"),
)]
fn parse_link_expr<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Expr>
where
    E: nom::error::ParseError<StrSpan<'a>> + nom::error::FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, pos) = position(input)?;
    let (input, (target, modifier)) = tuple((
        preceded(
            tag_no_case("link"),
            ws(
                delimited(
                    char('('),
                    ws(parse_expr_tier1::<E>),
                    char(')')
                )
            )
        ),
        ws(parse_modifier_list::<E>)
    ))(input)?;

    Ok((
        input,
        Expr::Link {
            span: Span { offset: pos.location_offset(), line: pos.location_line(), column: pos.get_utf8_column() },
            target: Box::new(target),
            modifier,
        }
    ))
}

/// Parse a linkto expression. Assume no leading or trailing whitespaces.
/// 
/// `linkto(<ExprTier1>)<modifier list>`
/// 
/// With optional whitespaces between tokens
fn parse_linkto_expr<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Expr>
where
    E: nom::error::ParseError<StrSpan<'a>> + nom::error::FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, pos) = position(input)?;
    let (input, (target, modifier)) = tuple((
        preceded(
            tag_no_case("linkto"),
            ws(
                delimited(
                    char('('),
                    ws(parse_expr_tier1::<E>),
                    char(')')
                )
            )
        ),
        ws(parse_modifier_list::<E>)
    ))(input)?;

    Ok((
        input,
        Expr::BackLink {
            span: Span { offset: pos.location_offset(), line: pos.location_line(), column: pos.get_utf8_column() },
            target: Box::new(target),
            modifier,
        }
    ))
}

/// Parse a embed expression. Assume no leading or trailing whitespaces.
/// 
/// `embed(<ExprTier1>)<modifier list>`
/// 
/// With optional whitespaces between tokens
fn parse_embed_expr<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Expr>
where
    E: nom::error::ParseError<StrSpan<'a>> + nom::error::FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, pos) = position(input)?;
    let (input, (target, modifier)) = tuple((
        preceded(
            tag_no_case("embed"),
            ws(
                delimited(
                    char('('),
                    ws(parse_expr_tier1::<E>),
                    char(')')
                )
            )
        ),
        ws(parse_modifier_list::<E>)
    ))(input)?;

    Ok((
        input,
        Expr::Embed {
            span: Span { offset: pos.location_offset(), line: pos.location_line(), column: pos.get_utf8_column() },
            target: Box::new(target),
            modifier,
        }
    ))
}

/// Parse a incat expression. Assume no leading or trailing whitespaces.
/// 
/// `incat(<ExprTier1>)<modifier list>`
/// 
/// With optional whitespaces between tokens
fn parse_incat_expr<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Expr>
where
    E: nom::error::ParseError<StrSpan<'a>> + nom::error::FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, pos) = position(input)?;
    let (input, (target, modifier)) = tuple((
        preceded(
            tag_no_case("incat"),
            ws(
                delimited(
                    char('('),
                    ws(parse_expr_tier1::<E>),
                    char(')')
                )
            )
        ),
        ws(parse_modifier_list::<E>)
    ))(input)?;

    Ok((
        input,
        Expr::InCategory {
            span: Span { offset: pos.location_offset(), line: pos.location_line(), column: pos.get_utf8_column() },
            target: Box::new(target),
            modifier,
        }
    ))
}

/// Parse a prefix expression. Assume no leading or trailing whitespaces.
/// 
/// `prefix(<ExprTier1>)<modifier list>`
/// 
/// With optional whitespaces between tokens
fn parse_prefix_expr<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Expr>
where
    E: nom::error::ParseError<StrSpan<'a>> + nom::error::FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, pos) = position(input)?;
    let (input, (target, modifier)) = tuple((
        preceded(
            tag_no_case("prefix"),
            ws(
                delimited(
                    char('('),
                    ws(parse_expr_tier1::<E>),
                    char(')')
                )
            )
        ),
        ws(parse_modifier_list::<E>)
    ))(input)?;

    Ok((
        input,
        Expr::Prefix {
            span: Span { offset: pos.location_offset(), line: pos.location_line(), column: pos.get_utf8_column() },
            target: Box::new(target),
            modifier,
        }
    ))
}


/// Parse a toggle expression. Assume no leading or trailing whitespaces.
/// 
/// `toggle(<ExprTier1>)`
/// 
/// With optional whitespaces between tokens
fn parse_toggle_expr<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Expr>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, pos) = position(input)?;
    let (input, target) = preceded(
        tag_no_case("incat"),
        ws(
            delimited(
                char('('),
                ws(parse_expr_tier1::<E>),
                char(')')
            )
        )
    )(input)?;

    Ok((
        input,
        Expr::Toggle {
            span: Span { offset: pos.location_offset(), line: pos.location_line(), column: pos.get_utf8_column() },
            target: Box::new(target),
        }
    ))
}

/// Parse a term. Assume no leading or trailing whitespaces
/// 
/// A term is
/// * `(<ExprTier1>)`
/// * `<Page>`
/// * `<other unary exprs>`
fn parse_term<'a, E>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Expr>
where
    E: 'a + ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    alt((
        delimited(
            char('('),
            ws(parse_expr_tier1::<E>),
            char(')'),
        ),
        parse_page_expr::<E>,
        parse_link_expr::<E>,
        parse_linkto_expr::<E>,
        parse_embed_expr::<E>,
        parse_incat_expr::<E>,
        parse_prefix_expr::<E>,
        parse_toggle_expr::<E>,
    ))(input)
}

/// Parse a binary expression (`&`). Assume no leading or trailing whitespaces
/// * `<Term>&<Term>`
fn parse_expr_tier3<'a, E>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Expr>
where
    E: 'a + ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, pos) = position(input)?;
    let (input, set1) = ws(parse_term::<E>)(input)?;
    let (input, exprs) = many0(tuple((
        char('&'),
        ws(parse_term::<E>)
    )))(input)?;
    let folded = exprs.into_iter().fold(set1, |acc, (op, set2)| {
        match op {
            '&' => Expr::Intersection {
                span: Span { offset: pos.location_offset(), line: pos.location_line(), column: pos.get_utf8_column() },
                set1: Box::new(acc),
                set2: Box::new(set2),
            },
            _ => unreachable!(),
        }
    });
    Ok((input, folded))
}

/// Parse a binary expression (`^`). Assume no leading or trailing whitespaces
/// * `<ExprTier3>^<ExprTier3>`
fn parse_expr_tier2<'a, E>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Expr>
where
    E: 'a + ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, pos) = position(input)?;
    let (input, set1) = ws(parse_expr_tier3::<E>)(input)?;
    let (input, exprs) = many0(tuple((
        char('^'),
        ws(parse_expr_tier3::<E>)
    )))(input)?;
    let folded = exprs.into_iter().fold(set1, |acc, (op, set2)| {
        match op {
            '^' => Expr::Xor {
                span: Span { offset: pos.location_offset(), line: pos.location_line(), column: pos.get_utf8_column() },
                set1: Box::new(acc),
                set2: Box::new(set2),
            },
            _ => unreachable!(),
        }
    });
    Ok((input, folded))
}

/// Parse a binary expression (`+`, `-`). Assume no leading or trailing whitespaces
/// * `<ExprTier2>+<ExprTier2>`
/// * `<ExprTier2>-<ExprTier2>`
fn parse_expr_tier1<'a, E>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Expr>
where
    E: 'a + ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, pos) = position(input)?;
    let (input, set1) = ws(parse_expr_tier2::<E>)(input)?;
    let (input, exprs) = many0(tuple((
        alt((char('+'), char('-'))),
        ws(parse_expr_tier2::<E>)
    )))(input)?;
    let folded = exprs.into_iter().fold(set1, |acc, (op, set2)| {
        match op {
            '+' => Expr::Union {
                span: Span { offset: pos.location_offset(), line: pos.location_line(), column: pos.get_utf8_column() },
                set1: Box::new(acc),
                set2: Box::new(set2),
            },
            '-' => Expr::Difference {
                span: Span { offset: pos.location_offset(), line: pos.location_line(), column: pos.get_utf8_column() },
                set1: Box::new(acc),
                set2: Box::new(set2),
            },
            _ => unreachable!(),
        }
    });
    Ok((input, folded))
}

pub fn parse<'a, E>(input: StrSpan) -> IResult<StrSpan, Expr, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    todo!()
}
