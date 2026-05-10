pub fn selected_mutex_try_lock_map_report(raw: &str) -> String {
    mutex_try_lock_map_model::selected_mutex_try_lock_map(raw)
}

pub fn dead_live_mutex_try_lock_map_report(raw: &str) -> String {
    format!("dead-live-mutex-try-lock-map-report:{raw}")
}
