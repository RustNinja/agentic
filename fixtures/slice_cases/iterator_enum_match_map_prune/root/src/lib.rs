use opensourced::opensourced;

#[opensourced]
pub fn selected_enum_match_report(raw: &str) -> String {
    enum_match_api::selected_enum_match_report(raw)
}

pub fn dead_enum_match_report(raw: &str) -> String {
    enum_match_api::dead_enum_match_report(raw)
}
