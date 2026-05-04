use proc_macro::TokenStream;

#[proc_macro_derive(FixtureDerive, attributes(fixture_helper))]
pub fn fixture_derive(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_derive(FixtureRecord, attributes(fixture_helper, fixture_serde))]
pub fn fixture_record(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_derive(FixtureEnum, attributes(fixture_helper, fixture_serde))]
pub fn fixture_enum(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_derive(FixtureObject, attributes(fixture_helper))]
pub fn fixture_object(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_derive(FixtureError, attributes(fixture_error))]
pub fn fixture_error(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn fixture_attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn fixture_export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn fixture_constructor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
