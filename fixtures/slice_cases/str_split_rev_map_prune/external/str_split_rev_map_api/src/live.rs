pub fn selected_str_split_rev_map_report(raw: &str) -> String {
    str_split_rev_map_model::selected_str_split_rev_map(raw)
}

pub fn dead_live_str_split_rev_map_report(raw: &str) -> String {
    format!("dead-str-split-rev-map-live-report:{raw}")
}
