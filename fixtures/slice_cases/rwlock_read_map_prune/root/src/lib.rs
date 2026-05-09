use opensourced::opensourced;

#[opensourced]
pub fn selected_rwlock_read_map_report(raw: &str) -> String {
    rwlock_read_api::selected_rwlock_read_map_report(raw)
}

pub fn dead_rwlock_read_map_report(raw: &str) -> String {
    rwlock_read_api::dead_rwlock_read_map_report(raw)
}
