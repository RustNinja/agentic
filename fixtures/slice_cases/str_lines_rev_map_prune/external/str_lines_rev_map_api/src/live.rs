pub fn selected_str_lines_rev_map_report(raw: &str) -> String {
    str_lines_rev_map_model::selected_str_lines_rev_map(raw)
}

pub fn dead_live_str_lines_rev_map_report(raw: &str) -> String {
    format!("dead-str-lines-rev-map-live-report:{raw}")
}
