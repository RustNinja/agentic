use opensourced::opensourced;

#[opensourced]
pub fn selected_string_shrink_to_fit_map_report(raw: &str) -> String {
    string_shrink_to_fit_map_api::selected_string_shrink_to_fit_map_report(raw)
}

pub fn dead_string_shrink_to_fit_map_report(raw: &str) -> String {
    string_shrink_to_fit_map_api::dead_string_shrink_to_fit_map_report(raw)
}
