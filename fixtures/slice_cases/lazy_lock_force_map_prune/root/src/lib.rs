use opensourced::opensourced;

#[opensourced]
pub fn selected_lazy_lock_force_map_report(raw: &str) -> String {
    lazy_lock_force_map_api::selected_lazy_lock_force_map_report(raw)
}

pub fn dead_lazy_lock_force_map_report(raw: &str) -> String {
    lazy_lock_force_map_api::dead_lazy_lock_force_map_report(raw)
}
