pub fn selected_rc_unwrap_or_clone_map_report(raw: &str) -> String {
    rc_unwrap_or_clone_map_model::selected_rc_unwrap_or_clone_map(raw)
}

pub fn dead_live_rc_unwrap_or_clone_map_report(raw: &str) -> String {
    format!("dead-live-rc-unwrap-or-clone-map-report:{raw}")
}
