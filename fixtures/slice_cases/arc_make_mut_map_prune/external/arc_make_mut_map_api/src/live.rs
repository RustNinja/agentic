pub fn selected_arc_make_mut_map_report(raw: &str) -> String {
    arc_make_mut_map_model::selected_arc_make_mut_map(raw)
}

pub fn dead_live_arc_make_mut_map_report(raw: &str) -> String {
    format!("dead-live-arc-make-mut-map:{raw}")
}
