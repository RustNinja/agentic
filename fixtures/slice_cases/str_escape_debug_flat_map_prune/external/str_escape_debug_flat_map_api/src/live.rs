pub fn selected_str_escape_debug_flat_map_report(raw: &str) -> String {
    str_escape_debug_flat_map_model::selected_str_escape_debug_flat_map(raw)
}

pub fn dead_live_str_escape_debug_flat_map_report(raw: &str) -> String {
    format!("dead-str-escape-debug-flat-map-live-report:{raw}")
}
