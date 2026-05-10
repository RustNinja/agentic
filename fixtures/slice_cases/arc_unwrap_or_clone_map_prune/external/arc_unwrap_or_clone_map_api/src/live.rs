pub fn selected_arc_unwrap_or_clone_map_report(raw: &str) -> String {
    arc_unwrap_or_clone_map_model::selected_arc_unwrap_or_clone_map(raw)
}

pub fn dead_live_arc_unwrap_or_clone_map_report(raw: &str) -> String {
    format!("dead-live-arc-unwrap-or-clone-map-report:{raw}")
}
