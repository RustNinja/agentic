pub fn selected_arc_into_inner_map_report(raw: &str) -> String {
    arc_into_inner_map_model::selected_arc_into_inner_map(raw)
}

pub fn dead_live_arc_into_inner_map_report(raw: &str) -> String {
    format!("dead-live-arc-into-inner-map-report:{raw}")
}
