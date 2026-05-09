pub fn selected_mutex_lock_mut_map_report(raw: &str) -> String {
    mutex_lock_mut_model::selected_mutex_lock_mut_map(raw)
}

pub fn dead_live_mutex_lock_mut_map_report(raw: &str) -> String {
    format!("dead-live-mutex-lock-mut-map:{raw}")
}
