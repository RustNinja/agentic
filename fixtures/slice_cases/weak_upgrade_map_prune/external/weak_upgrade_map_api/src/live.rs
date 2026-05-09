pub fn selected_weak_upgrade_map_report(raw: &str) -> String {
    weak_upgrade_map_model::selected_weak_upgrade_map(raw)
}

pub fn dead_live_weak_upgrade_map_report(raw: &str) -> String {
    format!("dead-live-weak-upgrade-map:{raw}")
}
