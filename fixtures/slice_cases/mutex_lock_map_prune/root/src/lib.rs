use opensourced::opensourced;

#[opensourced]
pub fn selected_mutex_lock_map_report(raw: &str) -> String {
    mutex_lock_api::selected_mutex_lock_map_report(raw)
}

pub fn dead_mutex_lock_map_report(raw: &str) -> String {
    mutex_lock_api::dead_mutex_lock_map_report(raw)
}
