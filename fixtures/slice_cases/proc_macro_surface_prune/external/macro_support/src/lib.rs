use proc_macro::TokenStream;

#[proc_macro_derive(SurfaceRecord, attributes(surface_helper))]
pub fn surface_record(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn surface_attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_derive(DeadRecord, attributes(surface_helper))]
pub fn dead_record(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn dead_attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
