use opensourced::opensourced;

#[opensourced]
pub fn selected_once_lock_set_get_map_report(raw: &str) -> String {
    once_lock_set_get_map_api::selected_once_lock_set_get_map_report(raw)
}

pub fn dead_once_lock_set_get_map_report(raw: &str) -> String {
    once_lock_set_get_map_api::dead_once_lock_set_get_map_report(raw)
}
