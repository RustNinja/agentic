use opensourced::opensourced;

#[opensourced]
pub fn selected_osstr_to_str_map_report(raw: &str) -> String {
    osstr_to_str_map_api::selected_osstr_to_str_map_report(raw)
}

pub fn dead_osstr_to_str_map_report(raw: &str) -> String {
    osstr_to_str_map_api::dead_osstr_to_str_map_report(raw)
}
