use opensourced::opensourced;

#[opensourced]
pub fn selected_max_by_report(raw: &str) -> String {
    max_by_api::selected_max_by_report(raw)
}

pub fn dead_max_by_report(raw: &str) -> String {
    max_by_api::dead_max_by_report(raw)
}
