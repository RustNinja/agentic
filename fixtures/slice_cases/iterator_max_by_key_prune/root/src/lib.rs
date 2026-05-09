use opensourced::opensourced;

#[opensourced]
pub fn selected_max_by_key_report(raw: &str) -> String {
    max_by_key_api::selected_max_by_key_report(raw)
}

pub fn dead_max_by_key_report(raw: &str) -> String {
    max_by_key_api::dead_max_by_key_report(raw)
}
