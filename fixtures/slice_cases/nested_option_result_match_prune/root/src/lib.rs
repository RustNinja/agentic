use opensourced::opensourced;

#[opensourced]
pub fn selected_nested_match_report(raw: &str) -> String {
    nested_match_api::selected_nested_match_report(raw)
}

pub fn dead_nested_match_report(raw: &str) -> String {
    nested_match_api::dead_nested_match_report(raw)
}
