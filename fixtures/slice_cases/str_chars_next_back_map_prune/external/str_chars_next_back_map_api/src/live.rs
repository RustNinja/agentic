pub fn selected_str_chars_next_back_map_report(raw: &str) -> String {
    str_chars_next_back_map_model::selected_str_chars_next_back_map(raw)
}

pub fn dead_live_str_chars_next_back_map_report(raw: &str) -> String {
    format!("dead-str-chars-next-back-map-live-report:{raw}")
}
