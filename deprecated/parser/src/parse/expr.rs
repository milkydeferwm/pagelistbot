use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::char;
use nom::combinator::all_consuming;
use nom::error::{ParseError, FromExternalError};
use nom::multi::{separated_list1, many0};
use nom::sequence::{delimited, preceded, tuple};

use nom_locate::position;

use interface::types::ast::*;

use super::StrSpan;
use super::modifier::parse_modifier_list;
use super::string::parse_string;
use super::util::*;

#[cfg(test)]
use parser_test_macro::parse_test;

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
fn parse_page_expr<'a, E>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Node, E>
where
    E: 'a + ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, s_pos) = position(input)?;
    let (input, list) = alt((
        separated_list1(ws(char(',')), parse_string),
        preceded(
            tag_no_case("page"),
            leading_ws(
                delimited(
                    char('('),
                    ws(separated_list1(ws(char(',')), parse_string)),
                    char(')')
                )
            )
        )
    ))(input)?;
    let (input, e_pos) = position(input)?;

    Ok((
        input,
        Node::new(Span {
            begin_line: s_pos.location_line(),
            begin_col: s_pos.get_utf8_column(),
            begin_offset: s_pos.location_offset(),
            end_line: e_pos.location_line(),
            end_col: e_pos.get_utf8_column(),
            end_offset: e_pos.location_offset(),
        },
        Expr::Page {
            titles: std::collections::BTreeSet::from_iter(list)
        })
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
fn parse_link_expr<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Node, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, s_pos) = position(input)?;
    let (input, (target, modifier)) = tuple((
        preceded(
            tag_no_case("link"),
            leading_ws(
                delimited(
                    char('('),
                    ws(parse_expr_tier1::<E>),
                    char(')')
                )
            )
        ),
        leading_ws(parse_modifier_list::<E>)
    ))(input)?;
    let (input, e_pos) = position(input)?;

    Ok((
        input,
        Node::new(Span {
            begin_line: s_pos.location_line(),
            begin_col: s_pos.get_utf8_column(),
            begin_offset: s_pos.location_offset(),
            end_line: e_pos.location_line(),
            end_col: e_pos.get_utf8_column(),
            end_offset: e_pos.location_offset(),
        },
        Expr::Link {
            target: Box::new(target),
            modifier,
        })
    ))
}

/// Parse a linkto expression. Assume no leading or trailing whitespaces.
/// 
/// `linkto(<ExprTier1>)<modifier list>`
/// 
/// With optional whitespaces between tokens
#[cfg_attr(
    test,
    parse_test(test_linkto_expr, "test/expr/linkto_expr.in"),
)]
fn parse_linkto_expr<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Node, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, s_pos) = position(input)?;
    let (input, (target, modifier)) = tuple((
        preceded(
            tag_no_case("linkto"),
            leading_ws(
                delimited(
                    char('('),
                    ws(parse_expr_tier1::<E>),
                    char(')')
                )
            )
        ),
        ws(parse_modifier_list::<E>)
    ))(input)?;
    let (input, e_pos) = position(input)?;

    Ok((
        input,
        Node::new(Span {
            begin_line: s_pos.location_line(),
            begin_col: s_pos.get_utf8_column(),
            begin_offset: s_pos.location_offset(),
            end_line: e_pos.location_line(),
            end_col: e_pos.get_utf8_column(),
            end_offset: e_pos.location_offset(),
        },
        Expr::BackLink {
            target: Box::new(target),
            modifier,
        })
    ))
}

/// Parse a embed expression. Assume no leading or trailing whitespaces.
/// 
/// `embed(<ExprTier1>)<modifier list>`
/// 
/// With optional whitespaces between tokens
#[cfg_attr(
    test,
    parse_test(test_embed_expr, "test/expr/embed_expr.in"),
)]
fn parse_embed_expr<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Node, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, s_pos) = position(input)?;
    let (input, (target, modifier)) = tuple((
        preceded(
            tag_no_case("embed"),
            leading_ws(
                delimited(
                    char('('),
                    ws(parse_expr_tier1::<E>),
                    char(')')
                )
            )
        ),
        ws(parse_modifier_list::<E>)
    ))(input)?;
    let (input, e_pos) = position(input)?;

    Ok((
        input,
        Node::new(Span {
            begin_line: s_pos.location_line(),
            begin_col: s_pos.get_utf8_column(),
            begin_offset: s_pos.location_offset(),
            end_line: e_pos.location_line(),
            end_col: e_pos.get_utf8_column(),
            end_offset: e_pos.location_offset(),
        },
        Expr::Embed {
            target: Box::new(target),
            modifier,
        })
    ))
}

