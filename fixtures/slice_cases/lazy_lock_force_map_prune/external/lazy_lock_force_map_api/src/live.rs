pub fn selected_lazy_lock_force_map_report(raw: &str) -> String {
    lazy_lock_force_map_model::selected_lazy_lock_force_map(raw)
}

pub fn dead_live_lazy_lock_force_map_report(raw: &str) -> String {
    format!("dead-lazy-lock-force-map-live-report:{raw}")
}
