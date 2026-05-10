pub fn selected_string_reserve_push_map_report(raw: &str) -> String {
    string_reserve_push_map_model::selected_string_reserve_push_map(raw)
}

pub fn dead_live_string_reserve_push_map_report(raw: &str) -> String {
    format!("dead-string-reserve-push-map-live-report:{raw}")
}
