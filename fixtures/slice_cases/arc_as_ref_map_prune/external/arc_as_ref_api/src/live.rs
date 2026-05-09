pub fn selected_arc_as_ref_map_report(raw: &str) -> String {
    arc_as_ref_model::selected_arc_as_ref_map(raw)
}

pub fn dead_live_arc_as_ref_map_report(raw: &str) -> String {
    format!("dead-live-arc-as-ref-map:{raw}")
}
