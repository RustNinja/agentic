use opensourced::opensourced;

#[opensourced]
pub fn selected_once_lock_get_or_init_map_report(raw: &str) -> String {
    once_lock_get_or_init_api::selected_once_lock_get_or_init_map_report(raw)
}

pub fn dead_once_lock_get_or_init_map_report(raw: &str) -> String {
    once_lock_get_or_init_api::dead_once_lock_get_or_init_map_report(raw)
}
