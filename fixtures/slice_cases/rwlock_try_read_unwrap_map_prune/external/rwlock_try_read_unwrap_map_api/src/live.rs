pub fn selected_rwlock_try_read_unwrap_map_report(raw: &str) -> String {
    rwlock_try_read_unwrap_map_model::selected_rwlock_try_read_unwrap_map(raw)
}

pub fn dead_live_rwlock_try_read_unwrap_map_report(raw: &str) -> String {
    format!("dead-live-rwlock-try-read-unwrap-map-report:{raw}")
}
