use opensourced::opensourced;

#[opensourced]
pub fn selected_struct_filter_report(raw: &str) -> String {
    struct_filter_api::selected_struct_filter_report(raw)
}

pub fn dead_struct_filter_report(raw: &str) -> String {
    struct_filter_api::dead_struct_filter_report(raw)
}
