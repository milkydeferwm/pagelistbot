use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, AttributeArgs};

#[proc_macro_attribute]
pub fn parse_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_tokens = parse_macro_input!(attr as AttributeArgs);
    let item_tokens = parse_macro_input!(item as ItemFn);

    assert!(attr_tokens.len() == 2);
    let test_fn_name = &attr_tokens[0];
    let test_fn_input = &attr_tokens[1];
    let test_fn = &item_tokens.sig.ident;

    let output = quote!(
        #item_tokens

        #[cfg(test)]
        #[test]
        fn #test_fn_name () {
            use nom::Finish;
            use std::collections::BTreeSet;

            let input = include!( #test_fn_input );
            for (input, expected) in input {
                let result = #test_fn ::<nom::error::VerboseError<nom_locate::LocatedSpan<&str>>>(nom_locate::LocatedSpan::new(input)).finish().map(|(_, res)| res);
                assert_eq!(result, expected);
            }
        }
    );

    TokenStream::from(output)
}
