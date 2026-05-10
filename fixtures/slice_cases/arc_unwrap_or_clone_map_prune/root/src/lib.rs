use opensourced::opensourced;

#[opensourced]
pub fn selected_arc_unwrap_or_clone_map_report(raw: &str) -> String {
    arc_unwrap_or_clone_map_api::selected_arc_unwrap_or_clone_map_report(raw)
}

pub fn dead_arc_unwrap_or_clone_map_report(raw: &str) -> String {
    arc_unwrap_or_clone_map_api::dead_arc_unwrap_or_clone_map_report(raw)
}
