use opensourced::opensourced;

#[opensourced]
pub fn selected_pattern_report(raw: &str) -> String {
    pattern_api::selected_pattern_report(raw)
}

pub fn dead_pattern_report(raw: &str) -> String {
    pattern_api::dead_pattern_report(raw)
}
