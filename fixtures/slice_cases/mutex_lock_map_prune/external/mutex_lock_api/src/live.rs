pub fn selected_mutex_lock_map_report(raw: &str) -> String {
    mutex_lock_model::selected_mutex_lock_map(raw)
}

pub fn dead_live_mutex_lock_map_report(raw: &str) -> String {
    format!("dead-live-mutex-lock-map:{raw}")
}
