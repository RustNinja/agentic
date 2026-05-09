use opensourced::opensourced;

#[opensourced]
pub fn selected_rposition_report(raw: &str) -> String {
    rposition_api::selected_rposition_report(raw)
}

pub fn dead_rposition_report(raw: &str) -> String {
    rposition_api::dead_rposition_report(raw)
}
