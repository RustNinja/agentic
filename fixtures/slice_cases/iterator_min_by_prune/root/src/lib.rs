use opensourced::opensourced;

#[opensourced]
pub fn selected_min_by_report(raw: &str) -> String {
    min_by_api::selected_min_by_report(raw)
}

pub fn dead_min_by_report(raw: &str) -> String {
    min_by_api::dead_min_by_report(raw)
}
