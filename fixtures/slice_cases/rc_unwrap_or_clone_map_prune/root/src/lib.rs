use opensourced::opensourced;

#[opensourced]
pub fn selected_rc_unwrap_or_clone_map_report(raw: &str) -> String {
    rc_unwrap_or_clone_map_api::selected_rc_unwrap_or_clone_map_report(raw)
}

pub fn dead_rc_unwrap_or_clone_map_report(raw: &str) -> String {
    rc_unwrap_or_clone_map_api::dead_rc_unwrap_or_clone_map_report(raw)
}
