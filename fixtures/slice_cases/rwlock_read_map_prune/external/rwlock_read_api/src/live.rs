pub fn selected_rwlock_read_map_report(raw: &str) -> String {
    rwlock_read_model::selected_rwlock_read_map(raw)
}

pub fn dead_live_rwlock_read_map_report(raw: &str) -> String {
    format!("dead-live-rwlock-read-map:{raw}")
}
