use opensourced::opensourced;

#[opensourced]
pub fn selected_macro_report(raw: &str) -> String {
    macro_api::selected_macro_report(raw)
}

pub fn dead_macro_report(raw: &str) -> String {
    macro_api::dead_macro_report(raw)
}
