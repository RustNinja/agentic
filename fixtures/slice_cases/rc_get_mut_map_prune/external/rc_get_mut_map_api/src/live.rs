pub fn selected_rc_get_mut_map_report(raw: &str) -> String {
    rc_get_mut_map_model::selected_rc_get_mut_map(raw)
}

pub fn dead_live_rc_get_mut_map_report(raw: &str) -> String {
    format!("dead-rc-get-mut-map-live-report:{raw}")
}