/// Parse a incat expression. Assume no leading or trailing whitespaces.
/// 
/// `incat(<ExprTier1>)<modifier list>`
/// 
/// With optional whitespaces between tokens
#[cfg_attr(
    test,
    parse_test(test_incat_expr, "test/expr/incat_expr.in"),
)]
fn parse_incat_expr<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Node, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, s_pos) = position(input)?;
    let (input, (target, modifier)) = tuple((
        preceded(
            tag_no_case("incat"),
            leading_ws(
                delimited(
                    char('('),
                    ws(parse_expr_tier1::<E>),
                    char(')')
                )
            )
        ),
        ws(parse_modifier_list::<E>)
    ))(input)?;
    let (input, e_pos) = position(input)?;

    Ok((
        input,
        Node::new(Span {
            begin_line: s_pos.location_line(),
            begin_col: s_pos.get_utf8_column(),
            begin_offset: s_pos.location_offset(),
            end_line: e_pos.location_line(),
            end_col: e_pos.get_utf8_column(),
            end_offset: e_pos.location_offset(),
        },
        Expr::InCategory {
            target: Box::new(target),
            modifier,
        })
    ))
}

/// Parse a prefix expression. Assume no leading or trailing whitespaces.
/// 
/// `prefix(<ExprTier1>)<modifier list>`
/// 
/// With optional whitespaces between tokens
#[cfg_attr(
    test,
    parse_test(test_prefix_expr, "test/expr/prefix_expr.in"),
)]
fn parse_prefix_expr<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Node, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, s_pos) = position(input)?;
    let (input, (target, modifier)) = tuple((
        preceded(
            tag_no_case("prefix"),
            leading_ws(
                delimited(
                    char('('),
                    ws(parse_expr_tier1::<E>),
                    char(')')
                )
            )
        ),
        ws(parse_modifier_list::<E>)
    ))(input)?;
    let (input, e_pos) = position(input)?;

    Ok((
        input,
        Node::new(Span {
            begin_line: s_pos.location_line(),
            begin_col: s_pos.get_utf8_column(),
            begin_offset: s_pos.location_offset(),
            end_line: e_pos.location_line(),
            end_col: e_pos.get_utf8_column(),
            end_offset: e_pos.location_offset(),
        },
        Expr::Prefix {
            target: Box::new(target),
            modifier,
        })
    ))
}


/// Parse a toggle expression. Assume no leading or trailing whitespaces.
/// 
/// `toggle(<ExprTier1>)`
/// 
/// With optional whitespaces between tokens
#[cfg_attr(
    test,
    parse_test(test_toggle_expr, "test/expr/toggle_expr.in"),
)]
fn parse_toggle_expr<'a, E: 'a>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Node, E>
where
    E: ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, s_pos) = position(input)?;
    let (input, target) = preceded(
        tag_no_case("toggle"),
        leading_ws(
            delimited(
                char('('),
                ws(parse_expr_tier1::<E>),
                char(')')
            )
        )
    )(input)?;
    let (input, e_pos) = position(input)?;

    Ok((
        input,
        Node::new(Span {
            begin_line: s_pos.location_line(),
            begin_col: s_pos.get_utf8_column(),
            begin_offset: s_pos.location_offset(),
            end_line: e_pos.location_line(),
            end_col: e_pos.get_utf8_column(),
            end_offset: e_pos.location_offset(),
        },
        Expr::Toggle {
            target: Box::new(target),
        })
    ))
}

