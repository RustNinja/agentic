use opensourced::opensourced;

#[opensourced]
pub fn selected_rwlock_get_mut_map_report(raw: &str) -> String {
    rwlock_get_mut_map_api::selected_rwlock_get_mut_map_report(raw)
}

pub fn dead_rwlock_get_mut_map_report(raw: &str) -> String {
    rwlock_get_mut_map_api::dead_rwlock_get_mut_map_report(raw)
}
