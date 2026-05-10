pub fn selected_vec_windows_map_report(raw: &str) -> String {
    vec_windows_map_model::selected_vec_windows_map(raw)
}

pub fn dead_live_vec_windows_map_report(raw: &str) -> String {
    format!("dead-vec-windows-map-live-report:{raw}")
}
