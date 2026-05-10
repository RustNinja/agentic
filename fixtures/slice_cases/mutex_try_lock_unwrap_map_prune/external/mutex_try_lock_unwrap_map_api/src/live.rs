pub fn selected_mutex_try_lock_unwrap_map_report(raw: &str) -> String {
    mutex_try_lock_unwrap_map_model::selected_mutex_try_lock_unwrap_map(raw)
}

pub fn dead_live_mutex_try_lock_unwrap_map_report(raw: &str) -> String {
    format!("dead-live-mutex-try-lock-unwrap-map-report:{raw}")
}
