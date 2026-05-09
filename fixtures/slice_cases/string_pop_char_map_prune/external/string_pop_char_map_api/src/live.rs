pub fn selected_string_pop_char_map_report(raw: &str) -> String {
    string_pop_char_map_model::selected_string_pop_char_map(raw)
}

pub fn dead_live_string_pop_char_map_report(raw: &str) -> String {
    format!("dead-string-pop-char-map-live-report:{raw}")
}
