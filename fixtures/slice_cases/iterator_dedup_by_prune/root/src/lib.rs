use opensourced::opensourced;

#[opensourced]
pub fn selected_dedup_by_report(raw: &str) -> String {
    dedup_api::selected_dedup_by_report(raw)
}

pub fn dead_dedup_by_report(raw: &str) -> String {
    dedup_api::dead_dedup_by_report(raw)
}
