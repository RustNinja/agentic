pub fn selected_str_rsplit_map_report(raw: &str) -> String {
    str_rsplit_map_model::selected_str_rsplit_map(raw)
}

pub fn dead_live_str_rsplit_map_report(raw: &str) -> String {
    format!("dead-str-rsplit-map-live-report:{raw}")
}
