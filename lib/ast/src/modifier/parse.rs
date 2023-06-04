//! Parse module for modifiers.

#![cfg(feature = "parse")]

use core::num::ParseIntError;
use crate::{
    LocatedStr,
    make_range,
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
    Finish,
    branch::alt,
    combinator::{all_consuming, opt, map},
    error::{ParseError, FromExternalError},
    sequence::tuple,
};
use nom_locate::position;

impl Modifier {
    /// Parse the modifier from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<'a, E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
    {
        let span = LocatedStr::new(program);
        all_consuming(
            whitespace(Self::parse_internal::<E>)
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse the modifier from a span. Assume no whitespaces before.
    pub(crate) fn parse_internal<'a, E>(program: LocatedStr<'a>) -> IResult<LocatedStr<'a>, Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
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

impl ModifierNs {
    /// Parse the modifier from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<'a, E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
    {
        let span = LocatedStr::new(program);
        all_consuming(
            whitespace(Self::parse_internal::<E>)
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse the modifier from a span. Assume no whitespaces before.
    pub(crate) fn parse_internal<'a, E>(program: LocatedStr<'a>) -> IResult<LocatedStr<'a>, Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
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
        let modifier_ns = Self {
            span: make_range(pos_start.location_offset(), pos_end.location_offset()),
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
        impl $name {
            /// Parse the modifier from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
            pub fn parse<'a, E>(program: &'a str) -> Result<Self, E>
            where
                E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
            {
                let span = LocatedStr::new(program);
                all_consuming(
                    whitespace(Self::parse_internal::<E>)
                )(span).finish().map(|(_, x)| x)
            }

            /// Parse the modifier from a span. Assume no whitespaces before.
            pub(crate) fn parse_internal<'a, E>(program: LocatedStr<'a>) -> IResult<LocatedStr<'a>, Self, E>
            where
                E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
            {
                let (residual, (pos_start, $token_field, lparen, val, rparen, pos_end)) = tuple((
                    position,
                    $token::parse_internal,
                    leading_whitespace(LeftParen::parse_internal),
                    leading_whitespace(LitIntOrInf::parse_internal),
                    leading_whitespace(RightParen::parse_internal),
                    position,
                ))(program)?;
                let modifier = Self {
                    span: make_range(pos_start.location_offset(), pos_end.location_offset()),
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
        impl $name {
            /// Parse the modifier from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
            pub fn parse<'a, E>(program: &'a str) -> Result<Self, E>
            where
                E: ParseError<LocatedStr<'a>>,
            {
                let span = LocatedStr::new(program);
                all_consuming(
                    whitespace(Self::parse_internal::<E>)
                )(span).finish().map(|(_, x)| x)
            }

            /// Parse the modifier from a span. Assume no whitespaces before.
            pub(crate) fn parse_internal<'a, E>(program: LocatedStr<'a>) -> IResult<LocatedStr<'a>, Self, E>
            where
                E: ParseError<LocatedStr<'a>>,
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
                let (lparen, rparen) = match opt_paren {
                    Some((lparen, rparen)) => (Some(lparen), Some(rparen)),
                    None => (None, None),
                };
                let modifier = Self {
                    span: make_range(pos_start.location_offset(), pos_end.location_offset()),
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

#[cfg(test)]
mod test {
    use alloc::{format, vec, vec::Vec};
    use crate::{LocatedStr, IntOrInf, literal::LitInt};
    use super::{
        Modifier,
        ModifierLimit, ModifierResolve, ModifierNs, ModifierDepth, ModifierNoRedir, ModifierOnlyRedir, ModifierDirect,
    };
    use nom::error::Error;

    #[test]
    fn test_parse_modifier() {
        let input_limit = "limit(-1)";
        let input_resolve = " Resolve";
        let input_ns = "nS (-1 ,0, 2004 ) ";
        let input_depth = "  dePth( 5) ";
        let input_noredir = "noredir  ";
        let input_onlyredir = " ONLYREDIR ";
        let input_direct = "DiReCt";

        let mod_limit = Modifier::parse::<Error<LocatedStr<'_>>>(input_limit).unwrap();
        let mod_resolve = Modifier::parse::<Error<LocatedStr<'_>>>(input_resolve).unwrap();
        let mod_ns = Modifier::parse::<Error<LocatedStr<'_>>>(input_ns).unwrap();
        let mod_depth = Modifier::parse::<Error<LocatedStr<'_>>>(input_depth).unwrap();
        let mod_noredir = Modifier::parse::<Error<LocatedStr<'_>>>(input_noredir).unwrap();
        let mod_onlyredir = Modifier::parse::<Error<LocatedStr<'_>>>(input_onlyredir).unwrap();
        let mod_direct = Modifier::parse::<Error<LocatedStr<'_>>>(input_direct).unwrap();

        assert!(matches!(mod_limit, Modifier::Limit(_)));
        assert!(matches!(mod_resolve, Modifier::Resolve(_)));
        assert!(matches!(mod_ns, Modifier::Ns(_)));
        assert!(matches!(mod_depth, Modifier::Depth(_)));
        assert!(matches!(mod_noredir, Modifier::NoRedir(_)));
        assert!(matches!(mod_onlyredir, Modifier::OnlyRedir(_)));
        assert!(matches!(mod_direct, Modifier::Direct(_)));

        assert_eq!(&input_limit[mod_limit.get_span().to_range()], "limit(-1)");
        assert_eq!(&input_resolve[mod_resolve.get_span().to_range()], "Resolve");
        assert_eq!(&input_ns[mod_ns.get_span().to_range()], "nS (-1 ,0, 2004 )");
        assert_eq!(&input_depth[mod_depth.get_span().to_range()], "dePth( 5)");
        assert_eq!(&input_noredir[mod_noredir.get_span().to_range()], "noredir");
        assert_eq!(&input_onlyredir[mod_onlyredir.get_span().to_range()], "ONLYREDIR");
        assert_eq!(&input_direct[mod_direct.get_span().to_range()], "DiReCt");

        assert_eq!(mod_limit.get_span().start, 0);
        assert_eq!(mod_resolve.get_span().start, 1);
        assert_eq!(mod_ns.get_span().start, 0);
        assert_eq!(mod_depth.get_span().start, 2);
        assert_eq!(mod_noredir.get_span().start, 0);
        assert_eq!(mod_onlyredir.get_span().start, 1);
        assert_eq!(mod_direct.get_span().start, 0);
    }

    #[test]
    fn test_parse_modifier_ns() {
        fn extract_nums(lits: &[LitInt]) -> Vec<i32> {
            lits.iter().map(|x| x.val).collect()
        }

        let input_1 = "ns(0)";
        let input_2 = "  Ns (0,1,2)";
        let input_3 = "nS( 0, 1, )  ";
        let input_4 = " NS ( 0 , +1 , -1 )  ";

        let mod_1 = ModifierNs::parse::<Error<LocatedStr<'_>>>(input_1).unwrap();
        let mod_2 = ModifierNs::parse::<Error<LocatedStr<'_>>>(input_2).unwrap();
        let mod_3 = ModifierNs::parse::<Error<LocatedStr<'_>>>(input_3).unwrap();
        let mod_4 = ModifierNs::parse::<Error<LocatedStr<'_>>>(input_4).unwrap();

        assert_eq!(extract_nums(&mod_1.vals), vec![0]);
        assert_eq!(extract_nums(&mod_2.vals), vec![0, 1, 2]);
        assert_eq!(extract_nums(&mod_3.vals), vec![0, 1]);
        assert_eq!(extract_nums(&mod_4.vals), vec![0, 1, -1]);

        assert_eq!(mod_1.commas.len(), 0);
        assert_eq!(mod_2.commas.len(), 2);
        assert_eq!(mod_3.commas.len(), 2);
        assert_eq!(mod_4.commas.len(), 2);

        assert_eq!(&input_1[mod_1.get_span().to_range()], "ns(0)");
        assert_eq!(&input_2[mod_2.get_span().to_range()], "Ns (0,1,2)");
        assert_eq!(&input_3[mod_3.get_span().to_range()], "nS( 0, 1, )");
        assert_eq!(&input_4[mod_4.get_span().to_range()], "NS ( 0 , +1 , -1 )");

        assert_eq!(mod_1.get_span().start, 0);
        assert_eq!(mod_2.get_span().start, 2);
        assert_eq!(mod_3.get_span().start, 0);
        assert_eq!(mod_4.get_span().start, 1);
    }

    macro_rules! intorinf_modifier_make_test {
        ($test:ident, $target:ident, $lit:literal) => {
            #[test]
            fn $test() {
                let input_1 = format!("{}(0)", $lit);
                let input_2 = format!("  {} ( -1)", $lit);
                let input_3 = format!("{}(+100 )  ", $lit);
                let input_4 = format!(" {} ( -10000 )  ", $lit);

                let mod_1 = $target::parse::<Error<LocatedStr<'_>>>(&input_1).unwrap();
                let mod_2 = $target::parse::<Error<LocatedStr<'_>>>(&input_2).unwrap();
                let mod_3 = $target::parse::<Error<LocatedStr<'_>>>(&input_3).unwrap();
                let mod_4 = $target::parse::<Error<LocatedStr<'_>>>(&input_4).unwrap();

                assert_eq!(mod_1.val.val, IntOrInf::Int(0));
                assert_eq!(mod_2.val.val, IntOrInf::Inf);
                assert_eq!(mod_3.val.val, IntOrInf::Int(100));
                assert_eq!(mod_4.val.val, IntOrInf::Inf);

                assert_eq!(&input_1[mod_1.val.get_span().to_range()], "0");
                assert_eq!(&input_2[mod_2.val.get_span().to_range()], "-1");
                assert_eq!(&input_3[mod_3.val.get_span().to_range()], "+100");
                assert_eq!(&input_4[mod_4.val.get_span().to_range()], "-10000");

                assert_eq!(&input_1[mod_1.get_span().to_range()], &format!("{}(0)", $lit));
                assert_eq!(&input_2[mod_2.get_span().to_range()], &format!("{} ( -1)", $lit));
                assert_eq!(&input_3[mod_3.get_span().to_range()], &format!("{}(+100 )", $lit));
                assert_eq!(&input_4[mod_4.get_span().to_range()], &format!("{} ( -10000 )", $lit));

                assert_eq!(mod_1.get_span().start, 0);
                assert_eq!(mod_2.get_span().start, 2);
                assert_eq!(mod_3.get_span().start, 0);
                assert_eq!(mod_4.get_span().start, 1);
            }
        }
    }

    intorinf_modifier_make_test!(test_parse_modifier_limit, ModifierLimit, "limit");
    intorinf_modifier_make_test!(test_parse_modifier_depth, ModifierDepth, "depth");

    macro_rules! no_param_modifier_make_test {
        ($test:ident, $target:ident, $lit:literal) => {
            #[test]
            fn $test() {
                let input_1 = $lit;
                let input_2 = format!("  {}()", $lit);
                let input_3 = format!("{}  ( ) ", $lit);
                let input_4 = format!(" {}  ", $lit);

                let mod_1 = $target::parse::<Error<LocatedStr<'_>>>(&input_1).unwrap();
                let mod_2 = $target::parse::<Error<LocatedStr<'_>>>(&input_2).unwrap();
                let mod_3 = $target::parse::<Error<LocatedStr<'_>>>(&input_3).unwrap();
                let mod_4 = $target::parse::<Error<LocatedStr<'_>>>(&input_4).unwrap();

                assert_eq!(&input_1[mod_1.get_span().to_range()], $lit);
                assert_eq!(&input_2[mod_2.get_span().to_range()], &format!("{}()", $lit));
                assert_eq!(&input_3[mod_3.get_span().to_range()], &format!("{}  ( )", $lit));
                assert_eq!(&input_4[mod_4.get_span().to_range()], $lit);

                assert_eq!(mod_1.get_span().start, 0);
                assert_eq!(mod_2.get_span().start, 2);
                assert_eq!(mod_3.get_span().start, 0);
                assert_eq!(mod_4.get_span().start, 1);

                assert_eq!(mod_1.lparen, None);
                assert_eq!(mod_1.rparen, None);
                assert_eq!(&input_2[mod_2.lparen.unwrap().get_span().to_range()], "(");
                assert_eq!(&input_2[mod_2.rparen.unwrap().get_span().to_range()], ")");
                assert_eq!(&input_3[mod_3.lparen.unwrap().get_span().to_range()], "(");
                assert_eq!(&input_3[mod_3.rparen.unwrap().get_span().to_range()], ")");
                assert_eq!(mod_4.lparen, None);
                assert_eq!(mod_4.rparen, None);
            }
        }
    }

    no_param_modifier_make_test!(test_parse_modifier_resolve, ModifierResolve, "resolve");
    no_param_modifier_make_test!(test_parse_modifier_noredir, ModifierNoRedir, "noredir");
    no_param_modifier_make_test!(test_parse_modifier_onlyredir, ModifierOnlyRedir, "onlyredir");
    no_param_modifier_make_test!(test_parse_modifier_direct, ModifierDirect, "direct");
}
