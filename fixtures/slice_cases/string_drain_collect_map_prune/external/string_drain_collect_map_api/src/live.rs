pub fn selected_string_drain_collect_map_report(raw: &str) -> String {
    string_drain_collect_map_model::selected_string_drain_collect_map(raw)
}

pub fn dead_live_string_drain_collect_map_report(raw: &str) -> String {
    format!("dead-string-drain-collect-map-live-report:{raw}")
}
