use opensourced::opensourced;

#[opensourced]
pub fn selected_string_extend_chars_map_report(raw: &str) -> String {
    string_extend_chars_map_api::selected_string_extend_chars_map_report(raw)
}

pub fn dead_string_extend_chars_map_report(raw: &str) -> String {
    string_extend_chars_map_api::dead_string_extend_chars_map_report(raw)
}
