use opensourced::opensourced;

#[opensourced]
pub fn selected_all_report(raw: &str) -> String {
    all_api::selected_all_report(raw)
}

pub fn dead_all_report(raw: &str) -> String {
    all_api::dead_all_report(raw)
}
