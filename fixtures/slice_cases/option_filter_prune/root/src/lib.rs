use opensourced::opensourced;

#[opensourced]
pub fn selected_option_filter_report(raw: &str) -> String {
    option_filter_api::selected_option_filter_report(raw)
}

pub fn dead_option_filter_report(raw: &str) -> String {
    option_filter_api::dead_option_filter_report(raw)
}
