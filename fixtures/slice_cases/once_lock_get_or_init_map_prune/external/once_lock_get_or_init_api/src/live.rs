pub fn selected_once_lock_get_or_init_map_report(raw: &str) -> String {
    once_lock_get_or_init_model::selected_once_lock_get_or_init_map(raw)
}

pub fn dead_live_once_lock_get_or_init_map_report(raw: &str) -> String {
    format!("dead-live-once-lock-get-or-init-map:{raw}")
}
