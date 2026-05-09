pub fn selected_str_char_escape_default_map_report(raw: &str) -> String {
    str_char_escape_default_map_model::selected_str_char_escape_default_map(raw)
}

pub fn dead_live_str_char_escape_default_map_report(raw: &str) -> String {
    format!("dead-str-char-escape-default-map-live-report:{raw}")
}
