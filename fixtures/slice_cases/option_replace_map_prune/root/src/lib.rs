use opensourced::opensourced;

#[opensourced]
pub fn selected_option_replace_map_report(raw: &str) -> String {
    option_replace_map_api::selected_option_replace_map_report(raw)
}

pub fn dead_option_replace_map_report(raw: &str) -> String {
    option_replace_map_api::dead_option_replace_map_report(raw)
}
