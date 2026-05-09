pub fn selected_str_split_ascii_whitespace_map_report(raw: &str) -> String {
    str_split_ascii_whitespace_map_model::selected_str_split_ascii_whitespace_map(raw)
}

pub fn dead_live_str_split_ascii_whitespace_map_report(raw: &str) -> String {
    format!("dead-str-split-ascii-whitespace-map-live-report:{raw}")
}
