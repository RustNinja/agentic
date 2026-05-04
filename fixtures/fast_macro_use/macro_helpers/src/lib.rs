use proc_macro::TokenStream;

#[proc_macro_derive(FixtureDerive, attributes(fixture_helper))]
pub fn fixture_derive(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn fixture_attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
