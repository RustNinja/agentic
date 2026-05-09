pub fn selected_str_split_once_map_report(raw: &str) -> String {
    str_split_once_map_model::selected_str_split_once_map(raw)
}

pub fn dead_live_str_split_once_map_report(raw: &str) -> String {
    format!("dead-str-split-once-map-live-report:{raw}")
}