/// Parse a term. Assume no leading or trailing whitespaces
/// 
/// A term is
/// * `(<ExprTier1>)`
/// * `<Page>`
/// * `<other unary exprs>`
fn parse_term<'a, E>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Node, E>
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
#[cfg_attr(
    test,
    parse_test(test_expr_tier3, "test/expr/expr_tier3.in"),
)]
fn parse_expr_tier3<'a, E>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Node, E>
where
    E: 'a + ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, s_pos) = position(input)?;
    let (input, set1) = parse_term::<E>(input)?;
    let (input, exprs) = many0(tuple((
        ws(char('&')),
        parse_term::<E>,
        position
    )))(input)?;
    let folded = exprs.into_iter().fold(set1, |acc, (op, set2, e_pos)| {
        match op {
            '&' => Node::new(Span {
                begin_line: s_pos.location_line(),
                begin_col: s_pos.get_utf8_column(),
                begin_offset: s_pos.location_offset(),
                end_line: e_pos.location_line(),
                end_col: e_pos.get_utf8_column(),
                end_offset: e_pos.location_offset(),
            },
            Expr::Intersection {
                set1: Box::new(acc),
                set2: Box::new(set2),
            }),
            _ => unreachable!(),
        }
    });
    Ok((input, folded))
}

/// Parse a binary expression (`^`). Assume no leading or trailing whitespaces
/// * `<ExprTier3>^<ExprTier3>`
#[cfg_attr(
    test,
    parse_test(test_expr_tier2, "test/expr/expr_tier2.in"),
)]
fn parse_expr_tier2<'a, E>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Node, E>
where
    E: 'a + ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, s_pos) = position(input)?;
    let (input, set1) = parse_expr_tier3::<E>(input)?;
    let (input, exprs) = many0(tuple((
        ws(char('^')),
        parse_expr_tier3::<E>,
        position
    )))(input)?;
    let folded = exprs.into_iter().fold(set1, |acc, (op, set2, e_pos)| {
        match op {
            '^' => Node::new(Span {
                begin_line: s_pos.location_line(),
                begin_col: s_pos.get_utf8_column(),
                begin_offset: s_pos.location_offset(),
                end_line: e_pos.location_line(),
                end_col: e_pos.get_utf8_column(),
                end_offset: e_pos.location_offset(),
            },
            Expr::Xor {
                set1: Box::new(acc),
                set2: Box::new(set2),
            }),
            _ => unreachable!(),
        }
    });
    Ok((input, folded))
}

/// Parse a binary expression (`+`, `-`). Assume no leading or trailing whitespaces
/// * `<ExprTier2>+<ExprTier2>`
/// * `<ExprTier2>-<ExprTier2>`
#[cfg_attr(
    test,
    parse_test(test_expr_tier1, "test/expr/expr_tier1.in"),
)]
fn parse_expr_tier1<'a, E>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Node, E>
where
    E: 'a + ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    let (input, s_pos) = position(input)?;
    let (input, set1) = parse_expr_tier2::<E>(input)?;
    let (input, exprs) = many0(tuple((
        ws(alt((char('+'), char('-')))),
        parse_expr_tier2::<E>,
        position
    )))(input)?;
    let folded = exprs.into_iter().fold(set1, |acc, (op, set2, e_pos)| {
        match op {
            '+' => Node::new(Span {
                begin_line: s_pos.location_line(),
                begin_col: s_pos.get_utf8_column(),
                begin_offset: s_pos.location_offset(),
                end_line: e_pos.location_line(),
                end_col: e_pos.get_utf8_column(),
                end_offset: e_pos.location_offset(),
            },
            Expr::Union {
                set1: Box::new(acc),
                set2: Box::new(set2),
            }),
            '-' => Node::new(Span {
                begin_line: s_pos.location_line(),
                begin_col: s_pos.get_utf8_column(),
                begin_offset: s_pos.location_offset(),
                end_line: e_pos.location_line(),
                end_col: e_pos.get_utf8_column(),
                end_offset: e_pos.location_offset(),
            },
            Expr::Difference {
                set1: Box::new(acc),
                set2: Box::new(set2),
            }),
            _ => unreachable!(),
        }
    });
    Ok((input, folded))
}

#[cfg_attr(
    test,
    parse_test(test_whitespaced_expr, "test/expr/whitespaced_expr.in"),
)]
pub(crate) fn parse<'a, E>(input: StrSpan<'a>) -> IResult<StrSpan<'a>, Node, E>
where
    E: 'a + ParseError<StrSpan<'a>> + FromExternalError<StrSpan<'a>, std::num::ParseIntError>
{
    all_consuming(ws(parse_expr_tier1::<E>))(input)
}
