pub fn selected_once_lock_get_mut_map_report(raw: &str) -> String {
    once_lock_get_mut_map_model::selected_once_lock_get_mut_map(raw)
}

pub fn dead_live_once_lock_get_mut_map_report(raw: &str) -> String {
    format!("dead-live-once-lock-get-mut-map-report:{raw}")
}
