use opensourced::opensourced;

#[opensourced]
pub fn selected_index_report(raw: &str) -> String {
    index_api::selected_index_report(raw)
}

pub fn dead_index_report(raw: &str) -> String {
    index_api::dead_index_report(raw)
}
