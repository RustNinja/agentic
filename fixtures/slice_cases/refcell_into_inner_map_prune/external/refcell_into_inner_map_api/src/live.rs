pub fn selected_refcell_into_inner_map_report(raw: &str) -> String {
    refcell_into_inner_map_model::selected_refcell_into_inner_map(raw)
}

pub fn dead_live_refcell_into_inner_map_report(raw: &str) -> String {
    format!("dead-live-refcell-into-inner-map-report:{raw}")
}
