pub fn selected_str_char_indices_filter_map_report(raw: &str) -> String {
    str_char_indices_filter_map_model::selected_str_char_indices_filter_map(raw)
}

pub fn dead_live_str_char_indices_filter_map_report(raw: &str) -> String {
    format!("dead-str-char-indices-filter-map-live-report:{raw}")
}
