use opensourced::opensourced;

#[opensourced]
pub fn selected_weak_upgrade_map_report(raw: &str) -> String {
    weak_upgrade_map_api::selected_weak_upgrade_map_report(raw)
}

pub fn dead_weak_upgrade_map_report(raw: &str) -> String {
    weak_upgrade_map_api::dead_weak_upgrade_map_report(raw)
}
