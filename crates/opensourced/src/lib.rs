use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn opensourced(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        let mut error = "compile_error!(\"#[opensourced] does not accept arguments\");"
            .parse::<TokenStream>()
            .expect("static compile_error token stream should parse");
        error.extend(item);
        return error;
    }

    item
}
