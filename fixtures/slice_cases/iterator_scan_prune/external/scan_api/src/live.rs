pub fn selected_scan_report(raw: &str) -> String {
    scan_model::selected_scan(raw)
}

pub fn dead_live_scan_report(raw: &str) -> String {
    format!("dead-live-scan:{raw}")
}
