pub fn selected_string_shrink_to_fit_map_report(raw: &str) -> String {
    string_shrink_to_fit_map_model::selected_string_shrink_to_fit_map(raw)
}

pub fn dead_live_string_shrink_to_fit_map_report(raw: &str) -> String {
    format!("dead-string-shrink-to-fit-map-live-report:{raw}")
}
