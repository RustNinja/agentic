pub fn selected_rc_make_mut_map_report(raw: &str) -> String {
    rc_make_mut_map_model::selected_rc_make_mut_map(raw)
}

pub fn dead_live_rc_make_mut_map_report(raw: &str) -> String {
    format!("dead-live-rc-make-mut-map:{raw}")
}
