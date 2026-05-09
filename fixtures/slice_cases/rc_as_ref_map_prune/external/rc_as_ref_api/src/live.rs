pub fn selected_rc_as_ref_map_report(raw: &str) -> String {
    rc_as_ref_model::selected_rc_as_ref_map(raw)
}

pub fn dead_live_rc_as_ref_map_report(raw: &str) -> String {
    format!("dead-live-rc-as-ref-map:{raw}")
}
