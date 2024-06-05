//! Parsing module for expressions.

use alloc::boxed::Box;
use core::num::ParseIntError;
use crate::{
    LocatedStr,
    make_range,
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
    Finish,
    branch::alt,
    combinator::{all_consuming, map},
    error::{ParseError, FromExternalError},
    multi::many0,
    sequence::tuple,
};
use nom_locate::position;

enum Level1Operator {
    Add(Add),
    Sub(Sub),
}

impl Expression {
    /// Parse the expression from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<'a, E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
    {
        let span = LocatedStr::new(program);
        all_consuming(
            whitespace(Self::parse_internal_level_1::<E>)
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse a level-1 expression. Level 1 has the lowest priority, and sits at the top of the AST.
    /// `ExpressionAdd` and `ExpressionSub` sit at this level.
    fn parse_internal_level_1<'a, E>(program: LocatedStr<'a>) -> IResult<LocatedStr<'a>, Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
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
                match lv1op {
                    Level1Operator::Add(add) => Self::Add(ExpressionAdd {
                        span: make_range(pos_start.location_offset(), pos_end.location_offset()),
                        expr1: Box::new(expr1),
                        add,
                        expr2: Box::new(expr2),
                    }),
                    Level1Operator::Sub(sub) => Self::Sub(ExpressionSub {
                        span: make_range(pos_start.location_offset(), pos_end.location_offset()),
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
    fn parse_internal_level_2<'a, E>(program: LocatedStr<'a>) -> IResult<LocatedStr<'a>, Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
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
                Self::Xor(ExpressionXor {
                    span: make_range(pos_start.location_offset(), pos_end.location_offset()),
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
    fn parse_internal_level_3<'a, E>(program: LocatedStr<'a>) -> IResult<LocatedStr<'a>, Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
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
                Self::And(ExpressionAnd {
                    span: make_range(pos_start.location_offset(), pos_end.location_offset()),
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
    fn parse_internal_level_4<'a, E>(program: LocatedStr<'a>) -> IResult<LocatedStr<'a>, Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
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

impl ExpressionParen {
    /// Parse the expression from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<'a, E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
    {
        let span = LocatedStr::new(program);
        all_consuming(
            whitespace(Self::parse_internal::<E>)
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse the expression from a span. Assume no whitespaces before.
    pub(crate) fn parse_internal<'a, E>(program: LocatedStr<'a>) -> IResult<LocatedStr<'a>, Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
    {
        let (residual, (pos_start, lparen, expr, rparen, pos_end)) = tuple((
            position,
            LeftParen::parse_internal,
            leading_whitespace(Expression::parse_internal_level_1),
            leading_whitespace(RightParen::parse_internal),
            position,
        ))(program)?;
        let expression_paren = Self {
            span: make_range(pos_start.location_offset(), pos_end.location_offset()),
            lparen,
            expr: Box::new(expr),
            rparen,
        };
        Ok((residual, expression_paren))
    }
}

impl ExpressionPage {
    /// Parse the expression from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<'a, E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
    {
        let span = LocatedStr::new(program);
        all_consuming(
            whitespace(Self::parse_internal::<E>)
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse the expression from a span. Assume no whitespaces before.
    pub(crate) fn parse_internal<'a, E>(program: LocatedStr<'a>) -> IResult<LocatedStr<'a>, Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
    {
        alt((
            Self::parse_internal_style_1,
            Self::parse_internal_style_2,
        ))(program)
    }

    /// Parse the expression with the first style.
    fn parse_internal_style_1<'a, E>(program: LocatedStr<'a>) -> IResult<LocatedStr<'a>, Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
    {
        let (residual, (pos_start, (vals, commas), pos_end)) = tuple((
            position,
            alternating1(
                leading_whitespace(Comma::parse_internal),
                leading_whitespace(LitString::parse_internal),
            ),
            position,
        ))(program)?;
        let expression_page = Self {
            span: make_range(pos_start.location_offset(), pos_end.location_offset()),
            page: None,
            lparen: None,
            vals,
            commas,
            rparen: None,
        };
        Ok((residual, expression_page))
    }

    /// Parse the expression with the second style.
    fn parse_internal_style_2<'a, E>(program: LocatedStr<'a>) -> IResult<LocatedStr<'a>, Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
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
        let expression_page = Self {
            span: make_range(pos_start.location_offset(), pos_end.location_offset()),
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
        impl $name {
            /// Parse the expression from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
            pub fn parse<'a, E>(program: &'a str) -> Result<Self, E>
            where
                E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
            {
                let span = LocatedStr::new(program);
                all_consuming(
                    whitespace(Self::parse_internal::<E>)
                )(span).finish().map(|(_, x)| x)
            }

            /// Parse the expression from a span. Assume no whitespaces before.
            pub(crate) fn parse_internal<'a, E>(program: LocatedStr<'a>) -> IResult<LocatedStr<'a>, Self, E>
            where
                E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
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
                let expression = Self {
                    span: make_range(pos_start.location_offset(), pos_end.location_offset()),
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

impl ExpressionToggle {
    /// Parse the expression from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<'a, E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
    {
        let span = LocatedStr::new(program);
        all_consuming(
            whitespace(Self::parse_internal::<E>)
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse the expression from a span. Assume no whitespaces before.
    pub(crate) fn parse_internal<'a, E>(program: LocatedStr<'a>) -> IResult<LocatedStr<'a>, Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
    {
        let (residual, (pos_start, toggle, lparen, expr, rparen, pos_end)) = tuple((
            position,
            Toggle::parse_internal,
            leading_whitespace(LeftParen::parse_internal),
            leading_whitespace(Expression::parse_internal_level_1),
            leading_whitespace(RightParen::parse_internal),
            position,
        ))(program)?;
        let expression_toggle = Self {
            span: make_range(pos_start.location_offset(), pos_end.location_offset()),
            toggle,
            lparen,
            expr: Box::new(expr),
            rparen,
        };
        Ok((residual, expression_toggle))
    }
}

#[cfg(test)]
mod test {
    use alloc::format;
    use crate::LocatedStr;
    use super::{
        Expression,
        ExpressionPage, ExpressionLink, ExpressionLinkTo, ExpressionEmbed, ExpressionInCat, ExpressionPrefix, ExpressionToggle,
    };
    use nom::error::Error;

    #[test]
    fn test_parse_expression() {
        let input_1 = " \"A\" + \"b\" ";
        let input_2 = "\"A\"-\"B\"";
        let input_3 = "  \"A\" ^ \"B\"";
        let input_4 = "\"A\"&\"B\" ";
        let input_5 = "(\"A\")";
        let input_6 = "\"A\"+\"B\"-\"C\"";
        let input_7 = "\"A\"+\"B\"^\"c\"";
        let input_8 = "\"A\"^\"B\"&\"c\"";
        let input_9 = "(\"A\" ^ \"B\" + \"C\") & ((\"D\" - \"E\") &\"F\")";

        let exp_1 = Expression::parse::<Error<LocatedStr<'_>>>(input_1).unwrap();
        let exp_2 = Expression::parse::<Error<LocatedStr<'_>>>(input_2).unwrap();
        let exp_3 = Expression::parse::<Error<LocatedStr<'_>>>(input_3).unwrap();
        let exp_4 = Expression::parse::<Error<LocatedStr<'_>>>(input_4).unwrap();
        let exp_5 = Expression::parse::<Error<LocatedStr<'_>>>(input_5).unwrap();
        let exp_6 = Expression::parse::<Error<LocatedStr<'_>>>(input_6).unwrap();
        let exp_7 = Expression::parse::<Error<LocatedStr<'_>>>(input_7).unwrap();
        let exp_8 = Expression::parse::<Error<LocatedStr<'_>>>(input_8).unwrap();
        let exp_9 = Expression::parse::<Error<LocatedStr<'_>>>(input_9).unwrap();

        assert!(matches!(exp_1, Expression::Add(_)));
        assert!(matches!(exp_2, Expression::Sub(_)));
        assert!(matches!(exp_3, Expression::Xor(_)));
        assert!(matches!(exp_4, Expression::And(_)));
        assert!(matches!(exp_5, Expression::Paren(_)));
        assert!(matches!(exp_6, Expression::Sub(_)));
        assert!(matches!(exp_7, Expression::Add(_)));
        assert!(matches!(exp_8, Expression::Xor(_)));
        assert!(matches!(exp_9, Expression::And(_)));
    }

    #[test]
    fn test_parse_expression_page() {
        let input_1 = "\"Main Page\"";
        let input_2 = " \"Hello\" , \"World\"";
        let input_3 = "page ( \"Test\",\"page\" )  ";
        let input_4 = "  Page(\"Sakura\")  ";

        let exp_1 = ExpressionPage::parse::<Error<LocatedStr<'_>>>(input_1).unwrap();
        let exp_2 = ExpressionPage::parse::<Error<LocatedStr<'_>>>(input_2).unwrap();
        let exp_3 = ExpressionPage::parse::<Error<LocatedStr<'_>>>(input_3).unwrap();
        let exp_4 = ExpressionPage::parse::<Error<LocatedStr<'_>>>(input_4).unwrap();

        assert_eq!(exp_1.vals.len(), 1);
        assert_eq!(exp_2.vals.len(), 2);
        assert_eq!(exp_3.vals.len(), 2);
        assert_eq!(exp_4.vals.len(), 1);

        assert_eq!(&input_1[exp_1.get_span().to_range()], "\"Main Page\"");
        assert_eq!(&input_2[exp_2.get_span().to_range()], "\"Hello\" , \"World\"");
        assert_eq!(&input_3[exp_3.get_span().to_range()], "page ( \"Test\",\"page\" )");
        assert_eq!(&input_4[exp_4.get_span().to_range()], "Page(\"Sakura\")");

        assert_eq!(exp_1.get_span().start, 0);
        assert_eq!(exp_2.get_span().start, 1);
        assert_eq!(exp_3.get_span().start, 0);
        assert_eq!(exp_4.get_span().start, 2);
    }

    macro_rules! unary_operation_make_test {
        ($test:ident, $target:ident, $lit:literal) => {
            #[test]
            fn $test() {
                let input_1 = format!("{}(\"Example\")", $lit);
                let input_2 = format!(" {} (\"Example\") . resolve ( )", $lit);
                let input_3 = format!("{}( \"Example\" ). noredir .onlyredir ", $lit);
                let input_4 = format!("  {} ( \"Example\" ) . Ns ( 0 , 1, 2 ) . limit ( 100 ) . onlyredir ", $lit);

                let exp_1 = $target::parse::<Error<LocatedStr<'_>>>(&input_1).unwrap();
                let exp_2 = $target::parse::<Error<LocatedStr<'_>>>(&input_2).unwrap();
                let exp_3 = $target::parse::<Error<LocatedStr<'_>>>(&input_3).unwrap();
                let exp_4 = $target::parse::<Error<LocatedStr<'_>>>(&input_4).unwrap();

                assert_eq!(exp_1.attributes.len(), 0);
                assert_eq!(exp_2.attributes.len(), 1);
                assert_eq!(exp_3.attributes.len(), 2);
                assert_eq!(exp_4.attributes.len(), 3);

                assert_eq!(&input_1[exp_1.get_span().to_range()], format!("{}(\"Example\")", $lit));
                assert_eq!(&input_2[exp_2.get_span().to_range()], format!("{} (\"Example\") . resolve ( )", $lit));
                assert_eq!(&input_3[exp_3.get_span().to_range()], format!("{}( \"Example\" ). noredir .onlyredir", $lit));
                assert_eq!(&input_4[exp_4.get_span().to_range()], format!("{} ( \"Example\" ) . Ns ( 0 , 1, 2 ) . limit ( 100 ) . onlyredir", $lit));

                assert_eq!(exp_1.get_span().start, 0);
                assert_eq!(exp_2.get_span().start, 1);
                assert_eq!(exp_3.get_span().start, 0);
                assert_eq!(exp_4.get_span().start, 2);
            }
        }
    }

    unary_operation_make_test!(test_parse_expression_link, ExpressionLink, "link");
    unary_operation_make_test!(test_parse_expression_linkto, ExpressionLinkTo, "linkto");
    unary_operation_make_test!(test_parse_expression_embed, ExpressionEmbed, "embed");
    unary_operation_make_test!(test_parse_expression_incat, ExpressionInCat, "incat");
    unary_operation_make_test!(test_parse_expression_prefix, ExpressionPrefix, "prefix");

    #[test]
    fn test_parse_expression_toggle() {
        let input_1 = "toggle(\"Main Page\")";
        let input_2 = " toggle ( \"Hello\" , \"World\" )";
        let input_3 = "toggle ( \"Test\",\"page\" )  ";
        let input_4 = "  toggle(linkto(\"Sakura\"))  ";

        let exp_1 = ExpressionToggle::parse::<Error<LocatedStr<'_>>>(input_1).unwrap();
        let exp_2 = ExpressionToggle::parse::<Error<LocatedStr<'_>>>(input_2).unwrap();
        let exp_3 = ExpressionToggle::parse::<Error<LocatedStr<'_>>>(input_3).unwrap();
        let exp_4 = ExpressionToggle::parse::<Error<LocatedStr<'_>>>(input_4).unwrap();

        assert_eq!(&input_1[exp_1.get_span().to_range()], "toggle(\"Main Page\")");
        assert_eq!(&input_2[exp_2.get_span().to_range()], "toggle ( \"Hello\" , \"World\" )");
        assert_eq!(&input_3[exp_3.get_span().to_range()], "toggle ( \"Test\",\"page\" )");
        assert_eq!(&input_4[exp_4.get_span().to_range()], "toggle(linkto(\"Sakura\"))");

        assert_eq!(exp_1.get_span().start, 0);
        assert_eq!(exp_2.get_span().start, 1);
        assert_eq!(exp_3.get_span().start, 0);
        assert_eq!(exp_4.get_span().start, 2);
    }
}
