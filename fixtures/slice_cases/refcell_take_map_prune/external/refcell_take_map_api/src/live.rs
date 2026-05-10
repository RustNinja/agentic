pub fn selected_refcell_take_map_report(raw: &str) -> String {
    refcell_take_map_model::selected_refcell_take_map(raw)
}

pub fn dead_live_refcell_take_map_report(raw: &str) -> String {
    format!("dead-live-refcell-take-map-report:{raw}")
}
