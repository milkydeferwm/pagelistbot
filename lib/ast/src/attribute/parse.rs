//! Parse module for attributes.

#![cfg(feature = "parse")]

use core::num::ParseIntError;
use crate::{
    Span,
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
    Finish, Slice,
    branch::alt,
    combinator::{all_consuming, map},
    error::{ParseError, FromExternalError},
    sequence::tuple,
};
use nom_locate::position;

impl<'a> Attribute<'a> {
    /// Parse the attribute from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let span = Span::new(program);
        all_consuming(
            whitespace(Self::parse_internal::<E>)
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse the attribute from a span. Assume no whitespaces before.
    pub(crate) fn parse_internal<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        alt((
            map(AttributeModifier::parse_internal, Self::Modifier),
        ))(program)
    }
}

impl<'a> AttributeModifier<'a> {
    /// Parse the attribute from a raw piece of source text. Leading and trailing whitespaces are automatically removed.
    pub fn parse<E>(program: &'a str) -> Result<Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let span = Span::new(program);
        all_consuming(
            whitespace(Self::parse_internal::<E>)
        )(span).finish().map(|(_, x)| x)
    }

    /// Parse the attribute from a span. Assume no whitespaces before.
    pub(crate) fn parse_internal<E>(program: Span<'a>) -> IResult<Span<'a>, Self, E>
    where
        E: ParseError<Span<'a>> + FromExternalError<Span<'a>, ParseIntError>,
    {
        let (residual, (pos_start, dot, modifier, pos_end)) = tuple((
            position,
            Dot::parse_internal,
            leading_whitespace(Modifier::parse_internal),
            position,
        ))(program)?;
        let length = pos_end.location_offset() - pos_start.location_offset();
        let attribute_modifier = Self {
            span: program.slice(..length),
            dot,
            modifier,
        };
        Ok((residual, attribute_modifier))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        Span,
        modifier::Modifier,
    };
    use super::{
        Attribute,
        AttributeModifier,
    };
    use nom::error::Error;

    #[test]
    fn test_parse_attribute() {
        let input_1 = ".direct";

        let attr_1 = Attribute::parse::<Error<Span<'_>>>(input_1).unwrap();

        assert!(matches!(attr_1, Attribute::Modifier(_)));

        assert_eq!(*attr_1.get_span().fragment(), ".direct");

        assert_eq!(attr_1.get_span().location_offset(), 0);
    }

    #[test]
    fn test_parse_attribute_modifier() {
        let input_1 = ".direct";
        let input_2 = " . ns (0,1,)";
        let input_3 = ".limit( 100 )  ";
        let input_4 = "  . noredir  ";

        let attr_1 = AttributeModifier::parse::<Error<Span<'_>>>(input_1).unwrap();
        let attr_2 = AttributeModifier::parse::<Error<Span<'_>>>(input_2).unwrap();
        let attr_3 = AttributeModifier::parse::<Error<Span<'_>>>(input_3).unwrap();
        let attr_4 = AttributeModifier::parse::<Error<Span<'_>>>(input_4).unwrap();

        assert!(matches!(attr_1.modifier, Modifier::Direct(_)));
        assert!(matches!(attr_2.modifier, Modifier::Ns(_)));
        assert!(matches!(attr_3.modifier, Modifier::Limit(_)));
        assert!(matches!(attr_4.modifier, Modifier::NoRedir(_)));

        assert_eq!(*attr_1.dot.get_span().fragment(), ".");
        assert_eq!(*attr_2.dot.get_span().fragment(), ".");
        assert_eq!(*attr_3.dot.get_span().fragment(), ".");
        assert_eq!(*attr_4.dot.get_span().fragment(), ".");

        assert_eq!(*attr_1.get_span().fragment(), ".direct");
        assert_eq!(*attr_2.get_span().fragment(), ". ns (0,1,)");
        assert_eq!(*attr_3.get_span().fragment(), ".limit( 100 )");
        assert_eq!(*attr_4.get_span().fragment(), ". noredir");

        assert_eq!(attr_1.get_span().location_offset(), 0);
        assert_eq!(attr_2.get_span().location_offset(), 1);
        assert_eq!(attr_3.get_span().location_offset(), 0);
        assert_eq!(attr_4.get_span().location_offset(), 2);
    }
}
