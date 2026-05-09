use opensourced::opensourced;

#[opensourced]
pub fn selected_str_strip_prefix_map_report(raw: &str) -> String {
    str_strip_prefix_map_api::selected_str_strip_prefix_map_report(raw)
}

pub fn dead_str_strip_prefix_map_report(raw: &str) -> String {
    str_strip_prefix_map_api::dead_str_strip_prefix_map_report(raw)
}
