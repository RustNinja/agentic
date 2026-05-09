use opensourced::opensourced;

#[opensourced]
pub fn selected_last_report(raw: &str) -> String {
    last_api::selected_last_report(raw)
}

pub fn dead_last_report(raw: &str) -> String {
    last_api::dead_last_report(raw)
}
