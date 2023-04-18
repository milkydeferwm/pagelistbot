//! Tokens. Including symbols, delimeters and keywords.

#[cfg(feature = "parse")]
pub mod parse;

macro_rules! define_token {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name {
            span: crate::Span,
        }
        crate::expose_span!($name);
    };
}

define_token!(Dot);             // `.`
define_token!(Comma);           // `,`
define_token!(LeftParen);       // `(`
define_token!(RightParen);      // `)`
define_token!(And);             // `&`
define_token!(Add);             // `+`
define_token!(Sub);             // `-`
define_token!(Caret);           // `^`
define_token!(Page);            // `page`
define_token!(Link);            // `link`
define_token!(LinkTo);          // `linkto`
define_token!(Embed);           // `embed`
define_token!(InCat);           // `incat`
define_token!(Prefix);          // `prefix`
define_token!(Toggle);          // `toggle`
define_token!(Limit);           // `limit`
define_token!(Resolve);         // `resolve`
define_token!(Ns);              // `ns`
define_token!(Depth);           // `depth`
define_token!(NoRedir);         // `noredir`
define_token!(OnlyRedir);       // `onlyredir`
define_token!(Direct);          // `direct`
