use opensourced::opensourced;

#[opensourced]
pub fn selected_retain_report(raw: &str) -> String {
    retain_api::selected_retain_report(raw)
}

pub fn dead_retain_report(raw: &str) -> String {
    retain_api::dead_retain_report(raw)
}
