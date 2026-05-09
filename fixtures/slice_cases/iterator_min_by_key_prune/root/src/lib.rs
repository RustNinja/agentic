use opensourced::opensourced;

#[opensourced]
pub fn selected_min_by_key_report(raw: &str) -> String {
    min_by_key_api::selected_min_by_key_report(raw)
}

pub fn dead_min_by_key_report(raw: &str) -> String {
    min_by_key_api::dead_min_by_key_report(raw)
}
