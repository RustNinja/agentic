use opensourced::opensourced;

#[opensourced]
pub fn selected_rwlock_try_write_unwrap_map_report(raw: &str) -> String {
    rwlock_try_write_unwrap_map_api::selected_rwlock_try_write_unwrap_map_report(raw)
}

pub fn dead_rwlock_try_write_unwrap_map_report(raw: &str) -> String {
    rwlock_try_write_unwrap_map_api::dead_rwlock_try_write_unwrap_map_report(raw)
}
