pub fn selected_arc_mutex_lock_map_report(raw: &str) -> String {
    arc_mutex_lock_model::selected_arc_mutex_lock_map(raw)
}

pub fn dead_live_arc_mutex_lock_map_report(raw: &str) -> String {
    format!("dead-live-arc-mutex-lock-map:{raw}")
}
