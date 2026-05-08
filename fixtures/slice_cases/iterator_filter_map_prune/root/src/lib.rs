use opensourced::opensourced;

#[opensourced]
pub fn selected_filter_report(raw: &str) -> String {
    filter_api::selected_filter_report(raw)
}

pub fn dead_filter_report(raw: &str) -> String {
    filter_api::dead_filter_report(raw)
}
