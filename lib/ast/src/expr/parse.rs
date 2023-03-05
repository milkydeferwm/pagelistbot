//! Parsing module for expressions.

#![cfg(feature = "parse")]

use alloc::boxed::Box;
use core::num::ParseIntError;
use crate::{
    Span,
    attribute::Attribute,
    literal::LitString,
    parse_util::{whitespace, leading_whitespace, alternating1},
    token::{
        Add, And, Caret, Sub, LeftParen, RightParen, Comma,
        Page, Link, LinkTo, Embed, InCat, Prefix, Toggle,
    }
};
use super::{
    Expression,
    ExpressionAnd, ExpressionAdd, ExpressionSub, ExpressionXor,
    ExpressionParen,
    ExpressionPage, ExpressionLink, ExpressionLinkTo, ExpressionEmbed, ExpressionInCat, ExpressionPrefix, ExpressionToggle,
};

use nom::{
    IResult,
    Finish, Slice,
    branch::alt,
    combinator::{all_consuming, map},
    error::{ParseError, FromExternalError},
    multi::many0,
    sequence::tuple,
};
use nom_locate::position;

enum Level1Operator<'a> {
    Add(Add<'a>),
    Sub(Sub<'a>),
}

impl<'a> Expression<'a> {
    /// Parse the expression from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let span = Span::new(program);
        all_consuming(
            whitespace(Self::parse_internal_level_1::<E>)
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse a level-1 expression. Level 1 has the lowest priority, and sits at the top of the AST.
    /// `ExpressionAdd` and `ExpressionSub` sit at this level.
    fn parse_internal_level_1<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let (residual, (pos_start, expr, exprs)) = tuple((
            position,
            Expression::parse_internal_level_2,
            many0(tuple((
                leading_whitespace(alt((
                    map(Add::parse_internal, Level1Operator::Add),
                    map(Sub::parse_internal, Level1Operator::Sub),
                ))),
                leading_whitespace(Expression::parse_internal_level_2),
                position,
            ))),
        ))(program)?;
        let expression = exprs.into_iter().fold(
            expr,
            |expr1, (lv1op, expr2, pos_end)| {
                let length = pos_end.location_offset() - pos_start.location_offset();
                match lv1op {
                    Level1Operator::Add(add) => Self::Add(ExpressionAdd {
                        span: program.slice(..length),
                        expr1: Box::new(expr1),
                        add,
                        expr2: Box::new(expr2),
                    }),
                    Level1Operator::Sub(sub) => Self::Sub(ExpressionSub {
                        span: program.slice(..length),
                        expr1: Box::new(expr1),
                        sub,
                        expr2: Box::new(expr2),
                    }),
                }
            }
        );
        Ok((residual, expression))
    }

    /// Parse a level-2 expression. Level 2 has the second-lowest priority.
    /// `ExpressionXor` sits at this level.
    fn parse_internal_level_2<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let (residual, (pos_start, expr, exprs)) = tuple((
            position,
            Expression::parse_internal_level_3,
            many0(tuple((
                leading_whitespace(Caret::parse_internal),
                leading_whitespace(Expression::parse_internal_level_3),
                position,
            ))),
        ))(program)?;
        let expression = exprs.into_iter().fold(
            expr,
            |expr1, (caret, expr2, pos_end)| {
                let length = pos_end.location_offset() - pos_start.location_offset();
                Self::Xor(ExpressionXor {
                    span: program.slice(..length),
                    expr1: Box::new(expr1),
                    xor: caret,
                    expr2: Box::new(expr2),
                })
            }
        );
        Ok((residual, expression))
    }

    /// Parse a level-3 expression. Level 3 has the second-highest priority.
    /// `ExpressionAnd` sits at this level.
    fn parse_internal_level_3<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let (residual, (pos_start, expr, exprs)) = tuple((
            position,
            Expression::parse_internal_level_4,
            many0(tuple((
                leading_whitespace(And::parse_internal),
                leading_whitespace(Expression::parse_internal_level_4),
                position,
            ))),
        ))(program)?;
        let expression = exprs.into_iter().fold(
            expr,
            |expr1, (and, expr2, pos_end)| {
                let length = pos_end.location_offset() - pos_start.location_offset();
                Self::And(ExpressionAnd {
                    span: program.slice(..length),
                    expr1: Box::new(expr1),
                    and,
                    expr2: Box::new(expr2),
                })
            }
        );
        Ok((residual, expression))
    }

    /// Parse a level-4 expression. Level 4 has the highest priority.
    /// `ExpressionParam` and all other expressions sit at this level.
    fn parse_internal_level_4<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        alt((
            map(ExpressionParen::parse_internal, Expression::Paren),
            map(ExpressionPage::parse_internal, Expression::Page),
            map(ExpressionLink::parse_internal, Expression::Link),
            map(ExpressionLinkTo::parse_internal, Expression::LinkTo),
            map(ExpressionEmbed::parse_internal, Expression::Embed),
            map(ExpressionInCat::parse_internal, Expression::InCat),
            map(ExpressionPrefix::parse_internal, Expression::Prefix),
            map(ExpressionToggle::parse_internal, Expression::Toggle),
        ))(program)
    }
}

