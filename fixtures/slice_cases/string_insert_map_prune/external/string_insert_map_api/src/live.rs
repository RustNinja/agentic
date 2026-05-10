pub fn selected_string_insert_map_report(raw: &str) -> String {
    string_insert_map_model::selected_string_insert_map(raw)
}

pub fn dead_live_string_insert_map_report(raw: &str) -> String {
    format!("dead-string-insert-map-live-report:{raw}")
}
