use proc_macro::TokenStream;

#[proc_macro_derive(Record, attributes(uniffi))]
pub fn record(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_derive(Enum, attributes(uniffi))]
pub fn enum_(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
