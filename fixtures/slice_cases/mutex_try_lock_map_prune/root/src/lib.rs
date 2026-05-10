use opensourced::opensourced;

#[opensourced]
pub fn selected_mutex_try_lock_map_report(raw: &str) -> String {
    mutex_try_lock_map_api::selected_mutex_try_lock_map_report(raw)
}

pub fn dead_mutex_try_lock_map_report(raw: &str) -> String {
    mutex_try_lock_map_api::dead_mutex_try_lock_map_report(raw)
}
