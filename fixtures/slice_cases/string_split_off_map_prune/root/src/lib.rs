use opensourced::opensourced;

#[opensourced]
pub fn selected_string_split_off_map_report(raw: &str) -> String {
    string_split_off_map_api::selected_string_split_off_map_report(raw)
}

pub fn dead_string_split_off_map_report(raw: &str) -> String {
    string_split_off_map_api::dead_string_split_off_map_report(raw)
}