impl<'a> ExpressionParen<'a> {
    /// Parse the expression from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let span = Span::new(program);
        all_consuming(
            whitespace(Self::parse_internal::<E>)
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse the expression from a span. Assume no whitespaces before.
    pub(crate) fn parse_internal<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let (residual, (pos_start, lparen, expr, rparen, pos_end)) = tuple((
            position,
            LeftParen::parse_internal,
            leading_whitespace(Expression::parse_internal_level_1),
            leading_whitespace(RightParen::parse_internal),
            position,
        ))(program)?;
        let length = pos_end.location_offset() - pos_start.location_offset();
        let expression_paren = Self {
            span: program.slice(..length),
            lparen,
            expr: Box::new(expr),
            rparen,
        };
        Ok((residual, expression_paren))
    }
}

impl<'a> ExpressionPage<'a> {
    /// Parse the expression from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let span = Span::new(program);
        all_consuming(
            whitespace(Self::parse_internal::<E>)
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse the expression from a span. Assume no whitespaces before.
    pub(crate) fn parse_internal<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        alt((
            Self::parse_internal_style_1,
            Self::parse_internal_style_2,
        ))(program)
    }

    /// Parse the expression with the first style.
    fn parse_internal_style_1<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let (residual, (pos_start, (vals, commas), pos_end)) = tuple((
            position,
            alternating1(
                leading_whitespace(Comma::parse_internal),
                leading_whitespace(LitString::parse_internal),
            ),
            position,
        ))(program)?;
        let length = pos_end.location_offset() - pos_start.location_offset();
        let expression_page = Self {
            span: program.slice(..length),
            page: None,
            lparen: None,
            vals,
            commas,
            rparen: None,
        };
        Ok((residual, expression_page))
    }

    /// Parse the expression with the second style.
    fn parse_internal_style_2<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let (residual, (pos_start, page, lparen, (vals, commas), rparen, pos_end)) = tuple((
            position,
            Page::parse_internal,
            leading_whitespace(LeftParen::parse_internal),
            alternating1(
                leading_whitespace(Comma::parse_internal),
                leading_whitespace(LitString::parse_internal),
            ),
            leading_whitespace(RightParen::parse_internal),
            position,
        ))(program)?;
        let length = pos_end.location_offset() - pos_start.location_offset();
        let expression_page = Self {
            span: program.slice(..length),
            page: Some(page),
            lparen: Some(lparen),
            vals,
            commas,
            rparen: Some(rparen),
        };
        Ok((residual, expression_page))
    }
}

macro_rules! unary_operation_make_parser {
    ($name:ident, $token_field:ident, $token:ident) => {
        impl<'a> $name<'a> {
            /// Parse the expression from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
            pub fn parse<E>(program: &'a str) -> Result<Self, E>
            where
                E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
            {
                let span = Span::new(program);
                all_consuming(
                    whitespace(Self::parse_internal::<E>)
                )(span).finish().map(|(_, x)| x)
            }

            /// Parse the expression from a span. Assume no whitespaces before.
            pub(crate) fn parse_internal<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
            where
                E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
            {
                let (residual, (pos_start, $token_field, lparen, expr, rparen, attributes, pos_end)) = tuple((
                    position,
                    $token::parse_internal,
                    leading_whitespace(LeftParen::parse_internal),
                    leading_whitespace(Expression::parse_internal_level_1),
                    leading_whitespace(RightParen::parse_internal),
                    many0(
                        leading_whitespace(Attribute::parse_internal),
                    ),
                    position,
                ))(program)?;
                let length = pos_end.location_offset() - pos_start.location_offset();
                let expression = Self {
                    span: program.slice(..length),
                    $token_field,
                    lparen,
                    expr: Box::new(expr),
                    rparen,
                    attributes,
                };
                Ok((residual, expression))
            }
        }
    }
}

unary_operation_make_parser!(ExpressionLink, link, Link);
unary_operation_make_parser!(ExpressionLinkTo, linkto, LinkTo);
unary_operation_make_parser!(ExpressionEmbed, embed, Embed);
unary_operation_make_parser!(ExpressionInCat, incat, InCat);
unary_operation_make_parser!(ExpressionPrefix, prefix, Prefix);

impl<'a> ExpressionToggle<'a> {
    /// Parse the expression from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let span = Span::new(program);
        all_consuming(
            whitespace(Self::parse_internal::<E>)
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse the expression from a span. Assume no whitespaces before.
    pub(crate) fn parse_internal<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let (residual, (pos_start, toggle, lparen, expr, rparen, pos_end)) = tuple((
            position,
            Toggle::parse_internal,
            leading_whitespace(LeftParen::parse_internal),
            leading_whitespace(Expression::parse_internal_level_1),
            leading_whitespace(RightParen::parse_internal),
            position,
        ))(program)?;
        let length = pos_end.location_offset() - pos_start.location_offset();
        let expression_toggle = Self {
            span: program.slice(..length),
            toggle,
            lparen,
            expr: Box::new(expr),
            rparen,
        };
        Ok((residual, expression_toggle))
    }
}
