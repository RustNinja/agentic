use opensourced::opensourced;

#[opensourced]
pub fn selected_parse_report(raw: &str) -> String {
    parse_api::selected_parse_report(raw)
}

pub fn dead_parse_report(raw: &str) -> String {
    parse_api::dead_parse_report(raw)
}
