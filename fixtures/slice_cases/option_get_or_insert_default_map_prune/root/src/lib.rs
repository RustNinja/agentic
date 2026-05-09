use opensourced::opensourced;

#[opensourced]
pub fn selected_option_get_or_insert_default_map_report(raw: &str) -> String {
    option_get_or_insert_default_map_api::selected_option_get_or_insert_default_map_report(raw)
}

pub fn dead_option_get_or_insert_default_map_report(raw: &str) -> String {
    option_get_or_insert_default_map_api::dead_option_get_or_insert_default_map_report(raw)
}
