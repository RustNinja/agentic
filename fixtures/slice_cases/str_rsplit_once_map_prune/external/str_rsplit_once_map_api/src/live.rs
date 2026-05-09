pub fn selected_str_rsplit_once_map_report(raw: &str) -> String {
    str_rsplit_once_map_model::selected_str_rsplit_once_map(raw)
}

pub fn dead_live_str_rsplit_once_map_report(raw: &str) -> String {
    format!("dead-str-rsplit-once-map-live-report:{raw}")
}
