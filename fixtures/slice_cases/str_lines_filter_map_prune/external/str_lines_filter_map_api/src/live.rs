pub fn selected_str_lines_filter_map_report(raw: &str) -> String {
    str_lines_filter_map_model::selected_str_lines_filter_map(raw)
}

pub fn dead_live_str_lines_filter_map_report(raw: &str) -> String {
    format!("dead-str-lines-filter-map-live-report:{raw}")
}
