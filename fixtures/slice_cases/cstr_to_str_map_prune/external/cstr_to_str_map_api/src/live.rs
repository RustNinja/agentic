pub fn selected_cstr_to_str_map_report(raw: &str) -> String {
    cstr_to_str_map_model::selected_cstr_to_str_map(raw)
}

pub fn dead_live_cstr_to_str_map_report(raw: &str) -> String {
    format!("dead-cstr-to-str-map-live-report:{raw}")
}
