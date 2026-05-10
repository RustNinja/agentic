pub fn selected_string_push_str_map_report(raw: &str) -> String {
    string_push_str_map_model::selected_string_push_str_map(raw)
}

pub fn dead_live_string_push_str_map_report(raw: &str) -> String {
    format!("dead-string-push-str-map-live-report:{raw}")
}
