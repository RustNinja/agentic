pub fn selected_str_chars_filter_map_report(raw: &str) -> String {
    str_chars_filter_map_model::selected_str_chars_filter_map(raw)
}

pub fn dead_live_str_chars_filter_map_report(raw: &str) -> String {
    format!("dead-str-chars-filter-map-live-report:{raw}")
}
