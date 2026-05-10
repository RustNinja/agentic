pub fn selected_str_replace_map_report(raw: &str) -> String {
    str_replace_map_model::selected_str_replace_map(raw)
}

pub fn dead_live_str_replace_map_report(raw: &str) -> String {
    format!("dead-str-replace-map-live-report:{raw}")
}
