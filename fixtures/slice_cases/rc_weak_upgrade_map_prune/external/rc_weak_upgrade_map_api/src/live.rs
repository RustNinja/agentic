pub fn selected_rc_weak_upgrade_map_report(raw: &str) -> String {
    rc_weak_upgrade_map_model::selected_rc_weak_upgrade_map(raw)
}

pub fn dead_live_rc_weak_upgrade_map_report(raw: &str) -> String {
    format!("dead-live-rc-weak-upgrade-map:{raw}")
}
