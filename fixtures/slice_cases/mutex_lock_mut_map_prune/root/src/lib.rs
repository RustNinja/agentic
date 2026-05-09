use opensourced::opensourced;

#[opensourced]
pub fn selected_mutex_lock_mut_map_report(raw: &str) -> String {
    mutex_lock_mut_api::selected_mutex_lock_mut_map_report(raw)
}

pub fn dead_mutex_lock_mut_map_report(raw: &str) -> String {
    mutex_lock_mut_api::dead_mutex_lock_mut_map_report(raw)
}
