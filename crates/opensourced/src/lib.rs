use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn opensourced(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "#[opensourced] does not accept arguments",
        )
        .to_compile_error()
        .into();
    }

    item
}
