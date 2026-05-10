pub fn selected_once_lock_take_map_report(raw: &str) -> String {
    once_lock_take_map_model::selected_once_lock_take_map(raw)
}

pub fn dead_live_once_lock_take_map_report(raw: &str) -> String {
    format!("dead-live-once-lock-take-map-report:{raw}")
}
