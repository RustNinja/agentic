pub fn selected_rc_into_inner_map_report(raw: &str) -> String {
    rc_into_inner_map_model::selected_rc_into_inner_map(raw)
}

pub fn dead_live_rc_into_inner_map_report(raw: &str) -> String {
    format!("dead-live-rc-into-inner-map-report:{raw}")
}
