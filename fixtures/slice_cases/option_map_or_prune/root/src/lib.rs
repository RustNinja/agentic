use opensourced::opensourced;

#[opensourced]
pub fn selected_option_map_or_value_report(raw: &str) -> String {
    option_map_or_value_api::selected_option_map_or_value_report(raw)
}

pub fn dead_option_map_or_value_report(raw: &str) -> String {
    option_map_or_value_api::dead_option_map_or_value_report(raw)
}
