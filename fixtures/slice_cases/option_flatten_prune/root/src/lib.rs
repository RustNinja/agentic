use opensourced::opensourced;

#[opensourced]
pub fn selected_option_flatten_report(raw: &str) -> String {
    option_flatten_api::selected_option_flatten_report(raw)
}

pub fn dead_option_flatten_report(raw: &str) -> String {
    option_flatten_api::dead_option_flatten_report(raw)
}
