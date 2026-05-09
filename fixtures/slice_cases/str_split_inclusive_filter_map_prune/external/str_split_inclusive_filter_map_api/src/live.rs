pub fn selected_str_split_inclusive_filter_map_report(raw: &str) -> String {
    str_split_inclusive_filter_map_model::selected_str_split_inclusive_filter_map(raw)
}

pub fn dead_live_str_split_inclusive_filter_map_report(raw: &str) -> String {
    format!("dead-str-split-inclusive-filter-map-live-report:{raw}")
}
