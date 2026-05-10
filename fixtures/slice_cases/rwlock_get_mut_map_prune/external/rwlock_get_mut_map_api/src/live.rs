pub fn selected_rwlock_get_mut_map_report(raw: &str) -> String {
    rwlock_get_mut_map_model::selected_rwlock_get_mut_map(raw)
}

pub fn dead_live_rwlock_get_mut_map_report(raw: &str) -> String {
    format!("dead-rwlock-get-mut-map-live-report:{raw}")
}
