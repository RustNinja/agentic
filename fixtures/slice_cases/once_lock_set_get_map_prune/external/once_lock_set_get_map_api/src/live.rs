pub fn selected_once_lock_set_get_map_report(raw: &str) -> String {
    once_lock_set_get_map_model::selected_once_lock_set_get_map(raw)
}

pub fn dead_live_once_lock_set_get_map_report(raw: &str) -> String {
    format!("dead-once-lock-set-get-map-live-report:{raw}")
}
