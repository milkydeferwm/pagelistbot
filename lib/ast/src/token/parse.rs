//! Parse module for token types.

#![cfg(feature = "parse")]

use super::{
    Dot, Comma, LeftParen, RightParen, And, Add, Sub, Caret,
    Page, Link, LinkTo, Embed, InCat, Prefix, Toggle,
    Limit, Resolve, Ns, Depth, NoRedir, OnlyRedir, Direct,
};

macro_rules! parse_token {
    ($token:ident, $lit:literal) => {
        impl $token {
            /// Parse the token from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
            pub fn parse<'a, E>(program: &'a str) -> Result<Self, E>
            where
                E: nom::error::ParseError<crate::LocatedStr<'a>>,
            {
                use nom::Finish;
                let span = crate::LocatedStr::new(program);
                nom::combinator::all_consuming(
                    crate::parse_util::whitespace(Self::parse_internal::<E>)
                )(span).finish().map(|(_, x)| x)
            }

            /// Parse the token from a span. Assume no whitespaces before.
            pub(crate) fn parse_internal<'a, E>(program: crate::LocatedStr<'a>) -> nom::IResult<crate::LocatedStr<'a>, Self, E>
            where
                E: nom::error::ParseError<crate::LocatedStr<'a>>,
            {
                let (residual, (span, pos_end)) = nom::sequence::tuple((
                    nom::bytes::complete::tag_no_case($lit),
                    nom_locate::position,
                ))(program)?;
                let token = Self { span: crate::make_range(span.location_offset(), pos_end.location_offset()) };
                Ok((residual, token))
            }
        }
    }
}

parse_token!(Dot, ".");
parse_token!(Comma, ",");
parse_token!(LeftParen, "(");
parse_token!(RightParen, ")");
parse_token!(And, "&");
parse_token!(Add, "+");
parse_token!(Sub, "-");
parse_token!(Caret, "^");
parse_token!(Page, "page");
parse_token!(Link, "link");
parse_token!(LinkTo, "linkto");
parse_token!(Embed, "embed");
parse_token!(InCat, "incat");
parse_token!(Prefix, "prefix");
parse_token!(Toggle, "toggle");
parse_token!(Limit, "limit");
parse_token!(Resolve, "resolve");
parse_token!(Ns, "ns");
parse_token!(Depth, "depth");
parse_token!(NoRedir, "noredir");
parse_token!(OnlyRedir, "onlyredir");
parse_token!(Direct, "direct");

#[cfg(test)]
mod test {
    macro_rules! make_test {
        ($test:ident, $token:ident, $lit:literal) => {
            #[test]
            fn $test() {
                use alloc::format;
                use crate::LocatedStr;
                use super::$token;
                use nom::error::Error;

                let gt = $lit;
                let upper = gt.to_uppercase();
                let lower = gt.to_lowercase();

                let input_1 = format!("{gt}");
                let input_2 = format!("  {upper}");
                let input_3 = format!("{lower} ");
                let input_4 = format!(" {gt} ");

                let tok_1 = $token::parse::<Error<LocatedStr<'_>>>(&input_1).unwrap();
                let tok_2 = $token::parse::<Error<LocatedStr<'_>>>(&input_2).unwrap();
                let tok_3 = $token::parse::<Error<LocatedStr<'_>>>(&input_3).unwrap();
                let tok_4 = $token::parse::<Error<LocatedStr<'_>>>(&input_4).unwrap();

                assert_eq!(&input_1[tok_1.get_span().to_range()], gt);
                assert_eq!(&input_2[tok_2.get_span().to_range()], &upper);
                assert_eq!(&input_3[tok_3.get_span().to_range()], &lower);
                assert_eq!(&input_4[tok_4.get_span().to_range()], gt);

                assert_eq!(tok_1.get_span().start, 0);
                assert_eq!(tok_2.get_span().start, 2);
                assert_eq!(tok_3.get_span().start, 0);
                assert_eq!(tok_4.get_span().start, 1);
            }
        }
    }

    make_test!(test_parse_dot, Dot, ".");
    make_test!(test_parse_comma, Comma, ",");
    make_test!(test_parse_leftparen, LeftParen, "(");
    make_test!(test_parse_rightparen, RightParen, ")");
    make_test!(test_parse_and, And, "&");
    make_test!(test_parse_add, Add, "+");
    make_test!(test_parse_sub, Sub, "-");
    make_test!(test_parse_caret, Caret, "^");
    make_test!(test_parse_page, Page, "PaGe");
    make_test!(test_parse_link, Link, "LiNk");
    make_test!(test_parse_linkto, LinkTo, "LiNkTo");
    make_test!(test_parse_embed, Embed, "EmBeD");
    make_test!(test_parse_incat, InCat, "InCaT");
    make_test!(test_parse_prefix, Prefix, "PrEfIx");
    make_test!(test_parse_toggle, Toggle, "ToGgLe");
    make_test!(test_parse_limit, Limit, "LiMiT");
    make_test!(test_parse_resolve, Resolve, "ReSoLvE");
    make_test!(test_parse_ns, Ns, "Ns");
    make_test!(test_parse_depth, Depth, "DePtH");
    make_test!(test_parse_noredir, NoRedir, "NoReDiR");
    make_test!(test_parse_onlyredir, OnlyRedir, "OnLyReDiR");
    make_test!(test_parse_direct, Direct, "DiReCt");
}
