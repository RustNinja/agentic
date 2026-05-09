use opensourced::opensourced;

#[opensourced]
pub fn selected_option_copied_map_report(raw: &str) -> String {
    option_copied_api::selected_option_copied_map_report(raw)
}

pub fn dead_option_copied_map_report(raw: &str) -> String {
    option_copied_api::dead_option_copied_map_report(raw)
}
