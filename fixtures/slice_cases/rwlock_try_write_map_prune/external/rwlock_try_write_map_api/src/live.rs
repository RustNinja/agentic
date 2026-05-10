pub fn selected_rwlock_try_write_map_report(raw: &str) -> String {
    rwlock_try_write_map_model::selected_rwlock_try_write_map(raw)
}

pub fn dead_live_rwlock_try_write_map_report(raw: &str) -> String {
    format!("dead-live-rwlock-try-write-map-report:{raw}")
}
