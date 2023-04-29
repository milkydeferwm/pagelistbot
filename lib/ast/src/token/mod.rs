//! Tokens. Including symbols, delimeters and keywords.

#[cfg(feature = "parse")]
pub mod parse;

macro_rules! define_token {
    ($name:ident, $hashas:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            span: crate::Span,
        }
        crate::expose_span!($name);
        impl core::hash::Hash for $name {
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                $hashas.hash(state);
            }
        }
    };
}

define_token!(Dot, ".");                    // `.`
define_token!(Comma, ",");                  // `,`
define_token!(LeftParen, "(");              // `(`
define_token!(RightParen, ")");             // `)`
define_token!(And, "&");                    // `&`
define_token!(Add, "+");                    // `+`
define_token!(Sub, "-");                    // `-`
define_token!(Caret, "^");                  // `^`
define_token!(Page, "page");                // `page`
define_token!(Link, "link");                // `link`
define_token!(LinkTo, "linkto");            // `linkto`
define_token!(Embed, "embed");              // `embed`
define_token!(InCat, "incat");              // `incat`
define_token!(Prefix, "prefix");            // `prefix`
define_token!(Toggle, "toggle");            // `toggle`
define_token!(Limit, "limit");              // `limit`
define_token!(Resolve, "resolve");          // `resolve`
define_token!(Ns, "ns");                    // `ns`
define_token!(Depth, "depth");              // `depth`
define_token!(NoRedir, "noredir");          // `noredir`
define_token!(OnlyRedir, "onlyredir");      // `onlyredir`
define_token!(Direct, "direct");            // `direct`
