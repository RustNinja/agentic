use opensourced::opensourced;

#[opensourced]
pub fn selected_cstr_to_str_map_report(raw: &str) -> String {
    cstr_to_str_map_api::selected_cstr_to_str_map_report(raw)
}

pub fn dead_cstr_to_str_map_report(raw: &str) -> String {
    cstr_to_str_map_api::dead_cstr_to_str_map_report(raw)
}
