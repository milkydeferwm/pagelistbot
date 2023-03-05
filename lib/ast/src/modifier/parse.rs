//! Parse module for modifiers.

#![cfg(feature = "parse")]

use core::num::ParseIntError;
use crate::{
    Span,
    parse_util::{whitespace, leading_whitespace, alternating1},
    literal::{LitInt, LitIntOrInf},
    token::{
        LeftParen, RightParen, Comma,
        Limit, Resolve, Ns, Depth, NoRedir, OnlyRedir, Direct,
    },
};
use super::{
    Modifier,
    ModifierLimit, ModifierResolve, ModifierNs, ModifierDepth, ModifierNoRedir, ModifierOnlyRedir, ModifierDirect,
};

use nom::{
    IResult,
    Finish, Slice,
    branch::alt,
    combinator::{all_consuming, opt, map},
    error::{ParseError, FromExternalError},
    sequence::tuple,
};
use nom_locate::position;

impl<'a> Modifier<'a> {
    /// Parse the modifier from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let span = Span::new(program);
        all_consuming(
            whitespace(Self::parse_internal::<E>)
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse the modifier from a span. Assume no whitespaces before.
    pub(crate) fn parse_internal<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        alt((
            map(ModifierLimit::parse_internal, Self::Limit),
            map(ModifierResolve::parse_internal, Self::Resolve),
            map(ModifierNs::parse_internal, Self::Ns),
            map(ModifierDepth::parse_internal, Self::Depth),
            map(ModifierNoRedir::parse_internal, Self::NoRedir),
            map(ModifierOnlyRedir::parse_internal, Self::OnlyRedir),
            map(ModifierDirect::parse_internal, Self::Direct),
        ))(program)
    }
}

impl<'a> ModifierNs<'a> {
    /// Parse the modifier from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let span = Span::new(program);
        all_consuming(
            whitespace(Self::parse_internal::<E>)
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse the modifier from a span. Assume no whitespaces before.
    pub(crate) fn parse_internal<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let (residual, (pos_start, ns, lparen, (vals, commas), rparen, pos_end)) = tuple((
            position,
            Ns::parse_internal,
            leading_whitespace(LeftParen::parse_internal),
            alternating1(
                leading_whitespace(Comma::parse_internal),
                leading_whitespace(LitInt::parse_internal),
            ),
            leading_whitespace(RightParen::parse_internal),
            position,
        ))(program)?;
        let length = pos_end.location_offset() - pos_start.location_offset();
        let modifier_ns = Self {
            span: program.slice(..length),
            ns,
            lparen,
            vals,
            commas,
            rparen,
        };
        Ok((residual, modifier_ns))
    }
}

macro_rules! intorlimit_modifier_parse {
    ($name:ident, $token_field:ident, $token:ident) => {
        impl<'a> $name<'a> {
            /// Parse the modifier from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
            pub fn parse<E>(program: &'a str) -> Result<Self, E>
            where
                E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
            {
                let span = Span::new(program);
                all_consuming(
                    whitespace(Self::parse_internal::<E>)
                )(span).finish().map(|(_, x)| x)
            }

            /// Parse the modifier from a span. Assume no whitespaces before.
            pub(crate) fn parse_internal<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
            where
                E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
            {
                let (residual, (pos_start, $token_field, lparen, val, rparen, pos_end)) = tuple((
                    position,
                    $token::parse_internal,
                    leading_whitespace(LeftParen::parse_internal),
                    leading_whitespace(LitIntOrInf::parse_internal),
                    leading_whitespace(RightParen::parse_internal),
                    position,
                ))(program)?;
                let length = pos_end.location_offset() - pos_start.location_offset();
                let modifier = Self {
                    span: program.slice(..length),
                    $token_field,
                    lparen,
                    val,
                    rparen,
                };
                Ok((residual, modifier))
            }
        }
    }
}

intorlimit_modifier_parse!(ModifierLimit, limit, Limit);
intorlimit_modifier_parse!(ModifierDepth, depth, Depth);

macro_rules! no_param_modifier_parse {
    ($name:ident, $token_field:ident, $token:ident) => {
        impl<'a> $name<'a> {
            /// Parse the modifier from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
            pub fn parse<E>(program: &'a str) -> Result<Self, E>
            where
                E: ParseError<Span<'a>>,
            {
                let span = Span::new(program);
                all_consuming(
                    whitespace(Self::parse_internal::<E>)
                )(span).finish().map(|(_, x)| x)
            }

            /// Parse the modifier from a span. Assume no whitespaces before.
            pub(crate) fn parse_internal<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
            where
                E: ParseError<Span<'a>>,
            {
                let (residual, (pos_start, $token_field, opt_paren, pos_end)) = tuple((
                    position,
                    $token::parse_internal,
                    opt(
                        tuple((
                            leading_whitespace(LeftParen::parse_internal),
                            leading_whitespace(RightParen::parse_internal),
                        ))
                    ),
                    position,
                ))(program)?;
                let length = pos_end.location_offset() - pos_start.location_offset();
                let (lparen, rparen) = match opt_paren {
                    Some((lparen, rparen)) => (Some(lparen), Some(rparen)),
                    None => (None, None),
                };
                let modifier = Self {
                    span: program.slice(..length),
                    $token_field,
                    lparen,
                    rparen,
                };
                Ok((residual, modifier))
            }
        }
    }
}

no_param_modifier_parse!(ModifierResolve, resolve, Resolve);
no_param_modifier_parse!(ModifierNoRedir, noredir, NoRedir);
no_param_modifier_parse!(ModifierOnlyRedir, onlyredir, OnlyRedir);
no_param_modifier_parse!(ModifierDirect, direct, Direct);
