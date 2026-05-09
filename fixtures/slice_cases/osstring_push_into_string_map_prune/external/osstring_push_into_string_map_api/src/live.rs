pub fn selected_osstring_push_into_string_map_report(raw: &str) -> String {
    osstring_push_into_string_map_model::selected_osstring_push_into_string_map(raw)
}

pub fn dead_live_osstring_push_into_string_map_report(raw: &str) -> String {
    format!("dead-osstring-push-into-string-map-live-report:{raw}")
}
