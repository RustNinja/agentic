use opensourced::opensourced;

#[opensourced]
pub fn selected_string_insert_str_map_report(raw: &str) -> String {
    string_insert_str_map_api::selected_string_insert_str_map_report(raw)
}

pub fn dead_string_insert_str_map_report(raw: &str) -> String {
    string_insert_str_map_api::dead_string_insert_str_map_report(raw)
}
