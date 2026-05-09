use opensourced::opensourced;

#[opensourced]
pub fn selected_rwlock_write_map_report(raw: &str) -> String {
    rwlock_write_api::selected_rwlock_write_map_report(raw)
}

pub fn dead_rwlock_write_map_report(raw: &str) -> String {
    rwlock_write_api::dead_rwlock_write_map_report(raw)
}
