pub fn selected_arc_get_mut_map_report(raw: &str) -> String {
    arc_get_mut_map_model::selected_arc_get_mut_map(raw)
}

pub fn dead_live_arc_get_mut_map_report(raw: &str) -> String {
    format!("dead-arc-get-mut-map-live-report:{raw}")
}
