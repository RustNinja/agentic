use opensourced::opensourced;

#[opensourced]
pub fn selected_arc_mutex_lock_map_report(raw: &str) -> String {
    arc_mutex_lock_api::selected_arc_mutex_lock_map_report(raw)
}

pub fn dead_arc_mutex_lock_map_report(raw: &str) -> String {
    arc_mutex_lock_api::dead_arc_mutex_lock_map_report(raw)
}
