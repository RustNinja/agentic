use opensourced::opensourced;

#[opensourced]
pub fn selected_matches_guard_report(raw: &str) -> String {
    matches_guard_api::selected_matches_guard_report(raw)
}

pub fn dead_matches_guard_report(raw: &str) -> String {
    matches_guard_api::dead_matches_guard_report(raw)
}
