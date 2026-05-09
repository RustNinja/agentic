pub fn selected_str_strip_suffix_map_report(raw: &str) -> String {
    str_strip_suffix_map_model::selected_str_strip_suffix_map(raw)
}

pub fn dead_live_str_strip_suffix_map_report(raw: &str) -> String {
    format!("dead-str-strip-suffix-map-live-report:{raw}")
}
