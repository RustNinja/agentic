use opensourced::opensourced;

#[opensourced]
pub fn selected_option_unwrap_or_default_map_report(raw: &str) -> String {
    option_unwrap_or_default_map_api::selected_option_unwrap_or_default_map_report(raw)
}

pub fn dead_option_unwrap_or_default_map_report(raw: &str) -> String {
    option_unwrap_or_default_map_api::dead_option_unwrap_or_default_map_report(raw)
}
