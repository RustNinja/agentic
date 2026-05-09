pub fn selected_str_strip_prefix_map_report(raw: &str) -> String {
    str_strip_prefix_map_model::selected_str_strip_prefix_map(raw)
}

pub fn dead_live_str_strip_prefix_map_report(raw: &str) -> String {
    format!("dead-str-strip-prefix-map-live-report:{raw}")
}
