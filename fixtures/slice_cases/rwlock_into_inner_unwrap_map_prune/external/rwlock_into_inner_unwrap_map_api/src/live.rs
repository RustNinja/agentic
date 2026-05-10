pub fn selected_rwlock_into_inner_unwrap_map_report(raw: &str) -> String {
    rwlock_into_inner_unwrap_map_model::selected_rwlock_into_inner_unwrap_map(raw)
}

pub fn dead_live_rwlock_into_inner_unwrap_map_report(raw: &str) -> String {
    format!("dead-live-rwlock-into-inner-unwrap-map-report:{raw}")
}
