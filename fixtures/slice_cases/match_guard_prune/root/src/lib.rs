use opensourced::opensourced;

#[opensourced]
pub fn selected_guard_report(raw: &str) -> String {
    guard_api::selected_guard_report(raw)
}

pub fn dead_guard_report(raw: &str) -> String {
    guard_api::dead_guard_report(raw)
}
