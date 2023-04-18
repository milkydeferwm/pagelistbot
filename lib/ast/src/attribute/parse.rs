//! Parse module for attributes.

#![cfg(feature = "parse")]

use core::num::ParseIntError;
use crate::{
    LocatedStr,
    make_range,
    parse_util::{whitespace, leading_whitespace},
    token::Dot,
    modifier::Modifier,
};
use super::{
    Attribute,
    AttributeModifier,
};

use nom::{
    IResult,
    Finish,
    branch::alt,
    combinator::{all_consuming, map},
    error::{ParseError, FromExternalError},
    sequence::tuple,
};
use nom_locate::position;

impl Attribute {
    /// Parse the attribute from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<'a, E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
    {
        let span = LocatedStr::new(program);
        all_consuming(
            whitespace(Self::parse_internal::<E>)
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse the attribute from a span. Assume no whitespaces before.
    pub(crate) fn parse_internal<'a, E>(program: LocatedStr<'a>) -> IResult<LocatedStr<'a>, Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
    {
        alt((
            map(AttributeModifier::parse_internal, Self::Modifier),
        ))(program)
    }
}

impl AttributeModifier {
    /// Parse the attribute from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<'a, E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
    {
        let span = LocatedStr::new(program);
        all_consuming(
            whitespace(Self::parse_internal::<E>)
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse the attribute from a span. Assume no whitespaces before.
    pub(crate) fn parse_internal<'a, E>(program: LocatedStr<'a>) -> IResult<LocatedStr<'a>, Self, E>
    where
        E: ParseError<LocatedStr<'a>> + FromExternalError<LocatedStr<'a>, ParseIntError>,
    {
        let (residual, (pos_start, dot, modifier, pos_end)) = tuple((
            position,
            Dot::parse_internal,
            leading_whitespace(Modifier::parse_internal),
            position,
        ))(program)?;
        let attribute_modifier = Self {
            span: make_range(pos_start.location_offset(), pos_end.location_offset()),
            dot,
            modifier,
        };
        Ok((residual, attribute_modifier))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        LocatedStr,
        modifier::Modifier,
    };
    use super::{
        Attribute,
        AttributeModifier,
    };
    use alloc::borrow::ToOwned;
    use nom::error::Error;

    #[test]
    fn test_parse_attribute() {
        let input_1 = ".direct";

        let attr_1 = Attribute::parse::<Error<LocatedStr<'_>>>(input_1).unwrap();

        assert!(matches!(attr_1, Attribute::Modifier(_)));

        assert_eq!(&input_1[attr_1.get_span().to_owned()], ".direct");

        assert_eq!(attr_1.get_span().start, 0);
    }

    #[test]
    fn test_parse_attribute_modifier() {
        let input_1 = ".direct";
        let input_2 = " . ns (0,1,)";
        let input_3 = ".limit( 100 )  ";
        let input_4 = "  . noredir  ";

        let attr_1 = AttributeModifier::parse::<Error<LocatedStr<'_>>>(input_1).unwrap();
        let attr_2 = AttributeModifier::parse::<Error<LocatedStr<'_>>>(input_2).unwrap();
        let attr_3 = AttributeModifier::parse::<Error<LocatedStr<'_>>>(input_3).unwrap();
        let attr_4 = AttributeModifier::parse::<Error<LocatedStr<'_>>>(input_4).unwrap();

        assert!(matches!(attr_1.modifier, Modifier::Direct(_)));
        assert!(matches!(attr_2.modifier, Modifier::Ns(_)));
        assert!(matches!(attr_3.modifier, Modifier::Limit(_)));
        assert!(matches!(attr_4.modifier, Modifier::NoRedir(_)));

        assert_eq!(&input_1[attr_1.dot.get_span().to_owned()], ".");
        assert_eq!(&input_2[attr_2.dot.get_span().to_owned()], ".");
        assert_eq!(&input_3[attr_3.dot.get_span().to_owned()], ".");
        assert_eq!(&input_4[attr_4.dot.get_span().to_owned()], ".");

        assert_eq!(&input_1[attr_1.get_span().to_owned()], ".direct");
        assert_eq!(&input_2[attr_2.get_span().to_owned()], ". ns (0,1,)");
        assert_eq!(&input_3[attr_3.get_span().to_owned()], ".limit( 100 )");
        assert_eq!(&input_4[attr_4.get_span().to_owned()], ". noredir");

        assert_eq!(attr_1.get_span().start, 0);
        assert_eq!(attr_2.get_span().start, 1);
        assert_eq!(attr_3.get_span().start, 0);
        assert_eq!(attr_4.get_span().start, 2);
    }
}
