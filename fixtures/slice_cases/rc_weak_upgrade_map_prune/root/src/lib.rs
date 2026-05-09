use opensourced::opensourced;

#[opensourced]
pub fn selected_rc_weak_upgrade_map_report(raw: &str) -> String {
    rc_weak_upgrade_map_api::selected_rc_weak_upgrade_map_report(raw)
}

pub fn dead_rc_weak_upgrade_map_report(raw: &str) -> String {
    rc_weak_upgrade_map_api::dead_rc_weak_upgrade_map_report(raw)
}
