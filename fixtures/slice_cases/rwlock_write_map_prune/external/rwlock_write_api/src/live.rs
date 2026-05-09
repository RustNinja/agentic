pub fn selected_rwlock_write_map_report(raw: &str) -> String {
    rwlock_write_model::selected_rwlock_write_map(raw)
}

pub fn dead_live_rwlock_write_map_report(raw: &str) -> String {
    format!("dead-live-rwlock-write-map:{raw}")
}
