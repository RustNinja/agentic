use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

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

    let function = parse_macro_input!(item as ItemFn);

    quote!(#function).into()
}
