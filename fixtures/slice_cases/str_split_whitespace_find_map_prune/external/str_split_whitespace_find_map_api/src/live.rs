pub fn selected_str_split_whitespace_find_map_report(raw: &str) -> String {
    str_split_whitespace_find_map_model::selected_str_split_whitespace_find_map(raw)
}

pub fn dead_live_str_split_whitespace_find_map_report(raw: &str) -> String {
    format!("dead-str-split-whitespace-find-map-live-report:{raw}")
}
