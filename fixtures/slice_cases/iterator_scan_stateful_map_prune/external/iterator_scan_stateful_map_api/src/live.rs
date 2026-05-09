pub fn selected_iterator_scan_stateful_map_report(raw: &str) -> String {
    iterator_scan_stateful_map_model::selected_iterator_scan_stateful_map(raw)
}

pub fn dead_live_iterator_scan_stateful_map_report(raw: &str) -> String {
    format!("dead-iterator-scan-stateful-map-live-report:{raw}")
}
